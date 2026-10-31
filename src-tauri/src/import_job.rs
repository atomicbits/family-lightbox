//! Importing one source (F018/S05): its new files staged in the working copy, put on the NAS with
//! their month's manifest (S02), kept locally for the months this Mac keeps, and only then the
//! originals moved to the drop folder's `_imported/`. Each step is written to `job.json`, so an
//! import that stops (the app quits, the NAS goes away) continues where it was.

use crate::import_plan::{Action, SourcePlan};
use crate::{hash, manifest, push};
use chrono::{Datelike, Local};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const STAGING: &str = ".import";
const JOB_FILE: &str = "job.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileState {
    /// To copy, not yet staged.
    Planned,
    Staged,
    /// Verified on the NAS.
    OnNas,
    /// The library has it already: only the original is tidied away.
    Duplicate,
    /// The original is in `_imported/` (or deleted).
    Done,
    /// Couldn't be staged or put on the NAS: left in the source.
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobFile {
    pub src: String,
    /// Relative to the library root.
    pub target: String,
    pub hash: String,
    pub size: u64,
    pub state: FileState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    Stage,
    Nas,
    Local,
    Source,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    /// The source folder (a drop folder's subfolder).
    pub source: String,
    /// The drop folder it is in, where `_imported/` goes.
    pub drop_root: String,
    pub name: String,
    pub delete_imported: bool,
    pub keep_months: u32,
    pub step: Step,
    pub files: Vec<JobFile>,
    pub problems: Vec<String>,
    pub created: String,
}

/// Where a job runs: the working copy, the `Library` share (none when it isn't connected), the
/// device writing the manifests, and the month from which on this Mac keeps originals.
pub struct Env<'a> {
    pub library: &'a Path,
    pub share: Option<&'a Path>,
    pub device: &'a str,
    pub keep_from: String,
}

/// The first month this Mac keeps: `keep` months back from now, as `YYYY/MM`.
pub fn keep_from(keep: u32) -> String {
    let now = Local::now();
    let months = now.year() * 12 + now.month0() as i32 - keep as i32 + 1;
    format!("{:04}/{:02}", months.div_euclid(12), months.rem_euclid(12) + 1)
}

impl Job {
    pub fn from_plan(plan: &SourcePlan, drop_root: &Path, id: &str, delete_imported: bool, keep_months: u32) -> Job {
        let files = plan
            .files
            .iter()
            .map(|f| JobFile {
                src: f.src.clone(),
                target: f.target.clone(),
                hash: f.hash.clone(),
                size: f.size,
                state: if matches!(f.action, Action::Duplicate { .. }) { FileState::Duplicate } else { FileState::Planned },
            })
            .collect();
        Job {
            id: id.to_string(),
            source: plan.path.clone(),
            drop_root: drop_root.display().to_string(),
            name: plan.name.clone(),
            delete_imported,
            keep_months,
            step: Step::Stage,
            files,
            problems: Vec::new(),
            created: manifest::local_iso(Local::now()),
        }
    }

    pub fn dir(library: &Path, id: &str) -> PathBuf {
        library.join(STAGING).join(id)
    }

    pub fn save(&self, library: &Path) -> Result<(), String> {
        let dir = Job::dir(library, &self.id);
        fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let v = serde_json::to_value(self).map_err(|e| e.to_string())?;
        manifest::write(&dir.join(JOB_FILE), &v)
    }

    pub fn count(&self, s: FileState) -> usize {
        self.files.iter().filter(|f| f.state == s).count()
    }
}

/// Imports that didn't finish, oldest first.
pub fn pending(library: &Path) -> Vec<Job> {
    let mut out: Vec<Job> = fs::read_dir(library.join(STAGING))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| serde_json::from_slice::<Job>(&fs::read(e.path().join(JOB_FILE)).ok()?).ok())
        .filter(|j| j.step != Step::Done)
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Progress {
    pub step: String,
    pub done: usize,
    pub total: usize,
}

