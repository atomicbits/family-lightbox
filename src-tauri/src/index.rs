//! Builds the app's own index from the manifests in the working copy (F008/S01). Same tables and
//! rows as `media index` in the CLI (data contract 7), but in its own file, which only the app
//! writes, so `db.rs` reads it unchanged and the CLI's `index.sqlite` is left alone.

use crate::sync::excluded;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Instant, UNIX_EPOCH};

pub const INDEX_FILE: &str = "familylightbox-index.sqlite";
const MANIFEST: &str = ".manifest.json";

/// The CLI's schema (`index_tables` in `media`), so both indexes read the same.
const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS folders (folder TEXT PRIMARY KEY, month TEXT, contract_version TEXT,
        generated_at TEXT, generated_by TEXT, n_files INTEGER);
    CREATE TABLE IF NOT EXISTS items (path TEXT PRIMARY KEY, folder TEXT, month TEXT, size INTEGER,
        hash TEXT, taken TEXT, taken_from TEXT, kind TEXT, source TEXT, sidecars TEXT,
        local INTEGER, preview INTEGER);
    CREATE INDEX IF NOT EXISTS items_month ON items(month);
    CREATE INDEX IF NOT EXISTS items_hash ON items(hash);
    CREATE INDEX IF NOT EXISTS items_taken ON items(taken);
    CREATE TABLE IF NOT EXISTS live_photos (video TEXT PRIMARY KEY, photo TEXT);
    CREATE TABLE IF NOT EXISTS cameras (path TEXT PRIMARY KEY, camera TEXT);
";

/// The number of an iPhone file name, `IMG_1234` or `IMG_E1234` at the end of its stem, also after
/// the import's date prefix (F017/S01).
pub fn img_number(path: &str) -> Option<u32> {
    let name = path.rsplit('/').next()?;
    let stem = name.rsplit_once('.').map_or(name, |(s, _)| s).to_ascii_uppercase();
    let at = stem.rfind("IMG_")?;
    let rest = &stem[at + 4..];
    let digits = rest.strip_prefix('E').unwrap_or(rest);
    (digits.len() == 4 && digits.bytes().all(|b| b.is_ascii_digit())).then(|| digits.parse().ok())?
}

/// The local time of a `taken` (the phone's own clock), to compare two files of one phone.
fn taken_time(taken: &str) -> Option<chrono::NaiveDateTime> {
    chrono::NaiveDateTime::parse_from_str(taken.get(..19)?, "%Y-%m-%dT%H:%M:%S").ok()
}

/// How far apart a Live Photo's video and its photo may be taken.
pub const LIVE_SECONDS: i64 = 3;

/// The Live Photos (F017/S01): each `.MOV` video with the photo of the same month and number taken
/// within `LIVE_SECONDS`, the nearest when there are several. Rows are (path, month, kind, taken).
pub fn live_pairs(rows: &[(String, String, String, Option<String>)]) -> Vec<(String, String)> {
    use std::collections::HashMap;
    let mut photos: HashMap<(&str, u32), Vec<(&str, chrono::NaiveDateTime)>> = HashMap::new();
    for (path, month, kind, taken) in rows {
        if kind != "photo" {
            continue;
        }
        if let (Some(n), Some(t)) = (img_number(path), taken.as_deref().and_then(taken_time)) {
            photos.entry((month.as_str(), n)).or_default().push((path.as_str(), t));
        }
    }
    let mut out = Vec::new();
    for (path, month, kind, taken) in rows {
        if kind != "video" || !path.to_ascii_uppercase().ends_with(".MOV") {
            continue;
        }
        let (Some(n), Some(t)) = (img_number(path), taken.as_deref().and_then(taken_time)) else { continue };
        let nearest = photos
            .get(&(month.as_str(), n))
            .into_iter()
            .flatten()
            .map(|(p, pt)| (*p, (*pt - t).num_seconds().abs()))
            .filter(|(_, d)| *d <= LIVE_SECONDS)
            .min_by_key(|(_, d)| *d);
        if let Some((photo, _)) = nearest {
            out.push((path.clone(), photo.to_string()));
        }
    }
    out
}

