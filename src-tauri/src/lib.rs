//! Tauri-kern: leest de lokale index en previews van de CLI (datacontract sectie 4 en 7),
//! serveert previews via het `preview://`-protocol en opent originelen.

mod config;
mod connect;
mod db;
mod mounts;
mod nas;
mod pull;
mod runner;
mod sync;

use config::{Device, Settings};
use db::{Db, Item, MonthSummary};
use percent_encoding::percent_decode_str;
use std::path::{Component, Path, PathBuf};
use tauri::http::{header, Response, StatusCode};
use tauri::{Emitter, State};

/// Waar de CLI haar spullen zet (datacontract 7). Overschrijfbaar met MEDIA_CONFIG, MEDIA_PREVIEWS, MEDIA_LIBRARY.
pub struct Paths {
    pub config: PathBuf,
    pub previews: PathBuf,
    pub library: PathBuf,
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_default()
}

fn env_path(name: &str, default: PathBuf) -> PathBuf {
    std::env::var_os(name).map(PathBuf::from).unwrap_or(default)
}

pub fn paths() -> Paths {
    let h = home();
    Paths {
        config: env_path("MEDIA_CONFIG", h.join(".config").join("media")),
        previews: env_path("MEDIA_PREVIEWS", h.join("Pictures").join("Previews")),
        library: env_path("MEDIA_LIBRARY", h.join("Pictures").join("LibraryLocal")),
    }
}

impl Paths {
    fn index(&self) -> PathBuf {
        self.config.join("index.sqlite")
    }
}

#[derive(serde::Serialize)]
pub struct Status {
    pub index: String,
    pub index_exists: bool,
    pub previews: String,
    pub previews_exists: bool,
    pub library: String,
    pub library_exists: bool,
}

#[tauri::command]
fn status(paths: State<Paths>) -> Status {
    Status {
        index: paths.index().display().to_string(),
        index_exists: paths.index().is_file(),
        previews: paths.previews.display().to_string(),
        previews_exists: paths.previews.is_dir(),
        library: paths.library.display().to_string(),
        library_exists: paths.library.is_dir(),
    }
}

#[derive(serde::Serialize)]
pub struct SettingsState {
    pub settings: Settings,
    pub error: Option<String>,
    pub path: String,
}

#[tauri::command]
fn settings(paths: State<Paths>) -> SettingsState {
    let (settings, error) = config::load_settings(&paths.config);
    SettingsState { settings, error, path: paths.config.join(config::SETTINGS_FILE).display().to_string() }
}

#[tauri::command]
fn save_settings(settings: Settings, paths: State<Paths>) -> Result<(), String> {
    config::save_settings(&paths.config, &settings)
}

#[derive(serde::Serialize)]
pub struct DeviceState {
    pub device: Option<Device>,
    pub error: Option<String>,
    pub path: String,
}

#[tauri::command]
fn device(paths: State<Paths>) -> DeviceState {
    let path = paths.config.join(config::DEVICE_FILE).display().to_string();
    match config::load_device(&paths.config) {
        Ok(device) => DeviceState { device, error: None, path },
        Err(e) => DeviceState { device: None, error: Some(e), path },
    }
}

#[tauri::command]
fn create_device(name: String, owner: String, paths: State<Paths>) -> Result<Device, String> {
    config::create_device(&paths.config, &name, &owner)
}

#[tauri::command]
fn nas_mounts(paths: State<Paths>) -> mounts::NasMounts {
    let (settings, _) = config::load_settings(&paths.config);
    mounts::find_shares(&mounts::system_mounts(), &settings)
}

#[derive(serde::Serialize)]
pub struct ConnectResult {
    pub attempts: Vec<connect::Attempt>,
    pub mounts: mounts::NasMounts,
}

/// Connects the shares that aren't mounted. Blocks while the macOS sign-in dialog is open, so it
/// runs on a blocking thread.
#[tauri::command]
async fn connect_nas(allow_ui: bool, paths: State<'_, Paths>) -> Result<ConnectResult, String> {
    let config = paths.config.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (settings, _) = config::load_settings(&config);
        let before = mounts::find_shares(&mounts::system_mounts(), &settings);
        let todo = connect::to_connect(&before, &settings);
        #[cfg(target_os = "macos")]
        let attempts = connect::connect_all(&connect::NetFsMounter { allow_ui }, &settings.nas_host, &todo);
        #[cfg(not(target_os = "macos"))]
        let _ = allow_ui;
        #[cfg(not(target_os = "macos"))]
        let attempts = if todo.is_empty() {
            Vec::new()
        } else {
            return Err("verbinden met de NAS kan nog alleen op macOS".to_string());
        };
        Ok(ConnectResult { attempts, mounts: mounts::find_shares(&mounts::system_mounts(), &settings) })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(serde::Serialize)]
