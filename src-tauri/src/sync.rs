//! Bringing a folder tree from a NAS mount over to a local folder, like `rsync --update` in
//! `media pull` (data contract 8; F007/S02). Copies what is missing locally or newer on the
//! NAS; never deletes a local file other than its own temporary files, never writes to the NAS.

use std::fs::{self, File};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BLOCK: usize = 1 << 20;
const PARTIAL_SUFFIX: &str = ".flb.partial";
const STALE: Duration = Duration::from_secs(3600);

/// Names every component ignores (data contract section 1), as files and as folders.
pub fn excluded(name: &str) -> bool {
    matches!(
        name,
        "@eaDir" | "#recycle" | "#snapshot" | ".DS_Store" | "Thumbs.db" | ".TemporaryItems" | ".Spotlight-V100" | ".fseventsd"
    ) || name.starts_with("._")
        || name.ends_with(".tmp")
        || name.ends_with(".partial")
}

#[derive(Debug)]
pub struct Job {
    pub remote: PathBuf,
    pub local: PathBuf,
    /// Which files take part, by relative path with `/` (for example "ends in .jpg").
    pub filter: fn(&str) -> bool,
    /// Only these top-level folders, if given (for example a few years).
    pub only: Option<Vec<String>>,
    /// How many folder levels below the root are walked (`Some(2)`: years and their subfolders).
    pub max_depth: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    CopyMissing,
    CopyNewer,
    CopyChanged,
    KeepSame,
    KeepLocalNewer,
}

impl Decision {
    pub fn copies(self) -> bool {
        matches!(self, Decision::CopyMissing | Decision::CopyNewer | Decision::CopyChanged)
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub rel: String,
    pub decision: Decision,
    pub size: u64,
    pub mtime: SystemTime,
}

#[derive(Debug, Default)]
pub struct Plan {
    pub entries: Vec<Entry>,
    pub copy_files: u64,
    pub copy_bytes: u64,
    pub kept: u64,
    /// Folders the account may not read, relative to the remote root.
    pub skipped: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Planning,
    Copying,
}

#[allow(dead_code)] // read by the progress line in the UI from F007/S05 on
pub struct Progress<'a> {
    pub phase: Phase,
    pub folder: &'a str,
    pub files_done: u64,
    pub files_total: u64,
    pub bytes_done: u64,
    pub bytes_total: u64,
}

#[derive(Debug, PartialEq)]
pub enum Stop {
    Cancelled,
    Failed(String),
}

#[derive(Debug, Default, PartialEq)]
pub struct Outcome {
    pub copied_missing: u64,
    pub copied_newer: u64,
    pub copied_changed: u64,
    pub bytes: u64,
    pub kept: u64,
    pub skipped: Vec<String>,
    pub cancelled: bool,
    pub error: Option<String>,
}

fn secs(t: SystemTime) -> i64 {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => -(e.duration().as_secs() as i64) - 1,
    }
}

/// Compared in whole seconds: SMB, AFP and APFS round modification times differently.
pub fn decide(remote_size: u64, remote_mtime: SystemTime, local: Option<(u64, SystemTime)>) -> Decision {
    let Some((local_size, local_mtime)) = local else { return Decision::CopyMissing };
    let (r, l) = (secs(remote_mtime), secs(local_mtime));
    if r > l {
        Decision::CopyNewer
    } else if r < l {
        Decision::KeepLocalNewer
    } else if remote_size != local_size {
        Decision::CopyChanged
    } else {
        Decision::KeepSame
    }
}

fn partial_name(name: &str) -> String {
    format!(".{name}{PARTIAL_SUFFIX}")
}

/// Removes this engine's temporary files older than an hour in one local folder.
fn clear_stale(local_dir: &Path) {
    let Ok(entries) = fs::read_dir(local_dir) else { return };
    let cutoff = SystemTime::now() - STALE;
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') && name.ends_with(PARTIAL_SUFFIX) {
            let old = e.metadata().and_then(|m| m.modified()).is_ok_and(|t| t < cutoff);
            if old {
                let _ = fs::remove_file(e.path());
            }
        }
    }
}

fn join_rel(prefix: &str, name: &str) -> String {
    if prefix.is_empty() { name.to_string() } else { format!("{prefix}/{name}") }
}

