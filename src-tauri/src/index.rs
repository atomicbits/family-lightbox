//! Builds the app's own index from the manifests in the working copy (F008/S01). Same tables and
//! rows as `media index` in the CLI (data contract 7), but in its own file, which only the app
//! writes, so `db.rs` reads it unchanged and the CLI's `index.sqlite` is left alone.

use crate::sync::excluded;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

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
";

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
    tx.commit().map_err(err)?;
    conn.close().map_err(|(_, e)| err(e))?;
    Ok(out)
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
                     "taken": "2026-07-30T16:58:12+02:00", "taken_from": "exif", "sidecars": [x]},
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
}
