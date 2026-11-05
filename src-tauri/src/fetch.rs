//! Fetching originals from the NAS (F009): what to fetch (S01), from the months' manifests in the
//! working copy, and the job that copies them into the working copy at their own paths, verified
//! by their hash, resumable (S02).

use crate::{hash, manifest};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Kept free on this Mac, as for the previews (feature 7/S04).
pub const RESERVE: u64 = 20_000_000_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wanted {
    /// Relative to `Library`.
    pub path: String,
    pub size: u64,
    /// `sha256:…` for an original; empty for a sidecar, which the manifest gives no hash.
    pub hash: String,
    pub sidecar: bool,
    #[serde(default)]
    pub done: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct FetchPlan {
    pub files: Vec<Wanted>,
    pub bytes: u64,
    /// Originals here already with the manifest's size.
    pub here: usize,
    /// Here with another size: never written over.
    pub conflicts: Vec<String>,
    /// In no manifest of the working copy.
    pub unknown: Vec<String>,
    /// Whether it fits on this Mac with `RESERVE` kept free; `None` when the space isn't known.
    pub fits: Option<bool>,
}

fn derived(path: &str) -> bool {
    let ext = path.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default();
    ext == "lrv" || ext == "thm"
}

/// `2014/07.Zomer/RAW/a.NEF` → `2014/07.Zomer`, the year subfolder whose manifest has it.
fn year_subfolder(path: &str) -> Option<String> {
    let mut it = path.splitn(3, '/');
    Some(format!("{}/{}", it.next()?, it.next()?))
}

/// What fetching `paths` takes (S01): from each month's manifest in `library`, the originals not
/// here yet with their sidecars; `free` is the space on this Mac.
pub fn plan(library: &Path, paths: &[String], free: Option<u64>) -> FetchPlan {
    let mut out = FetchPlan::default();
    let mut manifests: HashMap<String, Option<HashMap<String, Value>>> = HashMap::new();
    for path in paths {
        let Some(folder) = year_subfolder(path) else {
            out.unknown.push(path.clone());
            continue;
        };
        let rows = manifests.entry(folder.clone()).or_insert_with(|| {
            let m = manifest::read(&library.join(&folder).join(manifest::MANIFEST_NAME))?;
            Some(m["files"].as_array()?.iter().filter_map(|r| Some((r["path"].as_str()?.to_string(), r.clone()))).collect())
        });
        let Some(row) = rows.as_ref().and_then(|r| r.get(path)) else {
            out.unknown.push(path.clone());
            continue;
        };
        let size = row["size"].as_u64().unwrap_or(0);
        match fs::metadata(library.join(path)) {
            Ok(m) if m.len() == size => out.here += 1,
            Ok(_) => out.conflicts.push(path.clone()),
            Err(_) => {
                out.files.push(Wanted { path: path.clone(), size, hash: row["hash"].as_str().unwrap_or("").to_string(), sidecar: false, done: false });
                out.bytes += size;
                for sc in row["sidecars"].as_array().into_iter().flatten().filter_map(Value::as_str).filter(|s| !derived(s)) {
                    if !library.join(sc).exists() {
                        out.files.push(Wanted { path: sc.to_string(), size: 0, hash: String::new(), sidecar: true, done: false });
                    }
                }
            }
        }
    }
    out.fits = free.map(|f| out.bytes + RESERVE <= f);
    out
}

/// A fetch under way, kept in the app's state so it continues after an interruption (S02).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub files: Vec<Wanted>,
    pub problems: Vec<String>,
}

pub const FETCH_DIR: &str = "fetch";

impl Job {
    pub fn new(plan: &FetchPlan) -> Job {
        let id = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        Job { id, files: plan.files.clone(), problems: vec![] }
    }

    fn path(config: &Path, id: &str) -> PathBuf {
        config.join(FETCH_DIR).join(format!("{id}.json"))
    }

    pub fn save(&self, config: &Path) -> Result<(), String> {
        let p = Job::path(config, &self.id);
        fs::create_dir_all(p.parent().unwrap_or(config)).map_err(|e| e.to_string())?;
        manifest::write(&p, &serde_json::to_value(self).map_err(|e| e.to_string())?)
    }