/// Walks the remote tree and decides per file what to do. Nothing is copied yet.
pub fn plan(job: &Job, progress: &mut dyn FnMut(&Progress), cancel: &AtomicBool) -> Result<Plan, Stop> {
    let mut p = Plan::default();
    let top: Vec<String> = match &job.only {
        Some(list) => list.clone(),
        None => {
            let entries = fs::read_dir(&job.remote).map_err(|e| Stop::Failed(format!("{}: {e}", job.remote.display())))?;
            let mut names: Vec<String> = entries.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
            names.sort();
            names
        }
    };
    clear_stale(&job.local);
    for name in top {
        if excluded(&name) {
            continue;
        }
        let path = job.remote.join(&name);
        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            // A requested top-level folder that doesn't exist (a year without files) is fine;
            // one that was just listed and is gone now means the NAS went away.
            Err(e) if e.kind() == ErrorKind::NotFound && job.only.is_some() => continue,
            Err(e) => return Err(Stop::Failed(format!("{}: {e}", path.display()))),
        };
        if meta.is_dir() {
            if within_depth(job, 1) {
                walk(job, &path, &name, &mut p, progress, cancel)?;
            }
        } else if meta.is_file() {
            consider(job, &name, &meta, &mut p, progress, cancel)?;
        }
    }
    Ok(p)
}

/// Whether a folder at `level` below the root (1 = a direct child) is still walked.
fn within_depth(job: &Job, level: usize) -> bool {
    job.max_depth.is_none_or(|d| level <= d)
}

fn walk(job: &Job, dir: &Path, rel: &str, p: &mut Plan, progress: &mut dyn FnMut(&Progress), cancel: &AtomicBool) -> Result<(), Stop> {
    let entries = match fs::read_dir(dir) {
        Ok(it) => it,
        Err(e) if e.kind() == ErrorKind::PermissionDenied => {
            p.skipped.push(rel.to_string());
            return Ok(());
        }
        Err(e) => return Err(Stop::Failed(format!("{}: {e}", dir.display()))),
    };
    clear_stale(&job.local.join(rel));
    let mut list = Vec::new();
    for e in entries {
        let e = e.map_err(|e| Stop::Failed(format!("{}: {e}", dir.display())))?;
        list.push(e);
    }
    list.sort_by_key(|e| e.file_name());
    for e in list {
        let name = e.file_name().to_string_lossy().into_owned();
        if excluded(&name) {
            continue;
        }
        let child = join_rel(rel, &name);
        let ft = e.file_type().map_err(|err| Stop::Failed(format!("{}: {err}", e.path().display())))?;
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            if within_depth(job, child.split('/').count()) {
                walk(job, &e.path(), &child, p, progress, cancel)?;
            }
        } else if ft.is_file() {
            let meta = e.metadata().map_err(|err| Stop::Failed(format!("{}: {err}", e.path().display())))?;
            consider(job, &child, &meta, p, progress, cancel)?;
        }
    }
    Ok(())
}

fn consider(job: &Job, rel: &str, meta: &fs::Metadata, p: &mut Plan, progress: &mut dyn FnMut(&Progress), cancel: &AtomicBool) -> Result<(), Stop> {
    if cancel.load(Ordering::Relaxed) {
        return Err(Stop::Cancelled);
    }
    if !(job.filter)(rel) {
        return Ok(());
    }
    let mtime = meta.modified().map_err(|e| Stop::Failed(format!("{rel}: {e}")))?;
    let local = fs::symlink_metadata(job.local.join(rel)).ok().filter(|m| m.is_file()).and_then(|m| Some((m.len(), m.modified().ok()?)));
    let decision = decide(meta.len(), mtime, local);
    if decision.copies() {
        p.copy_files += 1;
        p.copy_bytes += meta.len();
    } else {
        p.kept += 1;
    }
    p.entries.push(Entry { rel: rel.to_string(), decision, size: meta.len(), mtime });
    let folder = rel.rsplit_once('/').map_or("", |(f, _)| f);
    progress(&Progress { phase: Phase::Planning, folder, files_done: p.entries.len() as u64, files_total: 0, bytes_done: 0, bytes_total: 0 });
    Ok(())
}