/// Copies `from` to `to` through a temporary name, keeping its modification time.
fn copy_with_time(from: &Path, to: &Path) -> std::io::Result<()> {
    if let Some(d) = to.parent() {
        fs::create_dir_all(d)?;
    }
    let mut name = to.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{}.tmp", std::process::id()));
    let tmp = to.with_file_name(name);
    let r = fs::copy(from, &tmp).and_then(|_| {
        let t = fs::metadata(from)?.modified()?;
        fs::File::options().write(true).open(&tmp)?.set_modified(t)?;
        fs::rename(&tmp, to)
    });
    if r.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    r
}

/// Moves `from` to `to`, by renaming where it can and copying otherwise; never over a file.
fn move_file(from: &Path, to: &Path) -> std::io::Result<()> {
    if let Some(d) = to.parent() {
        fs::create_dir_all(d)?;
    }
    let mut to = to.to_path_buf();
    let mut n = 2;
    while to.exists() {
        let stem = to.file_stem().unwrap_or_default().to_string_lossy().into_owned();
        let ext = to.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        to.set_file_name(format!("{} ({n}){ext}", stem.trim_end_matches(&format!(" ({})", n - 1))));
        n += 1;
    }
    if fs::rename(from, &to).is_err() {
        copy_with_time(from, &to)?;
        fs::remove_file(from)?;
    }
    Ok(())
}

/// Removes the empty folders below `root`, never `root` itself.
fn prune(root: &Path) {
    fn go(dir: &Path) -> bool {
        let Ok(rd) = fs::read_dir(dir) else { return false };
        let mut empty = true;
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name();
            if p.is_dir() {
                if go(&p) {
                    let _ = fs::remove_dir(&p);
                } else {
                    empty = false;
                }
            } else if name == ".DS_Store" {
                let _ = fs::remove_file(&p);
            } else {
                empty = false;
            }
        }
        empty
    }
    go(root);
}

/// Runs the job from its current step to the end, saving it after every step; `stop_after` ends
/// it early, as an interruption would (for the tests).
pub fn run(job: &mut Job, env: &Env, progress: &mut dyn FnMut(&Progress), stop_after: Option<Step>) -> Result<(), String> {
    let staging = Job::dir(env.library, &job.id);
    job.save(env.library)?;
    loop {
        let step = job.step;
        match step {
            Step::Stage => stage(job, &staging, progress)?,
            Step::Nas => nas(job, &staging, env, progress)?,
            Step::Local => local(job, &staging, env)?,
            Step::Source => source(job, progress),
            Step::Done => break,
        }
        job.step = match step {
            Step::Stage => Step::Nas,
            Step::Nas => Step::Local,
            Step::Local => Step::Source,
            _ => Step::Done,
        };
        job.save(env.library)?;
        if stop_after == Some(step) {
            return Ok(());
        }
    }
    // nothing of the job is left but its record; the staging folder goes when it is empty
    prune(&staging);
    Ok(())
}

fn stage(job: &mut Job, staging: &Path, progress: &mut dyn FnMut(&Progress)) -> Result<(), String> {
    let total = job.count(FileState::Planned);
    let mut done = 0;
    for i in 0..job.files.len() {
        if job.files[i].state != FileState::Planned {
            continue;
        }
        let f = &mut job.files[i];
        let to = staging.join(&f.target);
        let result = copy_with_time(Path::new(&f.src), &to).map_err(|e| format!("{}: {e}", f.src)).and_then(|_| {
            let h = hash::sha256_file(&to)?;
            if h == f.hash { Ok(()) } else { Err(format!("{}: changed since the plan", f.src)) }
        });
        match result {
            Ok(()) => f.state = FileState::Staged,
            Err(e) => {
                let _ = fs::remove_file(&to);
                f.state = FileState::Failed;
                job.problems.push(e);
            }
        }
        done += 1;
        progress(&Progress { step: "stage".into(), done, total });
    }
    Ok(())
}

/// The month folders of the files in `state`.
fn folders(job: &Job, state: FileState) -> BTreeSet<String> {
    job.files.iter().filter(|f| f.state == state).filter_map(|f| f.target.get(..7).map(str::to_string)).collect()
}

