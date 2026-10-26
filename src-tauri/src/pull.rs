//! The app's counterpart of `media pull` (data contract 8): the manifests pass (F007/S03) and the
//! previews pass (S04). Built on the sync engine in `sync.rs`.

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

// ── Previews pass (F007/S04) ────────────────────────────────────────────────────────────────

/// At least this much must stay free after the previews pass (data contract 8).
pub const RESERVE: u64 = 20_000_000_000;
/// A counted plan is used for at most this long; after that, count again.
pub const PLAN_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(30 * 60);

/// A preview (`<original>.jpg`) or a thumbnail (`<original>.t.jpg`), at any depth.
pub fn is_preview(rel: &str) -> bool {
    rel.ends_with(".jpg")
}

/// The previews job: the chosen `Previews` mount to the local previews folder.
pub fn previews_job(settings: &Settings, found: &NasMounts, local: &Path) -> Result<Job, String> {
    let mount = NasMounts::chosen(&found.previews)
        .ok_or_else(|| format!("gedeelde map '{}' is niet verbonden", settings.shares.previews))?;
    Ok(Job { remote: mount.mount_point.clone().into(), local: local.to_path_buf(), filter: is_preview, only: None, max_depth: None })
}

/// Free bytes for an unprivileged user on the volume that holds `path` (or its nearest existing
/// parent, since the previews folder may not exist yet).
pub fn free_bytes(path: &Path) -> Option<u64> {
    let mut p = path;
    while !p.exists() {
        p = p.parent()?;
    }
    statvfs_free(p)
}

#[cfg(target_os = "macos")]
fn statvfs_free(p: &Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    let c = std::ffi::CString::new(p.as_os_str().as_bytes()).ok()?;
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: `c` is a valid NUL-terminated path and `st` a writable statvfs record.
    (unsafe { libc::statvfs(c.as_ptr(), &mut st) } == 0).then(|| st.f_bavail as u64 * st.f_frsize as u64)
}

#[cfg(not(target_os = "macos"))]
fn statvfs_free(_: &Path) -> Option<u64> {
    None
}

/// "29,1" or "20" (a whole number without ",0").
fn gb(bytes: u64) -> String {
    let s = format!("{:.1}", bytes as f64 / 1e9);
    s.strip_suffix(".0").unwrap_or(&s).replace('.', ",")
}

/// Fits when `copy_bytes + 20 GB <= free`; otherwise the three numbers and the shortfall.
pub fn space_check(copy_bytes: u64, free: Option<u64>) -> Result<(), String> {
    let Some(free) = free else { return Err("de vrije ruimte op deze schijf is niet te bepalen".into()) };
    if copy_bytes + RESERVE <= free {
        return Ok(());
    }
    let short = copy_bytes + RESERVE - free;
    Err(format!("nodig {} GB, vrij {} GB, reserve {} GB: {} GB tekort", gb(copy_bytes), gb(free), gb(RESERVE), gb(short)))
}

#[derive(Debug, Serialize)]
pub struct PreviewsSummary {
    pub copy_files: u64,
    pub copy_bytes: u64,
    pub kept: u64,
    pub skipped: Vec<String>,
    pub free: Option<u64>,
    /// The space problem, if it doesn't fit.
    pub problem: Option<String>,
    pub needs_confirmation: bool,
}

pub fn summarize(plan: &sync::Plan, free: Option<u64>, state: &config::AppState) -> PreviewsSummary {
    PreviewsSummary {
        copy_files: plan.copy_files,
        copy_bytes: plan.copy_bytes,
        kept: plan.kept,
        skipped: plan.skipped.clone(),
        free,
        problem: if plan.copy_files == 0 { None } else { space_check(plan.copy_bytes, free).err() },
        needs_confirmation: plan.copy_files > 0 && state.previews_confirmed_at.is_none(),
    }
}

/// A plan counted at `at`, still usable at `now`?
pub fn plan_is_fresh(at: std::time::Instant, now: std::time::Instant) -> bool {
    now.saturating_duration_since(at) <= PLAN_MAX_AGE
}

#[derive(Debug, Default, Serialize)]
pub struct PreviewsOutcome {
    pub copied: u64,
    pub bytes: u64,
    pub kept: u64,
    pub skipped: Vec<String>,
    pub cancelled: bool,
    pub error: Option<String>,
}