    pub fn pending(config: &Path) -> Option<Job> {
        let rd = fs::read_dir(config.join(FETCH_DIR)).ok()?;
        let mut jobs: Vec<Job> = rd.flatten().filter_map(|e| serde_json::from_slice(&fs::read(e.path()).ok()?).ok()).collect();
        jobs.sort_by(|a, b| a.id.cmp(&b.id));
        jobs.into_iter().next()
    }

    fn finish(&self, config: &Path) {
        let _ = fs::remove_file(Job::path(config, &self.id));
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Progress {
    pub done: usize,
    pub total: usize,
    pub bytes_done: u64,
    pub bytes_total: u64,
}

/// One file: through a temporary name, only its content and time, its hash checked; never over a
/// file that is there.
fn fetch_one(nas: &Path, here: &Path, want: &str) -> Result<(), String> {
    let (from, to) = (nas.join(want), here.join(want));
    if to.exists() {
        return Err(tr!("{}: staat er al", "{}: already there", to.display()));
    }
    if let Some(d) = to.parent() {
        fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    let mut name = to.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{}.fetch", std::process::id()));
    let tmp = to.with_file_name(name);
    let r = (|| -> std::io::Result<()> {
        let t = fs::metadata(&from)?.modified()?;
        let mut out = fs::File::create(&tmp)?;
        std::io::copy(&mut fs::File::open(&from)?, &mut out)?;
        out.sync_all()?;
        out.set_modified(t)
    })();
    r.map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("{}: {e}", from.display())
    })?;
    Ok(())
}

fn tmp_of(here: &Path, want: &str) -> PathBuf {
    let to = here.join(want);
    let mut name = to.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{}.fetch", std::process::id()));
    to.with_file_name(name)
}

/// Fetches what the job still has to (S02), from `nas` (the `Library` folder on the NAS) into
/// `here` (the working copy); the journal is saved after every file.
pub fn run(job: &mut Job, nas: &Path, here: &Path, config: &Path, progress: &mut dyn FnMut(&Progress)) -> Result<(), String> {
    job.save(config)?;
    let mut p = Progress { total: job.files.len(), bytes_total: job.files.iter().map(|f| f.size).sum(), ..Default::default() };
    p.done = job.files.iter().filter(|f| f.done).count();
    p.bytes_done = job.files.iter().filter(|f| f.done).map(|f| f.size).sum();
    progress(&p);
    for i in 0..job.files.len() {
        if job.files[i].done {
            continue;
        }
        let want = job.files[i].clone();
        let to = here.join(&want.path);
        let result = if to.exists() {
            // here by now (fetched before an interruption, or by the sync): only its hash counts
            match (want.hash.is_empty(), hash::sha256_file(&to)) {
                (true, _) => Ok(()),
                (false, Ok(h)) if h == want.hash => Ok(()),
                _ => Err(tr!("{}: staat er al, met een andere inhoud", "{}: already there, with different contents", want.path)),
            }
        } else {
            fetch_one(nas, here, &want.path).and_then(|()| {
                let tmp = tmp_of(here, &want.path);
                if !want.hash.is_empty() {
                    let h = hash::sha256_file(&tmp)?;
                    if h != want.hash {
                        let _ = fs::remove_file(&tmp);
                        return Err(tr!("{}: de inhoud op de NAS klopt niet met het manifest", "{}: the contents on the NAS do not match the manifest", want.path));
                    }
                }
                if to.exists() {
                    let _ = fs::remove_file(&tmp);
                    return Err(tr!("{}: staat er al", "{}: already there", want.path));
                }
                fs::rename(&tmp, &to).map_err(|e| format!("{}: {e}", to.display()))
            })
        };
        if let Err(e) = result {
            job.problems.push(e);
        }
        job.files[i].done = true;
        p.done += 1;
        p.bytes_done += want.size;
        job.save(config)?;
        progress(&p);
    }
    job.finish(config);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn put(root: &Path, rel: &str, body: &[u8]) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }

    fn hash_of(b: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        format!("sha256:{}", Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect::<String>())
    }