fn nas(job: &mut Job, staging: &Path, env: &Env, progress: &mut dyn FnMut(&Progress)) -> Result<(), String> {
    let share = env.share.ok_or("de NAS is niet verbonden: de import gaat verder zodra hij dat is")?;
    let months = folders(job, FileState::Staged);
    let total = months.len();
    for (n, folder) in months.iter().enumerate() {
        // the hashes known already, so building the manifest reads no file twice
        let rows: Vec<Value> = job
            .files
            .iter()
            .filter(|f| f.state == FileState::Staged && f.target.starts_with(&format!("{folder}/")))
            .filter_map(|f| {
                let m = fs::metadata(staging.join(&f.target)).ok()?;
                Some(json!({"path": f.target, "size": m.len(), "hash": f.hash, "mtime": manifest::local_iso(m.modified().ok()?)}))
            })
            .collect();
        let existing = json!({ "files": rows });
        let m = manifest::build(staging, folder, env.device, Some(&existing))?;
        let out = push::push_folder(staging, share, &m)?;
        // verified: a row that isn't missing or in conflict, and its sidecars that aren't missing
        let bad: HashSet<&str> = out.missing.iter().chain(&out.conflicts).map(String::as_str).collect();
        let mut ok: HashSet<&str> = HashSet::new();
        for r in m["files"].as_array().into_iter().flatten() {
            let Some(p) = r["path"].as_str().filter(|p| !bad.contains(p)) else { continue };
            ok.insert(p);
            ok.extend(r["sidecars"].as_array().into_iter().flatten().filter_map(Value::as_str).filter(|x| !bad.contains(x)));
        }
        for f in job.files.iter_mut().filter(|f| f.state == FileState::Staged && ok.contains(f.target.as_str())) {
            f.state = FileState::OnNas;
        }
        for p in out.missing.iter().chain(&out.conflicts).chain(&out.failed) {
            job.problems.push(format!("NAS: {p}"));
        }
        progress(&Progress { step: "nas".into(), done: n + 1, total });
    }
    // what didn't reach the NAS stays in the source
    for f in job.files.iter_mut().filter(|f| f.state == FileState::Staged) {
        f.state = FileState::Failed;
    }
    Ok(())
}

fn local(job: &mut Job, staging: &Path, env: &Env) -> Result<(), String> {
    for folder in folders(job, FileState::OnNas) {
        // the merged manifest of the month, for the index, whether its files stay here or not
        let staged = staging.join(&folder).join(manifest::MANIFEST_NAME);
        if let Some(m) = manifest::read(&staged) {
            fs::create_dir_all(env.library.join(&folder)).map_err(|e| format!("{folder}: {e}"))?;
            manifest::write(&env.library.join(&folder).join(manifest::MANIFEST_NAME), &m)?;
            let _ = fs::remove_file(&staged);
        }
    }
    for f in job.files.iter().filter(|f| f.state == FileState::OnNas) {
        let staged = staging.join(&f.target);
        if !staged.exists() {
            continue; // moved before an interruption
        }
        if f.target.get(..7).unwrap_or("") >= env.keep_from.as_str() && !env.library.join(&f.target).exists() {
            move_file(&staged, &env.library.join(&f.target)).map_err(|e| format!("{}: {e}", f.target))?;
        } else {
            let _ = fs::remove_file(&staged);
        }
    }
    Ok(())
}