/// Copies a counted plan. Refused without confirmation the first time, or when it doesn't fit
/// (free space measured again now). A given confirmation is recorded before copying starts.
pub fn run_previews(
    job: &Job,
    plan: &sync::Plan,
    confirmed: bool,
    config_dir: &Path,
    free_now: Option<u64>,
    progress: &mut dyn FnMut(&Progress),
    cancel: &AtomicBool,
) -> Result<PreviewsOutcome, String> {
    let mut state = config::load_state(config_dir);
    if plan.copy_files > 0 {
        if state.previews_confirmed_at.is_none() && !confirmed {
            return Err("bevestiging nodig".into());
        }
        space_check(plan.copy_bytes, free_now)?;
        if state.previews_confirmed_at.is_none() {
            state.previews_confirmed_at = Some(crate::nas::now_local());
            config::save_state(config_dir, &state)?;
        }
    }
    let r = sync::execute(job, plan, progress, cancel);
    Ok(PreviewsOutcome {
        copied: r.copied_missing + r.copied_newer + r.copied_changed,
        bytes: r.bytes,
        kept: r.kept,
        skipped: r.skipped,
        cancelled: r.cancelled,
        error: r.error,
    })
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

    // ── previews ──

    #[test]
    fn which_paths_are_previews() {
        assert!(is_preview("2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3.jpg"));
        assert!(is_preview("2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3.t.jpg"));
        for no in ["2026/07/iPhone-photos/IMG_1.JPG", "2026/07/x.png", "2026/07/.manifest.json"] {
            assert!(!is_preview(no), "{no}");
        }
    }

    #[test]
    fn space_fits_at_the_limit_and_not_one_byte_more() {
        assert!(space_check(1_000, Some(RESERVE + 1_000)).is_ok());
        assert!(space_check(1_000, Some(RESERVE + 999)).is_err());
        let msg = space_check(29_100_000_000, Some(35_200_000_000)).unwrap_err();
        assert_eq!(msg, "nodig 29,1 GB, vrij 35,2 GB, reserve 20 GB: 13,9 GB tekort");
        assert!(space_check(1, None).unwrap_err().contains("niet te bepalen"));
    }

    fn previews_plan(r: &Path, l: &Path) -> (Job, sync::Plan) {
        let j = Job { remote: r.into(), local: l.into(), filter: is_preview, only: None, max_depth: None };
        let p = sync::plan(&j, &mut |_| {}, &AtomicBool::new(false)).unwrap();
        (j, p)
    }

    #[test]
    fn confirmation_the_first_time_only() {
        let (r, l, cfg) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        put(r.path(), "2023/01/a.JPG.jpg", "a");
        put(r.path(), "2023/01/a.JPG.t.jpg", "t");
        let (j, p) = previews_plan(r.path(), l.path());
        let free = Some(RESERVE * 2);
        let st = config::load_state(cfg.path());
        assert!(summarize(&p, free, &st).needs_confirmation);
        let no = AtomicBool::new(false);
        assert_eq!(run_previews(&j, &p, false, cfg.path(), free, &mut |_| {}, &no).unwrap_err(), "bevestiging nodig");
        assert!(!l.path().join("2023").exists(), "nothing copied without confirmation");
        let out = run_previews(&j, &p, true, cfg.path(), free, &mut |_| {}, &no).unwrap();
        assert_eq!((out.copied, out.bytes), (2, 2));
        let st = config::load_state(cfg.path());
        assert!(st.previews_confirmed_at.is_some(), "confirmation recorded");
        put(r.path(), "2023/02/b.JPG.jpg", "b");
        let (j2, p2) = previews_plan(r.path(), l.path());
        assert!(!summarize(&p2, free, &st).needs_confirmation, "a second run doesn't ask");
        assert_eq!(run_previews(&j2, &p2, false, cfg.path(), free, &mut |_| {}, &no).unwrap().copied, 1);
        let (_, p3) = previews_plan(r.path(), l.path());
        let fresh_state = config::AppState::default();
        assert!(!summarize(&p3, free, &fresh_state).needs_confirmation, "nothing to copy needs no confirmation");
    }

    #[test]
    fn refused_when_it_does_not_fit_and_nothing_is_copied() {
        let (r, l, cfg) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        put(r.path(), "2023/01/a.JPG.jpg", "abcd");
        let (j, p) = previews_plan(r.path(), l.path());
        let tight = Some(RESERVE + 3);
        assert!(summarize(&p, tight, &config::AppState::default()).problem.unwrap().contains("tekort"));
        let err = run_previews(&j, &p, true, cfg.path(), tight, &mut |_| {}, &AtomicBool::new(false)).unwrap_err();
        assert!(err.contains("tekort"), "{err}");
        assert!(!l.path().join("2023").exists());
        assert_eq!(config::load_state(cfg.path()), config::AppState::default(), "no confirmation recorded on a refusal");
    }

    #[test]
    fn a_counted_plan_expires_after_30_minutes() {
        let at = std::time::Instant::now();
        assert!(plan_is_fresh(at, at + std::time::Duration::from_secs(29 * 60)));
        assert!(!plan_is_fresh(at, at + std::time::Duration::from_secs(31 * 60)));
    }

    #[test]
    fn free_bytes_walks_up_to_an_existing_folder() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nog/niet/aangemaakt");
        if cfg!(target_os = "macos") {
            assert!(free_bytes(&missing).unwrap() > 0);
        }
    }

    #[test]
    fn no_previews_mount_means_no_run() {
        let err = previews_job(&Settings::default(), &NasMounts::default(), Path::new("/tmp/x")).unwrap_err();
        assert_eq!(err, "gedeelde map 'Previews' is niet verbonden");
    }
}