#[derive(Deserialize)]
struct Manifest {
    contract_version: Option<String>,
    folder: Option<String>,
    month: Option<String>,
    generated_at: Option<String>,
    generated_by: Option<String>,
    files: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    path: String,
    size: Option<i64>,
    hash: Option<String>,
    kind: Option<String>,
    source: Option<String>,
    taken: Option<String>,
    taken_from: Option<String>,
    sidecars: Option<Vec<String>>,
    /// Make and model (data contract 1.3, F002/S01); kept apart from `items`, which mirrors the CLI.
    camera: Option<String>,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct BuildOutcome {
    pub manifests: usize,
    pub items: usize,
    pub photos: usize,
    pub videos: usize,
    pub sidecars: usize,
    pub other: usize,
    pub local: usize,
    pub previews: usize,
    /// Manifests that couldn't be read, as `<year>/<subfolder>` with the reason.
    pub broken: Vec<String>,
    pub millis: u64,
}

/// Every `<year>/<subfolder>/.manifest.json`, years and subfolders sorted by name (as the CLI).
pub fn manifest_paths(library: &Path) -> Result<Vec<PathBuf>, String> {
    let years = fs::read_dir(library).map_err(|e| format!("werkkopie niet gevonden: {} ({e})", library.display()))?;
    let mut years: Vec<PathBuf> = years
        .filter_map(|e| e.ok())
        .filter(|e| is_year(&e.file_name().to_string_lossy()) && e.path().is_dir())
        .map(|e| e.path())
        .collect();
    years.sort();
    let mut out = Vec::new();
    for year in years {
        let Ok(subs) = fs::read_dir(&year) else { continue }; // a year this account may not read
        let mut subs: Vec<PathBuf> = subs
            .filter_map(|e| e.ok())
            .filter(|e| !excluded(&e.file_name().to_string_lossy()))
            .map(|e| e.path().join(MANIFEST))
            .filter(|m| m.is_file())
            .collect();
        subs.sort();
        out.extend(subs);
    }
    Ok(out)
}

fn is_year(name: &str) -> bool {
    name.len() == 4 && name.bytes().all(|b| b.is_ascii_digit())
}

/// Builds the index at `target` from the manifests under `library`, in a temporary file that is
/// renamed over the old one. A failure leaves the old index as it was.
pub fn build(library: &Path, previews: &Path, target: &Path) -> Result<BuildOutcome, String> {
    let started = Instant::now();
    let paths = manifest_paths(library)?;
    let tmp = target.with_file_name(format!(
        "{}.tmp",
        target.file_name().map(|n| n.to_string_lossy()).unwrap_or_default()
    ));
    let _ = fs::remove_file(&tmp);
    if let Some(dir) = target.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("map {} maken mislukt: {e}", dir.display()))?;
    }
    let result = write(&tmp, library, previews, &paths).and_then(|mut out| {
        fs::rename(&tmp, target).map_err(|e| format!("index vervangen mislukt: {e}"))?;
        out.millis = started.elapsed().as_millis() as u64;
        Ok(out)
    });
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

