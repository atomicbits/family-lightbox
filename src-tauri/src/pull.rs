//! The app's counterpart of `media pull` (data contract 8): the manifests pass (F007/S03), and
//! later the previews pass (S04). Built on the sync engine in `sync.rs`.

use crate::config::{self, Settings};
use crate::mounts::{self, NasMounts};
use crate::sync::{self, Job, Progress};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicBool;

pub const MANIFEST_NAME: &str = ".manifest.json";

/// `<year>/<subfolder>/.manifest.json` with a four-digit year (`0000` included).
pub fn is_manifest(rel: &str) -> bool {
    let parts: Vec<&str> = rel.split('/').collect();
    parts.len() == 3
        && parts[0].len() == 4
        && parts[0].bytes().all(|b| b.is_ascii_digit())
        && !parts[1].is_empty()
        && parts[2] == MANIFEST_NAME
}

/// The manifests job: the chosen `Library` mount to the working copy, two levels deep.
pub fn manifests_job(settings: &Settings, found: &NasMounts, working_copy: &Path) -> Result<Job, String> {
    let mount = NasMounts::chosen(&found.library)
        .ok_or_else(|| format!("gedeelde map '{}' is niet verbonden", settings.shares.library))?;
    Ok(Job {
        remote: mount.mount_point.clone().into(),
        local: working_copy.to_path_buf(),
        filter: is_manifest,
        only: None,
        max_depth: Some(2),
    })
}

#[derive(Debug, Default, Serialize)]
pub struct ManifestsOutcome {
    pub copied: u64,
    pub kept: u64,
    /// Year folders the account may not read.
    pub skipped: Vec<String>,
    /// Copied manifests that aren't valid JSON with a `files` list; left in place.
    pub invalid: Vec<String>,
    /// Year subfolders whose manifest was copied in this run, for the index (feature 8).
    pub changed: Vec<String>,
    pub cancelled: bool,
    pub error: Option<String>,
}

fn valid_manifest(path: &Path) -> bool {
    fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
        .is_some_and(|v| v.get("files").is_some_and(|f| f.is_array()))
}

/// Plan, copy, then check every manifest copied in this run.
pub fn run_manifests(job: &Job, progress: &mut dyn FnMut(&Progress), cancel: &AtomicBool) -> ManifestsOutcome {
    let mut out = ManifestsOutcome::default();
    if let Err(e) = fs::read_dir(&job.remote) {
        out.error = Some(format!("{} is niet leesbaar: {e}", job.remote.display()));
        return out;
    }
    let plan = match sync::plan(job, progress, cancel) {
        Ok(p) => p,
        Err(sync::Stop::Cancelled) => {
            out.cancelled = true;
            return out;
        }
        Err(sync::Stop::Failed(e)) => {
            out.error = Some(e);
            return out;
        }
    };
    let result = sync::execute(job, &plan, progress, cancel);
    out.kept = result.kept;
    out.skipped = result.skipped;
    out.cancelled = result.cancelled;
    out.error = result.error;
    // What actually arrived: a planned copy whose local file now matches the remote one.
    for e in plan.entries.iter().filter(|e| e.decision.copies()) {
        let local = job.local.join(&e.rel);
        let now = fs::metadata(&local).ok().and_then(|m| Some((m.len(), m.modified().ok()?)));
        if sync::decide(e.size, e.mtime, now) != sync::Decision::KeepSame {
            continue;
        }
        out.copied += 1;
        if valid_manifest(&local) {
            out.changed.push(e.rel.trim_end_matches(&format!("/{MANIFEST_NAME}")).to_string());
        } else {
            out.invalid.push(e.rel.clone());
        }
    }
    out
}

/// The manifests pass for the app: settings and mounts from feature 6, into the working copy.
pub fn sync_manifests(config_dir: &Path, working_copy: &Path, progress: &mut dyn FnMut(&Progress), cancel: &AtomicBool) -> ManifestsOutcome {
    let (settings, _) = config::load_settings(config_dir);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    match manifests_job(&settings, &found, working_copy) {
        Ok(job) => run_manifests(&job, progress, cancel),
        Err(e) => ManifestsOutcome { error: Some(e), ..ManifestsOutcome::default() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    fn put(root: &Path, rel: &str, body: &str) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, body).unwrap();
        let t = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        fs::File::options().write(true).open(&p).unwrap().set_modified(t).unwrap();
    }

    fn job(r: &Path, l: &Path) -> Job {
        Job { remote: r.into(), local: l.into(), filter: is_manifest, only: None, max_depth: Some(2) }
    }

    const OK: &str = r#"{"contract_version": "1.2", "folder": "2026/07", "files": []}"#;

    #[test]
    fn which_paths_are_manifests() {
        for yes in ["2026/07/.manifest.json", "0000/zonder-datum/.manifest.json", "2012/scans Peter/.manifest.json"] {
            assert!(is_manifest(yes), "{yes}");
        }
        for no in [".manifest.json", "2026/.manifest.json", "2026/07/EOSR/.manifest.json", "2026/07/x.json", "26/07/.manifest.json", "abcd/07/.manifest.json"] {
            assert!(!is_manifest(no), "{no}");
        }
    }

    #[test]
    fn copies_manifests_only_and_checks_them() {
        let (r, l) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        put(r.path(), "2026/07/.manifest.json", OK);
        put(r.path(), "2012/scans Peter/.manifest.json", OK);
        put(r.path(), "2025/01/.manifest.json", "{ kapot");
        put(r.path(), "2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3", "raw");
        put(r.path(), "2026/07/EOSR/.manifest.json", OK);
        let out = run_manifests(&job(r.path(), l.path()), &mut |_| {}, &AtomicBool::new(false));
        assert_eq!(out.copied, 3);
        assert_eq!(out.changed, vec!["2012/scans Peter", "2026/07"]);
        assert_eq!(out.invalid, vec!["2025/01/.manifest.json"]);
        assert!(l.path().join("2025/01/.manifest.json").exists(), "an invalid manifest stays in place");
        assert!(!l.path().join("2026/07/EOSR").exists(), "nothing below the year subfolder");
        let again = run_manifests(&job(r.path(), l.path()), &mut |_| {}, &AtomicBool::new(false));
        assert_eq!((again.copied, again.kept), (0, 3));
        assert!(again.changed.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn an_unreadable_year_is_skipped() {
        use std::os::unix::fs::PermissionsExt;
        let (r, l) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        put(r.path(), "2007/01/.manifest.json", OK);
        put(r.path(), "2008/01/.manifest.json", OK);
        fs::set_permissions(r.path().join("2007"), fs::Permissions::from_mode(0o000)).unwrap();
        let out = run_manifests(&job(r.path(), l.path()), &mut |_| {}, &AtomicBool::new(false));
        fs::set_permissions(r.path().join("2007"), fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(out.skipped, vec!["2007"]);
        assert_eq!(out.changed, vec!["2008/01"]);
        assert!(out.error.is_none());
    }

    #[test]
    fn no_library_mount_means_no_run() {
        let err = manifests_job(&Settings::default(), &NasMounts::default(), Path::new("/tmp/x")).unwrap_err();
        assert_eq!(err, "gedeelde map 'Library' is niet verbonden");
        let out = run_manifests(&job(Path::new("/bestaat/niet"), Path::new("/tmp/x")), &mut |_| {}, &AtomicBool::new(false));
        assert!(out.error.unwrap().contains("niet leesbaar"));
    }
}