/// Copies what the plan says to copy, one file at a time, atomically.
pub fn execute(job: &Job, plan: &Plan, progress: &mut dyn FnMut(&Progress), cancel: &AtomicBool) -> Outcome {
    let mut folders = FolderTimes::default();
    let out = execute_inner(job, plan, progress, cancel, &mut folders);
    folders.restore(job);
    out
}

/// Writing a file changes its folder's modification time, and the CLI uses folder times to see
/// which months changed (`library_months`). Like `rsync -a` in `media pull`, folders end with the
/// time they had: an existing folder gets its old time back, a new one gets the NAS folder's time.
#[derive(Default)]
struct FolderTimes {
    /// Relative folder → its local time before the first copy into it (None: it didn't exist).
    before: std::collections::BTreeMap<String, Option<SystemTime>>,
}

impl FolderTimes {
    fn note(&mut self, job: &Job, rel_file: &str) {
        let mut rel = rel_file;
        while let Some((parent, _)) = rel.rsplit_once('/') {
            if !self.before.contains_key(parent) {
                let t = fs::metadata(job.local.join(parent)).and_then(|m| m.modified()).ok();
                self.before.insert(parent.to_string(), t);
            }
            rel = parent;
        }
    }

    fn restore(&self, job: &Job) {
        for (rel, before) in &self.before {
            let time = match before {
                Some(t) => Some(*t),
                None => fs::metadata(job.remote.join(rel)).and_then(|m| m.modified()).ok(),
            };
            if let Some(t) = time {
                let _ = File::open(job.local.join(rel)).and_then(|d| d.set_modified(t));
            }
        }
    }
}

fn execute_inner(job: &Job, plan: &Plan, progress: &mut dyn FnMut(&Progress), cancel: &AtomicBool, folders: &mut FolderTimes) -> Outcome {
    let mut out = Outcome { kept: plan.kept, skipped: plan.skipped.clone(), ..Outcome::default() };
    let (mut files_done, mut bytes_done) = (0u64, 0u64);
    for e in plan.entries.iter().filter(|e| e.decision.copies()) {
        if cancel.load(Ordering::Relaxed) {
            out.cancelled = true;
            return out;
        }
        let folder = e.rel.rsplit_once('/').map_or("", |(f, _)| f);
        let mut report = |bytes_done: u64| {
            progress(&Progress {
                phase: Phase::Copying,
                folder,
                files_done,
                files_total: plan.copy_files,
                bytes_done,
                bytes_total: plan.copy_bytes,
            })
        };
        folders.note(job, &e.rel);
        match copy_one(job, e, bytes_done, &mut report, cancel) {
            Ok(n) => {
                bytes_done += n;
                files_done += 1;
                out.bytes += n;
                match e.decision {
                    Decision::CopyMissing => out.copied_missing += 1,
                    Decision::CopyNewer => out.copied_newer += 1,
                    Decision::CopyChanged => out.copied_changed += 1,
                    _ => {}
                }
            }
            Err(Stop::Cancelled) => {
                out.cancelled = true;
                return out;
            }
            Err(Stop::Failed(msg)) => {
                out.error = Some(msg);
                return out;
            }
        }
    }
    out
}