fn write(tmp: &Path, library: &Path, previews: &Path, paths: &[PathBuf]) -> Result<BuildOutcome, String> {
    let err = |e: rusqlite::Error| format!("index schrijven mislukt: {e}");
    let mut conn = Connection::open(tmp).map_err(err)?;
    conn.execute_batch(SCHEMA).map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let mut out = BuildOutcome { manifests: paths.len(), ..Default::default() };
    {
        let mut item = tx
            .prepare("INSERT OR REPLACE INTO items VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)")
            .map_err(err)?;
        let mut folder_row = tx.prepare("INSERT OR REPLACE INTO folders VALUES (?1,?2,?3,?4,?5,?6)").map_err(err)?;
        let mut camera_row = tx.prepare("INSERT OR REPLACE INTO cameras VALUES (?1, ?2)").map_err(err)?;
        for mf in paths {
            let dir = mf.parent().unwrap_or(mf);
            let rel = dir.strip_prefix(library).unwrap_or(dir).to_string_lossy().replace('\\', "/");
            let man: Manifest = match fs::read(mf)
                .map_err(|e| e.to_string())
                .and_then(|b| serde_json::from_slice(&b).map_err(|e| e.to_string()))
            {
                Ok(m) => m,
                Err(e) => {
                    out.broken.push(format!("{rel}: {e}"));
                    continue;
                }
            };
            let folder = man.folder.clone().filter(|f| !f.is_empty()).unwrap_or(rel);
            for r in &man.files {
                let kind = r.kind.as_deref();
                let media = matches!(kind, Some("photo" | "video"));
                let local = library.join(&r.path).is_file();
                let preview = media && previews.join(format!("{}.jpg", r.path)).is_file();
                let sidecars = serde_json::to_string(r.sidecars.as_deref().unwrap_or(&[])).unwrap_or_default();
                item.execute(params![
                    r.path, folder, man.month, r.size, r.hash, r.taken, r.taken_from, kind, r.source, sidecars,
                    local as i64, preview as i64
                ])
                .map_err(err)?;
                if let (true, Some(camera)) = (media, r.camera.as_deref().filter(|c| !c.trim().is_empty())) {
                    camera_row.execute(params![r.path, camera]).map_err(err)?;
                }
                match kind {
                    Some("photo") => out.photos += 1,
                    Some("video") => out.videos += 1,
                    Some("sidecar") => out.sidecars += 1,
                    _ => out.other += 1,
                }
                out.local += local as usize;
                out.previews += preview as usize;
            }
            folder_row
                .execute(params![
                    folder,
                    man.month,
                    man.contract_version,
                    man.generated_at,
                    man.generated_by,
                    man.files.len() as i64
                ])
                .map_err(err)?;
            out.items += man.files.len();
        }
    }
    // the Live Photos, from the rows just written (F017/S01)
    {
        let rows: Vec<(String, String, String, Option<String>)> = {
            let mut stmt = tx
                .prepare("SELECT path, coalesce(month, folder), kind, taken FROM items WHERE kind IN ('photo', 'video')")
                .map_err(err)?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))).map_err(err)?;
            rows.collect::<Result<_, _>>().map_err(err)?
        };
        let mut insert = tx.prepare("INSERT OR REPLACE INTO live_photos VALUES (?1, ?2)").map_err(err)?;
        for (video, photo) in live_pairs(&rows) {
            insert.execute(params![video, photo]).map_err(err)?;
        }
    }
    tx.commit().map_err(err)?;
    conn.close().map_err(|(_, e)| err(e))?;
    Ok(out)
}

/// Sets the preview flag of these paths in the index, for previews made since the last build
/// (F012/S04), so the grid shows them without a rebuild. A build running meanwhile replaces the
/// file; it reads the previews folder itself, so it misses at most the ones made during it, until
/// the next build.
pub fn mark_previews(target: &Path, paths: &[&str]) -> Result<usize, String> {
    let err = |e: rusqlite::Error| format!("index bijwerken mislukt: {e}");
    let mut conn = Connection::open_with_flags(target, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE).map_err(err)?;
    conn.busy_timeout(std::time::Duration::from_secs(3)).map_err(err)?;
    let tx = conn.transaction().map_err(err)?;
    let mut n = 0;
    {
        let mut stmt = tx.prepare("UPDATE items SET preview = 1 WHERE path = ?1 AND preview = 0").map_err(err)?;
        for p in paths {
            n += stmt.execute([p]).map_err(err)?;
        }
    }
    tx.commit().map_err(err)?;
    Ok(n)
}

/// What the manifests look like now: their paths, sizes and modification times. A sync keeps the
/// NAS's time on a copied manifest, which can be older than the index, so comparing with the
/// index's own time wouldn't do.
pub fn fingerprint(library: &Path) -> Option<u64> {
    let paths = manifest_paths(library).ok()?;
    let mut h = DefaultHasher::new();
    for p in &paths {
        let meta = fs::metadata(p).ok();
        p.hash(&mut h);
        meta.as_ref().map(|m| m.len()).hash(&mut h);
        meta.and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .hash(&mut h);
    }
    Some(h.finish())
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct IndexState {
    pub running: bool,
    /// The last build that finished, or none yet.
    pub outcome: Option<BuildOutcome>,
    pub error: Option<String>,
    /// Builds finished since the app started; the grid reloads when it goes up.
    pub builds: u64,
}

pub trait IndexSink: Send + Sync + 'static {
    fn done(&self, state: &IndexState);
}