fn source(job: &mut Job, progress: &mut dyn FnMut(&Progress)) {
    let root = PathBuf::from(&job.drop_root);
    let total = job.files.iter().filter(|f| matches!(f.state, FileState::OnNas | FileState::Duplicate)).count();
    for (i, f) in job.files.iter_mut().filter(|f| matches!(f.state, FileState::OnNas | FileState::Duplicate)).enumerate() {
        let src = Path::new(&f.src);
        let result = if !src.exists() {
            Ok(()) // tidied before an interruption
        } else if job.delete_imported {
            fs::remove_file(src)
        } else {
            let rel = src.strip_prefix(&root).unwrap_or(src);
            move_file(src, &root.join(crate::sources::IMPORTED).join(rel))
        };
        match result {
            Ok(()) => f.state = FileState::Done,
            Err(e) => job.problems.push(format!("{}: {e}", f.src)),
        }
        progress(&Progress { step: "source".into(), done: i + 1, total });
    }
    prune(Path::new(&job.source));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::fixtures::{jpeg_file, tiff, CANON};
    use crate::import_plan::{plan, HashCache};
    use crate::sources::Layout;
    use std::collections::HashMap;

    struct F {
        _d: tempfile::TempDir,
        drop: PathBuf,
        library: PathBuf,
        share: PathBuf,
    }

    fn fixture() -> F {
        let d = tempfile::tempdir().unwrap();
        let (drop, library, share) = (d.path().join("Offload"), d.path().join("Library"), d.path().join("nas"));
        let mk = |rel: &str, b: &[u8]| {
            let p = drop.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, b).unwrap();
        };
        mk("Cam/100/IMG_1.JPG", &jpeg_file(&tiff(true, CANON, &[(0x9003, "2026:07:30 10:00:00")])));
        mk("Cam/100/IMG_2.JPG", &jpeg_file(&tiff(true, CANON, &[(0x9003, "2016:02:01 10:00:00")])));
        mk("Cam/100/IMG_1.xmp", b"<xmp/>");
        mk("Cam/101/OLD.JPG", b"already in the library");
        fs::create_dir_all(&library).unwrap();
        // the NAS has a file of that month already, and the library has OLD.JPG's content
        let theirs = share.join("2026/07/Other/x.jpg");
        fs::create_dir_all(theirs.parent().unwrap()).unwrap();
        fs::write(&theirs, b"theirs").unwrap();
        manifest::write(&share.join("2026/07/.manifest.json"), &json!({"contract_version": "1.3", "folder": "2026/07", "files": [{"path": "2026/07/Other/x.jpg", "size": 6, "hash": "sha256:t", "sidecars": []}]})).unwrap();
        F { _d: d, drop, library, share }
    }

    fn job(f: &F) -> Job {
        let known: HashMap<String, String> = [("2019/01/X/OLD.JPG".to_string(), hash::sha256_file(&f.drop.join("Cam/101/OLD.JPG")).unwrap())].into_iter().collect();
        let cache = HashCache::open(&f.library.join("h.sqlite")).unwrap();
        let p = plan(&f.drop.join("Cam"), "Cam", Layout { split: false, day: true }, &[], &known, &cache, &mut |_| {});
        Job::from_plan(&p, &f.drop, "20261031-120000", false, 18)
    }

    fn env<'a>(f: &'a F, share: bool) -> Env<'a> {
        Env { library: &f.library, share: share.then_some(f.share.as_path()), device: "mbp-test", keep_from: "2025/06".into() }
    }

    fn check_result(f: &F) {
        // on the NAS: the new files and the merged manifests
        assert!(f.share.join("2026/07/Cam/2026-07-30/IMG_1.JPG").exists() && f.share.join("2026/07/Cam/2026-07-30/IMG_1.xmp").exists());
        assert!(f.share.join("2016/02/Cam/2016-02-01/IMG_2.JPG").exists());
        let nas = manifest::read(&f.share.join("2026/07/.manifest.json")).unwrap();
        let paths: Vec<&str> = nas["files"].as_array().unwrap().iter().map(|r| r["path"].as_str().unwrap()).collect();
        assert_eq!(paths, ["2026/07/Cam/2026-07-30/IMG_1.JPG", "2026/07/Other/x.jpg"]);
        assert_eq!(nas["files"][0]["sidecars"], json!(["2026/07/Cam/2026-07-30/IMG_1.xmp"]));
        // locally: both manifests, the originals only of the months kept
        assert_eq!(manifest::read(&f.library.join("2026/07/.manifest.json")), Some(nas));
        assert!(f.library.join("2016/02/.manifest.json").exists());
        assert!(f.library.join("2026/07/Cam/2026-07-30/IMG_1.JPG").exists());
        assert!(!f.library.join("2016/02/Cam/2016-02-01/IMG_2.JPG").exists(), "2016 isn't kept here");
        // the source: everything in _imported, the source folder itself kept and empty
        for rel in ["Cam/100/IMG_1.JPG", "Cam/100/IMG_1.xmp", "Cam/100/IMG_2.JPG", "Cam/101/OLD.JPG"] {
            assert!(f.drop.join("_imported").join(rel).exists(), "{rel}");
        }
        assert!(f.drop.join("Cam").is_dir() && fs::read_dir(f.drop.join("Cam")).unwrap().count() == 0);
        // nothing left in the staging folder but the job's record, which says done
        let left: Vec<_> = fs::read_dir(Job::dir(&f.library, "20261031-120000")).unwrap().flatten().map(|e| e.file_name()).collect();
        assert_eq!(left, [std::ffi::OsString::from(JOB_FILE)]);
        assert!(pending(&f.library).is_empty());
    }

    #[test]
    fn a_whole_import() {
        let f = fixture();
        let mut j = job(&f);
        assert_eq!((j.count(FileState::Planned), j.count(FileState::Duplicate)), (3, 1));
        let mut steps = Vec::new();
        run(&mut j, &env(&f, true), &mut |p| steps.push(p.step.clone()), None).unwrap();
        assert_eq!(j.step, Step::Done);
        assert!(j.problems.is_empty(), "{:?}", j.problems);
        assert_eq!(j.count(FileState::Done), 4);
        assert!(steps.contains(&"stage".to_string()) && steps.contains(&"nas".to_string()) && steps.contains(&"source".to_string()));
        check_result(&f);
    }

    #[test]
    fn stopped_after_any_step_it_continues_to_the_same_result() {
        for stop in [Step::Stage, Step::Nas, Step::Local] {
            let f = fixture();
            let mut j = job(&f);
            run(&mut j, &env(&f, true), &mut |_| {}, Some(stop)).unwrap();
            // nothing is tidied in the source before the files are on the NAS
            assert!(f.drop.join("Cam/100/IMG_1.JPG").exists(), "{stop:?}");
            let mut again = pending(&f.library).pop().expect("the job is pending");
            run(&mut again, &env(&f, true), &mut |_| {}, None).unwrap();
            assert!(again.problems.is_empty(), "{stop:?}: {:?}", again.problems);
            check_result(&f);
        }
    }

    #[test]
    fn without_the_nas_it_waits_and_tidies_nothing() {
        let f = fixture();
        let mut j = job(&f);
        let err = run(&mut j, &env(&f, false), &mut |_| {}, None).unwrap_err();
        assert!(err.contains("niet verbonden"));
        assert_eq!(pending(&f.library)[0].step, Step::Nas);
        assert!(f.drop.join("Cam/100/IMG_1.JPG").exists() && !f.drop.join("_imported").exists());
        let mut again = pending(&f.library).pop().unwrap();
        run(&mut again, &env(&f, true), &mut |_| {}, None).unwrap();
        check_result(&f);
    }

    #[test]
    fn a_file_changed_after_the_plan_stays_where_it_is() {
        let f = fixture();
        let mut j = job(&f);
        fs::write(f.drop.join("Cam/100/IMG_2.JPG"), b"changed").unwrap();
        run(&mut j, &env(&f, true), &mut |_| {}, None).unwrap();
        assert_eq!(j.count(FileState::Failed), 1);
        assert!(j.problems[0].contains("changed since the plan"));
        assert!(f.drop.join("Cam/100/IMG_2.JPG").exists() && !f.share.join("2016/02/Cam").exists());
    }

    #[test]
    fn the_months_kept() {
        let k = keep_from(18);
        assert_eq!(k.len(), 7);
        assert!(k < keep_from(1) && keep_from(1) <= Local::now().format("%Y/%m").to_string());
    }
}