pub struct NasCheck {
    pub library: nas::Access,
    pub previews: nas::Access,
    pub lists: nas::Access,
    pub exports: nas::Access,
    pub registration: nas::Registration,
}

/// Access per connected share, then registering this device once `Lists` is writable. On a
/// blocking thread, because a slow NAS can take a while to answer.
#[tauri::command]
async fn nas_check(paths: State<'_, Paths>) -> Result<NasCheck, String> {
    let config = paths.config.clone();
    tauri::async_runtime::spawn_blocking(move || check_nas(&config)).await.map_err(|e| e.to_string())
}

pub fn check_nas(config: &Path) -> NasCheck {
    let (settings, _) = config::load_settings(config);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    let device = config::load_device(config).ok().flatten();
    let check = |list: &[mounts::Mount], write: bool| match mounts::NasMounts::chosen(list) {
        None => nas::Access::default(),
        Some(m) => nas::check_share(
            Path::new(&m.mount_point),
            write.then(|| device.as_ref().map_or("nodevice", |d| d.name.as_str())),
        ),
    };
    let lists = check(&found.lists, true);
    let registration = match (&device, lists.writable) {
        (None, _) => nas::Registration::NotTried("dit toestel heeft nog geen naam".into()),
        (_, None) => nas::Registration::NotTried("Lijsten is niet verbonden".into()),
        (_, Some(false)) => nas::Registration::NotTried("Lijsten is niet schrijfbaar".into()),
        (Some(d), Some(true)) => {
            let dir = mounts::NasMounts::chosen(&found.lists).map(|m| m.mount_point.clone()).unwrap_or_default();
            nas::register(Path::new(&dir), &nas::entry_for(d), &|| {})
        }
    };
    NasCheck {
        library: check(&found.library, false),
        previews: check(&found.previews, false),
        exports: check(&found.exports, false),
        lists,
        registration,
    }
}

/// Sends the runner's progress to the window as Tauri events (F007/S05).
struct EventSink(tauri::AppHandle);

impl runner::Sink for EventSink {
    fn progress(&self, state: &runner::SyncState) {
        let _ = self.0.emit("sync-progress", state);
    }
    fn done(&self, state: &runner::SyncState) {
        let _ = self.0.emit("sync-done", state);
    }
}

/// The two sync jobs for the current settings and mounts.
fn sync_jobs(paths: &Paths) -> runner::Jobs {
    let (settings, _) = config::load_settings(&paths.config);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    runner::Jobs {
        manifests: pull::manifests_job(&settings, &found, &paths.library),
        previews: pull::previews_job(&settings, &found, &paths.previews),
        config_dir: paths.config.clone(),
        free: pull::free_bytes,
    }
}

/// Start a sync in the background: `kind` is "all" (manifests, then previews) or "previews"
/// (after the confirmation dialog). Progress arrives as `sync-progress`, the end as `sync-done`.
#[tauri::command]
fn sync_start(kind: String, confirmed: bool, app: tauri::AppHandle, paths: State<Paths>, runner: State<runner::SyncRunner>) -> Result<(), String> {
    let kind = match kind.as_str() {
        "all" => runner::Kind::All,
        "previews" => runner::Kind::Previews,
        other => return Err(format!("onbekende synchronisatie: {other}")),
    };
    runner.start(kind, confirmed, sync_jobs(&paths), std::sync::Arc::new(EventSink(app))).map(|_| ())
}

#[tauri::command]
fn sync_cancel(runner: State<runner::SyncRunner>) {
    runner.cancel();
}

#[tauri::command]
fn sync_status(runner: State<runner::SyncRunner>) -> runner::SyncState {
    runner.status()
}

#[tauri::command]
fn months(paths: State<Paths>) -> Result<Vec<MonthSummary>, String> {
    Db::open(&paths.index())?.months()
}