#[derive(Clone, Debug)]
pub struct IndexJob {
    pub library: PathBuf,
    pub previews: PathBuf,
    pub target: PathBuf,
}

#[derive(Default)]
struct RunnerInner {
    state: IndexState,
    /// A request came in during a build: build once more afterwards.
    queued: bool,
    /// The manifests as they were when the last good build started.
    built_from: Option<u64>,
}

/// One build at a time, on its own thread (F008/S02).
#[derive(Clone, Default)]
pub struct IndexRunner {
    inner: Arc<Mutex<RunnerInner>>,
}

impl IndexRunner {
    pub fn status(&self) -> IndexState {
        self.inner.lock().map(|i| i.state.clone()).unwrap_or_default()
    }

    /// Whether the manifests changed since the last build (or there is no index yet).
    pub fn stale(&self, job: &IndexJob) -> bool {
        let built = self.inner.lock().ok().and_then(|i| i.built_from);
        !job.target.is_file() || built.is_none() || built != fingerprint(&job.library)
    }

    /// Build now, or once more after the build that is running. Returns the thread when one
    /// was started, `None` when the request was queued.
    pub fn request(&self, job: IndexJob, sink: Arc<dyn IndexSink>) -> Option<std::thread::JoinHandle<()>> {
        {
            let mut i = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if i.state.running {
                i.queued = true;
                return None;
            }
            i.state.running = true;
        }
        let me = self.clone();
        Some(std::thread::spawn(move || me.run(&job, sink.as_ref())))
    }