fn copy_one(job: &Job, e: &Entry, bytes_before: u64, report: &mut dyn FnMut(u64), cancel: &AtomicBool) -> Result<u64, Stop> {
    let src = job.remote.join(&e.rel);
    let target = job.local.join(&e.rel);
    let dir = target.parent().ok_or_else(|| Stop::Failed(format!("{}: geen map", target.display())))?;
    let name = target.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let tmp = dir.join(partial_name(&name));
    let fail = |msg: String| {
        let _ = fs::remove_file(&tmp);
        Stop::Failed(msg)
    };
    fs::create_dir_all(dir).map_err(|err| Stop::Failed(format!("{}: {err}", dir.display())))?;
    let mut input = File::open(&src).map_err(|err| Stop::Failed(format!("{}: {err}", src.display())))?;
    let mut output = File::create(&tmp).map_err(|err| fail(format!("{}: {err}", tmp.display())))?;
    let mut buf = vec![0u8; BLOCK];
    let mut copied = 0u64;
    loop {
        let n = input.read(&mut buf).map_err(|err| fail(format!("{}: {err}", src.display())))?;
        if n == 0 {
            break;
        }
        output.write_all(&buf[..n]).map_err(|err| fail(format!("{}: {err}", tmp.display())))?;
        copied += n as u64;
        report(bytes_before + copied);
        if cancel.load(Ordering::Relaxed) {
            drop(output);
            let _ = fs::remove_file(&tmp);
            return Err(Stop::Cancelled);
        }
    }
    output.set_modified(e.mtime).map_err(|err| fail(format!("{}: {err}", tmp.display())))?;
    output.sync_all().map_err(|err| fail(format!("{}: {err}", tmp.display())))?;
    drop(output);
    fs::rename(&tmp, &target).map_err(|err| fail(format!("{}: {err}", target.display())))?;
    Ok(copied)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jpg(rel: &str) -> bool {
        rel.ends_with(".jpg")
    }

    fn put(root: &Path, rel: &str, body: &[u8], mtime: SystemTime) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, body).unwrap();
        File::options().write(true).open(&p).unwrap().set_modified(mtime).unwrap();
    }

    fn t(secs: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_700_000_000 + secs)
    }

    fn job(remote: &Path, local: &Path) -> Job {
        Job { remote: remote.into(), local: local.into(), filter: jpg, only: None, max_depth: None }
    }

    fn run(j: &Job) -> (Plan, Outcome) {
        let cancel = AtomicBool::new(false);
        let p = plan(j, &mut |_| {}, &cancel).unwrap();
        let o = execute(j, &p, &mut |_| {}, &cancel);
        (p, o)
    }

    /// Every file under `root` with its content and modification time, for before/after checks.
    fn snapshot(root: &Path) -> Vec<(String, Vec<u8>, SystemTime)> {
        let mut out = Vec::new();
        fn go(dir: &Path, root: &Path, out: &mut Vec<(String, Vec<u8>, SystemTime)>) {
            let Ok(rd) = fs::read_dir(dir) else { return };
            for e in rd.flatten() {
                let p = e.path();
                let m = fs::symlink_metadata(&p).unwrap();
                if m.is_dir() {
                    go(&p, root, out);
                } else if m.is_file() {
                    out.push((p.strip_prefix(root).unwrap().display().to_string(), fs::read(&p).unwrap(), m.modified().unwrap()));
                }
            }
        }
        go(root, root, &mut out);
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    #[test]
    fn decisions_in_whole_seconds() {
        let now = t(100);
        assert_eq!(decide(5, now, None), Decision::CopyMissing);
        assert_eq!(decide(5, t(101), Some((5, now))), Decision::CopyNewer);
        assert_eq!(decide(5, now, Some((5, t(101)))), Decision::KeepLocalNewer);
        assert_eq!(decide(6, now, Some((5, now))), Decision::CopyChanged);
        assert_eq!(decide(5, now, Some((5, now))), Decision::KeepSame);
        assert_eq!(decide(5, now + Duration::from_millis(700), Some((5, now))), Decision::KeepSame, "sub-second difference");
    }

    #[test]
    fn copies_missing_newer_changed_and_keeps_the_rest() {
        let (r, l) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        put(r.path(), "2026/07/a.jpg", b"new", t(10));
        put(r.path(), "2026/07/b.jpg", b"newer", t(20));
        put(l.path(), "2026/07/b.jpg", b"old", t(10));
        put(r.path(), "2026/07/c.jpg", b"same", t(10));
        put(l.path(), "2026/07/c.jpg", b"same", t(10));
        put(r.path(), "2026/07/d.jpg", b"remote", t(10));
        put(l.path(), "2026/07/d.jpg", b"local is newer", t(30));
        put(r.path(), "2026/07/e.jpg", b"longer", t(10));
        put(l.path(), "2026/07/e.jpg", b"short", t(10));
        put(r.path(), "2026/07/notes.txt", b"filtered out", t(10));
        let (p, o) = run(&job(r.path(), l.path()));
        assert_eq!((p.copy_files, p.copy_bytes, p.kept), (3, 3 + 5 + 6, 2));
        assert_eq!((o.copied_missing, o.copied_newer, o.copied_changed, o.kept), (1, 1, 1, 2));
        assert_eq!(fs::read(l.path().join("2026/07/b.jpg")).unwrap(), b"newer");
        assert_eq!(fs::read(l.path().join("2026/07/d.jpg")).unwrap(), b"local is newer");
        assert_eq!(fs::read(l.path().join("2026/07/e.jpg")).unwrap(), b"longer");
        assert!(!l.path().join("2026/07/notes.txt").exists());
        let m = fs::metadata(l.path().join("2026/07/a.jpg")).unwrap();
        assert_eq!(secs(m.modified().unwrap()), secs(t(10)), "remote time kept");
        let again = run(&job(r.path(), l.path()));
        assert_eq!(again.0.copy_files, 0);
    }

    #[test]
    fn folder_times_stay_as_they_were_or_follow_the_nas() {
        let (r, l) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        put(r.path(), "2016/12/a.jpg", b"a", t(1));
        put(r.path(), "0000/zonder-datum/b.jpg", b"b", t(1));
        fs::create_dir_all(l.path().join("2016/12")).unwrap();
        let old = t(500);
        for d in ["2016/12", "2016"] {
            File::open(l.path().join(d)).unwrap().set_modified(old).unwrap();
        }
        let remote_new = t(900);
        for d in ["0000/zonder-datum", "0000"] {
            File::open(r.path().join(d)).unwrap().set_modified(remote_new).unwrap();
        }
        let (_, o) = run(&job(r.path(), l.path()));
        assert_eq!(o.copied_missing, 2);
        let local_time = |d: &str| secs(fs::metadata(l.path().join(d)).unwrap().modified().unwrap());
        assert_eq!(local_time("2016/12"), secs(old), "an existing folder keeps its time");
        assert_eq!(local_time("2016"), secs(old));
        assert_eq!(local_time("0000/zonder-datum"), secs(remote_new), "a new folder gets the NAS folder's time");
        assert_eq!(local_time("0000"), secs(remote_new));
    }

    #[cfg(unix)]
    #[test]
    fn exclusions_and_symlinks() {
        let (r, l) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        for rel in ["@eaDir/x.jpg", "2026/#snapshot/x.jpg", "2026/#recycle/x.jpg", "2026/._x.jpg", "2026/x.tmp", "2026/ok.jpg"] {
            put(r.path(), rel, b"x", t(1));
        }
        std::os::unix::fs::symlink(r.path().join("2026"), r.path().join("link")).unwrap();
        std::os::unix::fs::symlink(r.path().join("2026/ok.jpg"), r.path().join("2026/link.jpg")).unwrap();
        let (p, _) = run(&job(r.path(), l.path()));
        let rels: Vec<_> = p.entries.iter().map(|e| e.rel.as_str()).collect();
        assert_eq!(rels, vec!["2026/ok.jpg"]);
        assert!(excluded(".DS_Store") && excluded("Thumbs.db") && excluded("x.partial") && !excluded("x.jpg"));
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_folder_is_skipped_and_listed() {
        use std::os::unix::fs::PermissionsExt;
        let (r, l) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        put(r.path(), "2007/01/a.jpg", b"x", t(1));
        put(r.path(), "2008/01/b.jpg", b"y", t(1));
        let locked = r.path().join("2007");
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
        let (p, o) = run(&job(r.path(), l.path()));
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(p.skipped, vec!["2007"]);
        assert_eq!(o.copied_missing, 1);
        assert!(l.path().join("2008/01/b.jpg").exists() && !l.path().join("2007").exists());
    }

    #[cfg(unix)]
    #[test]
    fn remote_stays_untouched_even_read_only() {
        use std::os::unix::fs::PermissionsExt;
        let (r, l) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        put(r.path(), "2026/07/a.jpg", b"a", t(1));
        put(r.path(), "2026/08/b.jpg", b"b", t(2));
        let before = snapshot(r.path());
        for d in ["2026/07", "2026/08", "2026", ""] {
            fs::set_permissions(r.path().join(d), fs::Permissions::from_mode(0o555)).unwrap();
        }
        let (_, o) = run(&job(r.path(), l.path()));
        for d in ["", "2026", "2026/07", "2026/08"] {
            fs::set_permissions(r.path().join(d), fs::Permissions::from_mode(0o755)).unwrap();
        }
        assert_eq!(o.copied_missing, 2);
        assert_eq!(snapshot(r.path()), before);
        assert_eq!(snapshot(l.path()).len(), 2, "no temporary files left");
    }

    #[test]
    fn cancel_during_the_second_file_then_resume() {
        let (r, l) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        for (i, rel) in ["2026/a.jpg", "2026/b.jpg", "2026/c.jpg"].iter().enumerate() {
            put(r.path(), rel, &vec![b'x'; BLOCK + 10], t(i as u64));
        }
        let j = job(r.path(), l.path());
        let cancel = AtomicBool::new(false);
        let p = plan(&j, &mut |_| {}, &cancel).unwrap();
        let o = execute(&j, &p, &mut |pr| if pr.files_done == 1 && pr.bytes_done > (BLOCK as u64 + 10) { cancel.store(true, Ordering::Relaxed) }, &cancel);
        assert!(o.cancelled);
        assert_eq!(o.copied_missing, 1);
        let local: Vec<_> = snapshot(l.path()).into_iter().map(|f| f.0).collect();
        assert_eq!(local, vec!["2026/a.jpg"], "no temporary file of b.jpg left");
        let (_, o2) = run(&j);
        assert_eq!(o2.copied_missing, 2);
    }

    #[test]
    fn stale_temporary_files_are_cleared_fresh_ones_kept() {
        let (r, l) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        put(r.path(), "2026/a.jpg", b"a", t(1));
        let old = l.path().join("2026/.x.jpg.flb.partial");
        let fresh = l.path().join("2026/.y.jpg.flb.partial");
        put(l.path(), "2026/.x.jpg.flb.partial", b"old", SystemTime::now() - Duration::from_secs(7200));
        put(l.path(), "2026/.y.jpg.flb.partial", b"new", SystemTime::now());
        run(&job(r.path(), l.path()));
        assert!(!old.exists() && fresh.exists());
    }

    #[test]
    fn filter_and_top_level_limit() {
        let (r, l) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        put(r.path(), "2025/01/a.jpg", b"a", t(1));
        put(r.path(), "2026/01/b.jpg", b"b", t(1));
        let j = Job { only: Some(vec!["2026".into(), "1999".into()]), ..job(r.path(), l.path()) };
        let (p, _) = run(&j);
        assert_eq!(p.entries.iter().map(|e| e.rel.as_str()).collect::<Vec<_>>(), vec!["2026/01/b.jpg"]);
    }

    #[cfg(unix)]
    #[test]
    fn max_depth_stops_the_walk_before_opening_deeper_folders() {
        use std::os::unix::fs::PermissionsExt;
        let (r, l) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        put(r.path(), "2026/07/a.jpg", b"a", t(1));
        put(r.path(), "2026/07/EOSR/b.jpg", b"b", t(1));
        let deep = r.path().join("2026/07/locked");
        fs::create_dir_all(&deep).unwrap();
        fs::set_permissions(&deep, fs::Permissions::from_mode(0o000)).unwrap();
        let j = Job { max_depth: Some(2), ..job(r.path(), l.path()) };
        let (p, _) = run(&j);
        fs::set_permissions(&deep, fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(p.entries.iter().map(|e| e.rel.as_str()).collect::<Vec<_>>(), vec!["2026/07/a.jpg"]);
        assert!(p.skipped.is_empty(), "the level-3 folder was never opened: {:?}", p.skipped);
    }

    #[test]
    fn a_vanishing_remote_folder_is_an_error() {
        let (r, l) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        put(r.path(), "2025/a.jpg", b"a", t(1));
        put(r.path(), "2026/b.jpg", b"b", t(1));
        let gone = r.path().join("2026");
        let cancel = AtomicBool::new(false);
        let result = plan(&job(r.path(), l.path()), &mut |_| { let _ = fs::remove_dir_all(&gone); }, &cancel);
        assert!(matches!(result, Err(Stop::Failed(_))), "{result:?}");
        assert!(matches!(plan(&job(&r.path().join("weg"), l.path()), &mut |_| {}, &cancel), Err(Stop::Failed(_))));
    }
}