#[tauri::command]
fn items(month: String, paths: State<Paths>) -> Result<Vec<Item>, String> {
    Db::open(&paths.index())?.items(&month)
}

/// Een relatief contractpad (`2026/07/EOSR/.../X.CR3`) mag nooit uit de boom wijzen.
fn safe_join(root: &Path, rel: &str) -> Option<PathBuf> {
    let rel = Path::new(rel);
    if rel.is_absolute() || rel.components().any(|c| !matches!(c, Component::Normal(_))) {
        return None;
    }
    Some(root.join(rel))
}

fn original(paths: &Paths, rel: &str) -> Result<PathBuf, String> {
    let p = safe_join(&paths.library, rel).ok_or("ongeldig pad")?;
    if !p.is_file() {
        return Err("het origineel staat niet op deze laptop (alleen op de NAS)".into());
    }
    Ok(p)
}

/// Toon het origineel in de Finder / Verkenner.
#[tauri::command]
fn reveal(path: String, paths: State<Paths>) -> Result<(), String> {
    let p = original(&paths, &path)?;
    tauri_plugin_opener::reveal_item_in_dir(p).map_err(|e| e.to_string())
}

/// Open het origineel met het standaardprogramma (Voorvertoning, Lightroom, …).
#[tauri::command]
fn open_original(path: String, paths: State<Paths>) -> Result<(), String> {
    let p = original(&paths, &path)?;
    tauri_plugin_opener::open_path(p, None::<&str>).map_err(|e| e.to_string())
}

/// `preview://localhost/<pad>` → bestand onder de Previews-map. Het pad komt percent-gecodeerd binnen.
fn preview_response(previews: &Path, uri_path: &str) -> Response<Vec<u8>> {
    let decoded = percent_decode_str(uri_path.trim_start_matches('/')).decode_utf8_lossy();
    let file = safe_join(previews, &decoded).filter(|p| p.extension().is_some_and(|e| e == "jpg"));
    match file.and_then(|p| std::fs::read(p).ok()) {
        Some(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/jpeg")
            .header(header::CACHE_CONTROL, "max-age=86400")
            .body(bytes)
            .unwrap(),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(b"geen preview".to_vec())
            .unwrap(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let previews_root = paths().previews.clone();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(paths())
        .manage(runner::SyncRunner::default())
        .register_uri_scheme_protocol("preview", move |_ctx, request| {
            preview_response(&previews_root, request.uri().path())
        })
        .invoke_handler(tauri::generate_handler![
            status, months, items, reveal, open_original, settings, save_settings, device, create_device,
            nas_mounts, connect_nas, nas_check, sync_start, sync_cancel, sync_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_join_rejects_escapes() {
        let root = Path::new("/r");
        assert_eq!(safe_join(root, "2026/07/a.jpg"), Some(PathBuf::from("/r/2026/07/a.jpg")));
        assert_eq!(safe_join(root, "../etc/passwd"), None);
        assert_eq!(safe_join(root, "/etc/passwd"), None);
        assert_eq!(safe_join(root, "2026/../../x"), None);
    }

    #[test]
    fn preview_protocol_serves_jpg_and_refuses_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("2026/07")).unwrap();
        std::fs::write(dir.path().join("2026/07/IMG (1).JPG.t.jpg"), b"jpegdata").unwrap();
        std::fs::write(dir.path().join("geheim.txt"), b"nee").unwrap();
        let ok = preview_response(dir.path(), "/2026%2F07%2FIMG%20(1).JPG.t.jpg");
        assert_eq!(ok.status(), StatusCode::OK);
        assert_eq!(ok.body(), b"jpegdata");
        assert_eq!(ok.headers()[header::CONTENT_TYPE], "image/jpeg");
        assert_eq!(preview_response(dir.path(), "/2026/07/IMG%20(1).JPG.t.jpg").status(), StatusCode::OK);
        assert_eq!(preview_response(dir.path(), "/geheim.txt").status(), StatusCode::NOT_FOUND);
        assert_eq!(preview_response(dir.path(), "/..%2F..%2Fetc%2Fpasswd").status(), StatusCode::NOT_FOUND);
        assert_eq!(preview_response(dir.path(), "/2026/07/ontbreekt.jpg").status(), StatusCode::NOT_FOUND);
    }
}
