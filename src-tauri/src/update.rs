//! Updates (F004/S04): the app asks GitHub's latest release for a newer version and, only when
//! asked, downloads it, checks its signature against the public key in `tauri.conf.json`,
//! installs it and restarts. Never during a sync. A version whose signature was refused isn't
//! offered again until the app restarts.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_updater::{Error as UpdaterError, Update, Updater, UpdaterExt};

use crate::runner::SyncRunner;

/// Debug builds only: another `latest.json`, to test a refused signature by hand.
#[cfg(debug_assertions)]
const ENDPOINT_OVERRIDE: &str = "FAMILYLIGHTBOX_UPDATE_ENDPOINT";

pub const REFUSED: &str = "Update geweigerd: de handtekening klopt niet";
pub const DURING_SYNC: &str = "bijwerken kan na de synchronisatie";

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Offer {
    pub version: String,
    pub notes: String,
}

#[derive(Clone, Serialize)]
struct Progress {
    done: u64,
    total: Option<u64>,
}

#[derive(Default)]
pub struct UpdateState {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    found: Option<Update>,
    installing: bool,
    refused: Option<String>,
}

impl UpdateState {
    pub fn installing(&self) -> bool {
        self.inner.lock().map(|i| i.installing).unwrap_or(false)
    }

    /// Keeps what a check found; `None` when there is nothing new, or only the refused version.
    fn found(&self, update: Option<Update>) -> Option<Offer> {
        let mut inner = self.inner.lock().ok()?;
        let update = update.filter(|u| inner.refused.as_deref() != Some(u.version.as_str()));
        let offer = update.as_ref().map(|u| Offer {
            version: u.version.clone(),
            notes: u.body.clone().unwrap_or_default(),
        });
        inner.found = update;
        offer
    }

    /// Claims the update found for installing, unless one is being installed already.
    fn begin(&self) -> Result<Update, String> {
        let mut inner = self.inner.lock().map_err(|e| e.to_string())?;
        if inner.installing {
            return Err("de update wordt al geïnstalleerd".into());
        }
        let update = inner.found.clone().ok_or("geen update gevonden")?;
        inner.installing = true;
        Ok(update)
    }

    fn failed(&self, version: &str, refused: bool) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.installing = false;
            if refused {
                inner.refused = Some(version.to_string());
                inner.found = None;
            }
        }
    }
}

fn is_refused(e: &UpdaterError) -> bool {
    matches!(
        e,
        UpdaterError::Minisign(_)
            | UpdaterError::SignatureUtf8(_)
            | UpdaterError::Base64(_)
            | UpdaterError::SignedVersionMismatch { .. }
            | UpdaterError::MissingSignedVersion
    )
}

fn updater(app: &AppHandle) -> Result<Updater, String> {
    #[cfg(debug_assertions)]
    if let Ok(url) = std::env::var(ENDPOINT_OVERRIDE) {
        let url = url.parse().map_err(|e| format!("{ENDPOINT_OVERRIDE}: {e}"))?;
        return app.updater_builder().endpoints(vec![url]).and_then(|b| b.build()).map_err(|e| e.to_string());
    }
    app.updater().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_check(app: AppHandle, state: State<'_, UpdateState>) -> Result<Option<Offer>, String> {
    let update = updater(&app)?.check().await.map_err(|e| e.to_string())?;
    Ok(state.found(update))
}

#[tauri::command]
pub async fn update_install(
    app: AppHandle,
    state: State<'_, UpdateState>,
    runner: State<'_, SyncRunner>,
) -> Result<(), String> {
    if runner.status().running {
        return Err(DURING_SYNC.into());
    }
    let update = state.begin()?;
    let mut done = 0u64;
    let emitter = app.clone();
    let result = update
        .download_and_install(
            move |chunk, total| {
                done += chunk as u64;
                let _ = emitter.emit("update-progress", Progress { done, total });
            },
            || {},
        )
        .await;
    match result {
        Ok(()) => app.restart(),
        Err(e) => {
            let refused = is_refused(&e);
            state.failed(&update.version, refused);
            Err(if refused { REFUSED.into() } else { format!("bijwerken mislukt: {e}") })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_needs_a_found_update_and_claims_it_once() {
        let state = UpdateState::default();
        assert_eq!(state.begin().err().as_deref(), Some("geen update gevonden"));
        assert!(!state.installing());
    }

    #[test]
    fn a_failure_releases_the_claim() {
        let state = UpdateState::default();
        state.inner.lock().unwrap().installing = true;
        assert!(state.installing());
        state.failed("0.2.1", false);
        assert!(!state.installing());
        assert_eq!(state.inner.lock().unwrap().refused, None);
        state.failed("0.2.1", true);
        assert_eq!(state.inner.lock().unwrap().refused.as_deref(), Some("0.2.1"));
    }

    #[test]
    fn nothing_new_is_no_offer() {
        assert_eq!(UpdateState::default().found(None), None);
    }
}