    fn run(&self, job: &IndexJob, sink: &dyn IndexSink) {
        loop {
            let before = fingerprint(&job.library);
            let result = build(&job.library, &job.previews, &job.target);
            let (state, again) = {
                let mut i = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                match result {
                    Ok(out) => {
                        i.state.outcome = Some(out);
                        i.state.error = None;
                        i.built_from = before;
                    }
                    Err(e) => i.state.error = Some(e),
                }
                i.state.builds += 1;
                let again = std::mem::take(&mut i.queued);
                i.state.running = again;
                (i.state.clone(), again)
            };
            sink.done(&state);
            if !again {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use serde_json::json;

    struct Fixture {
        _dir: tempfile::TempDir,
        library: PathBuf,
        previews: PathBuf,
        target: PathBuf,
    }

    fn touch(p: &Path) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, b"x").unwrap();
    }

    fn manifest(f: &Fixture, sub: &str, value: serde_json::Value) {
        let p = f.library.join(sub).join(MANIFEST);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, serde_json::to_vec(&value).unwrap()).unwrap();
    }

    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let f = Fixture {
            library: dir.path().join("LibraryLocal"),
            previews: dir.path().join("Previews"),
            target: dir.path().join("config").join(INDEX_FILE),
            _dir: dir,
        };
        let p = "2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3";
        let v = "2026/07/EOSR/videos/2026-07-30/CO7A9140.MP4";
        let x = "2026/07/EOSR/photos/2026-07-30/CO7A9123.xmp";
        let o = "2026/07/EOSR/photos/2026-07-30/LOST.xmp";
        manifest(
            &f,
            "2026/07",
            json!({
                "contract_version": "1.2", "folder": "2026/07", "month": "2026/07",
                "generated_at": "2026-10-21T22:14:03+02:00", "generated_by": "mbp-peter",
                "files": [
                    {"path": p, "size": 25, "hash": "sha256:a", "kind": "photo", "source": "EOSR",
                     "taken": "2026-07-30T16:58:12+02:00", "taken_from": "exif", "sidecars": [x],
                     "camera": "Canon EOS R"},
                    {"path": v, "size": 90, "hash": "sha256:b", "kind": "video", "source": "EOSR",
                     "taken": "2026-07-30T17:00:00+02:00", "taken_from": "exif"},
                    {"path": o, "size": 1, "hash": "sha256:c", "kind": "sidecar", "source": "EOSR",
                     "taken": null, "taken_from": "none"},
                    {"path": "2026/07/notes.txt", "size": 3, "hash": "sha256:d", "kind": "other",
                     "taken": null, "taken_from": "none"}
                ]
            }),
        );
        // version 1.1: no `folder`; `month` null for a named folder
        manifest(
            &f,
            "2012/scans Peter",
            json!({"contract_version": "1.1", "month": null, "generated_at": "2026-10-01T10:00:00+02:00",
                   "generated_by": "raven",
                   "files": [{"path": "2012/scans Peter/scan1.tif", "size": 7, "hash": "sha256:e",
                              "kind": "photo", "taken": null, "taken_from": "none"}]}),
        );
        touch(&f.library.join(p));
        touch(&f.library.join(o));
        touch(&f.previews.join(format!("{p}.jpg")));
        touch(&f.previews.join(format!("{v}.jpg")));
        touch(&f.previews.join(format!("{o}.jpg"))); // a sidecar never counts as a preview
        f
    }

    /// path, folder, month, kind, sidecars, local, preview
    type Row = (String, String, Option<String>, String, String, i64, i64);

    fn rows(target: &Path) -> Vec<Row> {
        let c = Connection::open(target).unwrap();
        let mut s = c
            .prepare("SELECT path, folder, month, kind, sidecars, local, preview FROM items ORDER BY path")
            .unwrap();
        s.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    }

    #[test]
    fn rows_and_flags_follow_the_manifests_and_the_files() {
        let f = fixture();
        let out = build(&f.library, &f.previews, &f.target).unwrap();
        assert_eq!(
            (out.manifests, out.items, out.photos, out.videos, out.sidecars, out.other, out.local, out.previews),
            (2, 5, 2, 1, 1, 1, 2, 2)
        );
        assert!(out.broken.is_empty());
        let r = rows(&f.target);
        let get = |suffix: &str| r.iter().find(|x| x.0.ends_with(suffix)).unwrap().clone();
        let photo = get("CO7A9123.CR3");
        assert_eq!(photo.1, "2026/07");
        assert_eq!(photo.4, r#"["2026/07/EOSR/photos/2026-07-30/CO7A9123.xmp"]"#);
        assert_eq!((photo.5, photo.6), (1, 1));
        let video = get("CO7A9140.MP4");
        assert_eq!((video.4.as_str(), video.5, video.6), ("[]", 0, 1));
        let orphan = get("LOST.xmp");
        assert_eq!((orphan.3.as_str(), orphan.5, orphan.6), ("sidecar", 1, 0));
        let scan = get("scan1.tif");
        assert_eq!((scan.1.as_str(), scan.2.as_deref()), ("2012/scans Peter", None));

        let c = Connection::open(&f.target).unwrap();
        let n: (String, Option<String>, String, i64) = c
            .query_row(
                "SELECT folder, month, generated_by, n_files FROM folders WHERE contract_version = '1.1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(n, ("2012/scans Peter".into(), None, "raven".into(), 1));
    }

    #[test]
    fn the_result_reads_with_db_like_the_cli_index() {
        let f = fixture();
        build(&f.library, &f.previews, &f.target).unwrap();
        let db = Db::open(&f.target).unwrap();
        let months = db.months().unwrap();
        assert_eq!(months.iter().map(|m| m.month.as_str()).collect::<Vec<_>>(), ["2026/07", "2012/scans Peter"]);
        assert_eq!((months[0].photos, months[0].videos, months[0].previews, months[0].local), (1, 1, 2, 1));
        assert_eq!(db.items("2026/07").unwrap().len(), 2);
    }

    #[test]
    fn the_camera_reaches_the_items_the_counts_and_the_filter() {
        let f = fixture();
        build(&f.library, &f.previews, &f.target).unwrap();
        let db = Db::open(&f.target).unwrap();
        let items = db.items("2026/07").unwrap();
        let camera = |suffix: &str| items.iter().find(|i| i.path.ends_with(suffix)).unwrap().camera.clone();
        assert_eq!(camera("CO7A9123.CR3").as_deref(), Some("Canon EOS R"));
        assert_eq!(camera("CO7A9140.MP4"), None);
        // photos and videos only; "" is unknown
        assert_eq!(db.camera_counts().unwrap(), [("".to_string(), 2), ("Canon EOS R".to_string(), 1)]);
        let f1 = crate::filter::Filter { cameras: vec!["Canon EOS R".into()], ..Default::default() };
        assert_eq!(db.filtered_months(&f1).unwrap(), [("2026/07".to_string(), 1), ("2012/scans Peter".to_string(), 0)]);
        assert_eq!(db.filtered_items("2026/07", &f1).unwrap().len(), 1);
        let unknown = crate::filter::Filter { cameras: vec!["".into()], ..Default::default() };
        assert_eq!(db.filtered_items("2026/07", &unknown).unwrap()[0].kind, "video");
    }

    #[test]
    fn broken_manifests_are_skipped_and_named() {
        let f = fixture();
        let bad = f.library.join("2016/12").join(MANIFEST);
        fs::create_dir_all(bad.parent().unwrap()).unwrap();
        fs::write(&bad, b"{not json").unwrap();
        manifest(&f, "2016/11", json!({"folder": "2016/11"})); // no `files`
        let out = build(&f.library, &f.previews, &f.target).unwrap();
        assert_eq!(out.manifests, 4);
        assert_eq!(out.items, 5);
        assert_eq!(out.broken.len(), 2);
        assert!(out.broken[0].starts_with("2016/11: "));
        assert!(out.broken[1].starts_with("2016/12: "));
    }

    #[test]
    fn excluded_and_non_year_folders_are_ignored() {
        let f = fixture();
        manifest(&f, "2026/@eaDir", json!({"files": [{"path": "2026/@eaDir/x.jpg", "kind": "photo"}]}));
        manifest(&f, "Geboorteboek/2020", json!({"files": [{"path": "x.jpg", "kind": "photo"}]}));
        touch(&f.library.join("2026").join(MANIFEST)); // a manifest directly in a year isn't one
        let paths = manifest_paths(&f.library).unwrap();
        let rel: Vec<_> = paths.iter().map(|p| p.strip_prefix(&f.library).unwrap().to_string_lossy().into_owned()).collect();
        assert_eq!(rel, ["2012/scans Peter/.manifest.json", "2026/07/.manifest.json"]);
    }

    #[test]
    fn a_missing_working_copy_is_an_error() {
        let f = fixture();
        let err = build(&f.library.join("nope"), &f.previews, &f.target).unwrap_err();
        assert!(err.starts_with("werkkopie niet gevonden"), "{err}");
        assert!(!f.target.exists());
    }

    #[test]
    fn a_rebuild_replaces_the_index_and_a_failed_one_leaves_it() {
        let f = fixture();
        build(&f.library, &f.previews, &f.target).unwrap();
        // a reader with the old file open keeps its rows across the rename
        let reader = Db::open(&f.target).unwrap();
        fs::remove_dir_all(f.library.join("2012")).unwrap();
        let out = build(&f.library, &f.previews, &f.target).unwrap();
        assert_eq!(out.manifests, 1);
        assert_eq!(rows(&f.target).len(), 4);
        assert_eq!(reader.months().unwrap().len(), 2);

        // a temporary file in the way is a directory: the build fails, the index stays
        let tmp = f.target.with_file_name(format!("{INDEX_FILE}.tmp"));
        fs::create_dir_all(tmp.join("blocker")).unwrap();
        assert!(build(&f.library, &f.previews, &f.target).is_err());
        assert_eq!(rows(&f.target).len(), 4);
    }

    #[derive(Default)]
    struct Recorder {
        done: Mutex<Vec<IndexState>>,
    }

    impl IndexSink for Recorder {
        fn done(&self, s: &IndexState) {
            self.done.lock().unwrap().push(s.clone());
        }
    }

    fn job(f: &Fixture) -> IndexJob {
        IndexJob { library: f.library.clone(), previews: f.previews.clone(), target: f.target.clone() }
    }

    #[test]
    fn stale_when_a_manifest_is_newer_added_or_removed() {
        let f = fixture();
        let r = IndexRunner::default();
        assert!(r.stale(&job(&f)), "no index yet");
        r.request(job(&f), Arc::new(Recorder::default())).unwrap().join().unwrap();
        assert!(!r.stale(&job(&f)), "nothing changed");

        // a synced manifest keeps the NAS's (older) time: still a change
        let m = f.library.join("2026/07").join(MANIFEST);
        let old = std::time::SystemTime::now() - std::time::Duration::from_secs(86_400 * 30);
        fs::File::options().write(true).open(&m).unwrap().set_modified(old).unwrap();
        assert!(r.stale(&job(&f)), "older time on a manifest");
        r.request(job(&f), Arc::new(Recorder::default())).unwrap().join().unwrap();
        assert!(!r.stale(&job(&f)));

        manifest(&f, "2016/12", json!({"files": []}));
        assert!(r.stale(&job(&f)), "added");
        r.request(job(&f), Arc::new(Recorder::default())).unwrap().join().unwrap();
        fs::remove_dir_all(f.library.join("2016")).unwrap();
        assert!(r.stale(&job(&f)), "removed");

        r.request(job(&f), Arc::new(Recorder::default())).unwrap().join().unwrap();
        fs::remove_file(&f.target).unwrap();
        assert!(r.stale(&job(&f)), "index file gone");
    }

    #[test]
    fn a_request_during_a_build_runs_once_more_afterwards() {
        let f = fixture();
        let r = IndexRunner::default();
        r.inner.lock().unwrap().state.running = true; // as if a build were running
        let rec = Arc::new(Recorder::default());
        assert!(r.request(job(&f), rec.clone()).is_none());
        assert!(r.request(job(&f), rec.clone()).is_none(), "two requests queue one build");
        assert!(r.inner.lock().unwrap().queued);
        // the running build finishes, sees the queued request and builds again
        let me = r.clone();
        let rec2 = rec.clone();
        std::thread::spawn(move || me.run(&job(&f), rec2.as_ref())).join().unwrap();
        let done = rec.done.lock().unwrap();
        assert_eq!(done.len(), 2, "this build and the queued one");
        assert!(done[0].running && !done[1].running);
        assert_eq!(done[1].builds, 2);
        assert!(done[1].outcome.as_ref().unwrap().items == 5 && done[1].error.is_none());
        assert!(!r.status().running);
    }

    #[test]
    fn a_failed_build_reports_the_error_and_keeps_the_last_outcome() {
        let f = fixture();
        let r = IndexRunner::default();
        let rec = Arc::new(Recorder::default());
        r.request(job(&f), rec.clone()).unwrap().join().unwrap();
        let mut j = job(&f);
        j.library = f.library.join("nope");
        r.request(j, rec.clone()).unwrap().join().unwrap();
        let s = r.status();
        assert!(s.error.unwrap().starts_with("werkkopie niet gevonden"));
        assert_eq!(s.outcome.unwrap().items, 5);
        assert_eq!(s.builds, 2);
    }

    #[test]
    fn marking_previews_sets_their_flag_without_a_rebuild() {
        let f = fixture();
        build(&f.library, &f.previews, &f.target).unwrap();
        let scan = "2012/scans Peter/scan1.tif";
        let db = Db::open(&f.target).unwrap();
        assert_eq!(db.missing_previews().unwrap(), 1);
        assert_eq!(mark_previews(&f.target, &[scan, "2026/07/not-indexed.JPG"]).unwrap(), 1);
        assert_eq!(mark_previews(&f.target, &[scan]).unwrap(), 0);
        assert!(db.items("2012/scans Peter").unwrap().iter().all(|i| i.preview));
        assert_eq!(db.missing_previews().unwrap(), 0);
    }

    #[test]
    fn img_numbers_in_their_forms() {
        assert_eq!(img_number("2026/07/iPhone-videos/IMG_1234.MOV"), Some(1234));
        assert_eq!(img_number("2026/07/iPhone-photos/IMG_E1234.HEIC"), Some(1234));
        assert_eq!(img_number("2022/02/iPhone-videos/2022-02-20_16-47-13_IMG_5925.MOV"), Some(5925));
        assert_eq!(img_number("2026/07/EOSR/photos/CO7A9123.CR3"), None);
        assert_eq!(img_number("2026/07/x/IMG_123.MOV"), None);
        assert_eq!(img_number("2026/07/x/IMG_1234_1.MOV"), None);
    }

    #[test]
    fn live_photos_pair_by_month_number_and_time() {
        let r = |p: &str, m: &str, k: &str, t: &str| (p.to_string(), m.to_string(), k.to_string(), Some(t.to_string()));
        let rows = vec![
            r("2026/07/p/IMG_0001.HEIC", "2026/07", "photo", "2026-07-30T10:00:00+02:00"),
            r("2026/07/v/IMG_0001.MOV", "2026/07", "video", "2026-07-30T10:00:02+02:00"),   // 2 s: live
            r("2026/07/p/IMG_0002.HEIC", "2026/07", "photo", "2026-07-30T10:01:00+02:00"),
            r("2026/07/v/IMG_0002.MOV", "2026/07", "video", "2026-07-30T10:01:04+02:00"),   // 4 s: not
            r("2026/07/v/IMG_0003.MP4", "2026/07", "video", "2026-07-30T10:02:00+02:00"),   // .MP4: not
            r("2026/07/p/IMG_0003.HEIC", "2026/07", "photo", "2026-07-30T10:02:00+02:00"),
            r("2026/08/v/IMG_0001.MOV", "2026/08", "video", "2026-07-30T10:00:01+02:00"),   // other month
            // the number again after IMG_9999, far apart: the near one wins
            r("2026/07/p/IMG_0004.HEIC", "2026/07", "photo", "2026-07-01T09:00:00+02:00"),
            r("2026/07/p/IMG_E0004.HEIC", "2026/07", "photo", "2026-07-30T11:00:01+02:00"),
            r("2026/07/v/IMG_0004.MOV", "2026/07", "video", "2026-07-30T11:00:00+02:00"),
        ];
        let mut got = live_pairs(&rows);
        got.sort();
        assert_eq!(got, vec![
            ("2026/07/v/IMG_0001.MOV".to_string(), "2026/07/p/IMG_0001.HEIC".to_string()),
            ("2026/07/v/IMG_0004.MOV".into(), "2026/07/p/IMG_E0004.HEIC".into()),
        ]);
    }

    #[test]
    fn a_built_index_gives_each_live_video_its_photo() {
        let f = fixture();
        manifest(&f, "2026/08", json!({"contract_version": "1.2", "folder": "2026/08", "month": "2026/08", "files": [
            {"path": "2026/08/iPhone-photos/IMG_0001.HEIC", "size": 1, "hash": "sha256:p", "kind": "photo", "taken": "2026-08-01T10:00:00+02:00"},
            {"path": "2026/08/iPhone-videos/IMG_0001.MOV", "size": 1, "hash": "sha256:v", "kind": "video", "taken": "2026-08-01T10:00:01+02:00"},
            {"path": "2026/08/iPhone-videos/IMG_0002.MOV", "size": 1, "hash": "sha256:w", "kind": "video", "taken": "2026-08-01T11:00:00+02:00"}
        ]}));
        build(&f.library, &f.previews, &f.target).unwrap();
        let items = Db::open(&f.target).unwrap().items("2026/08").unwrap();
        let live: Vec<_> = items.iter().map(|i| (i.path.as_str(), i.live.as_deref())).collect();
        assert!(live.contains(&("2026/08/iPhone-videos/IMG_0001.MOV", Some("2026/08/iPhone-photos/IMG_0001.HEIC"))));
        assert!(live.contains(&("2026/08/iPhone-videos/IMG_0002.MOV", None)));
        assert!(live.contains(&("2026/08/iPhone-photos/IMG_0001.HEIC", None)));
    }
}