    fn setup() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
        let d = tempfile::tempdir().unwrap();
        let (nas, here, cfg) = (d.path().join("nas"), d.path().join("here"), d.path().join("cfg"));
        let files = [("2014/07.Zomer/a.NEF", &b"raw a"[..]), ("2014/07.Zomer/a.xmp", b"<x/>"), ("2014/07.Zomer/b.JPG", b"jpeg b"), ("2020/08/Cam/c.MP4", b"video"), ("2020/08/Cam/c.THM", b"t")];
        for (rel, b) in files {
            put(&nas, rel, b);
        }
        let row = |p: &str, b: &[u8], sc: &[&str]| json!({"path": p, "size": b.len(), "hash": hash_of(b), "sidecars": sc});
        for (folder, rows) in [
            ("2014/07.Zomer", vec![row("2014/07.Zomer/a.NEF", b"raw a", &["2014/07.Zomer/a.xmp"]), row("2014/07.Zomer/b.JPG", b"jpeg b", &[])]),
            ("2020/08", vec![row("2020/08/Cam/c.MP4", b"video", &["2020/08/Cam/c.THM"])]),
        ] {
            let m = json!({"folder": folder, "files": rows});
            put(&here, &format!("{folder}/.manifest.json"), m.to_string().as_bytes());
            put(&nas, &format!("{folder}/.manifest.json"), m.to_string().as_bytes());
        }
        (d, nas, here, cfg)
    }

    #[test]
    fn the_plan_of_a_fetch() {
        let (_d, _nas, here, _) = setup();
        put(&here, "2014/07.Zomer/b.JPG", b"jpeg b"); // here already
        let paths: Vec<String> = ["2014/07.Zomer/a.NEF", "2014/07.Zomer/b.JPG", "2020/08/Cam/c.MP4", "2019/01/X/nowhere.JPG"].iter().map(|s| s.to_string()).collect();
        let p = plan(&here, &paths, Some(RESERVE + 100));
        let got: Vec<(&str, bool)> = p.files.iter().map(|f| (f.path.as_str(), f.sidecar)).collect();
        assert_eq!(got, [("2014/07.Zomer/a.NEF", false), ("2014/07.Zomer/a.xmp", true), ("2020/08/Cam/c.MP4", false)], "no THM");
        assert_eq!((p.bytes, p.here, p.unknown.len(), p.fits), (10, 1, 1, Some(true)));
        put(&here, "2020/08/Cam/c.MP4", b"another video");
        assert_eq!(plan(&here, &paths, Some(RESERVE)).conflicts, ["2020/08/Cam/c.MP4"]);
        assert_eq!(plan(&here, &paths[..1], Some(RESERVE)).fits, Some(false), "the reserve");
    }

    #[test]
    fn fetched_verified_and_resumed() {
        let (_d, nas, here, cfg) = setup();
        let paths: Vec<String> = ["2014/07.Zomer/a.NEF", "2020/08/Cam/c.MP4"].iter().map(|s| s.to_string()).collect();
        let p = plan(&here, &paths, None);
        let mut job = Job::new(&p);
        // as if an earlier run fetched the first and stopped
        put(&here, "2014/07.Zomer/a.NEF", b"raw a");
        let mut last = Progress::default();
        run(&mut job, &nas, &here, &cfg, &mut |pr| last = pr.clone()).unwrap();
        assert!(job.problems.is_empty(), "{:?}", job.problems);
        assert_eq!(fs::read(here.join("2020/08/Cam/c.MP4")).unwrap(), b"video");
        assert_eq!(fs::read(here.join("2014/07.Zomer/a.xmp")).unwrap(), b"<x/>");
        assert_eq!((last.done, last.total), (3, 3));
        assert!(Job::pending(&cfg).is_none(), "the journal is gone");
        assert!(!fs::read_dir(here.join("2020/08/Cam")).unwrap().flatten().any(|e| e.file_name().to_string_lossy().ends_with(".fetch")));
    }

    #[test]
    fn a_file_that_differs_on_the_nas_is_not_kept() {
        let (_d, nas, here, cfg) = setup();
        put(&nas, "2020/08/Cam/c.MP4", b"VIDEO"); // the same size, other content
        let p = plan(&here, &["2020/08/Cam/c.MP4".to_string()], None);
        let mut job = Job::new(&p);
        run(&mut job, &nas, &here, &cfg, &mut |_| {}).unwrap();
        assert!(job.problems[0].contains("klopt niet"));
        assert!(!here.join("2020/08/Cam/c.MP4").exists());
    }
}
