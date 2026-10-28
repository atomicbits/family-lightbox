//! Which previews the app has to make (F012/S02): for every photo and video in the app's index,
//! whether its preview is fine, can be adopted, will come with the sync, has to be made, or failed
//! before. The app keeps its own record of which hash each preview was made from, in a file only
//! the app writes; the CLI's `index.sqlite` is not read or written.

use rusqlite::{Connection, OpenFlags, params};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const RECORD_FILE: &str = "familylightbox-previews.sqlite";

const SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS previews (path TEXT PRIMARY KEY, hash TEXT NOT NULL,
        made_at INTEGER NOT NULL, error TEXT);
";

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum State {
    /// Preview and thumbnail are local and were made from this hash.
    Fine,
    /// Both are local without a record (made by the CLI or another Mac): recorded, not made.
    Adopt,
    /// Not local, but both are on the NAS: the sync of feature 7 brings them.
    Copy,
    /// Missing locally and on the NAS, or made from another hash.
    Make,
    /// Making it failed before with this same hash.
    Failed,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Wanted {
    pub path: String,
    /// `coalesce(month, folder)`, the key the grid groups by.
    pub month: String,
    pub kind: String,
    pub hash: String,
    pub taken: Option<String>,
    pub state: State,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct Counts {
    pub fine: usize,
    pub adopted: usize,
    pub copy: usize,
    pub make: usize,
    pub failed: usize,
}

#[derive(Debug, Default)]
pub struct Plan {
    /// Everything that is not fine or adopted, in the order to work on it.
    pub todo: Vec<Wanted>,
    pub counts: Counts,
    pub millis: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct Entry {
    hash: String,
    made_at: i64,
    error: Option<String>,
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

/// The app's own record: which hash each preview was made from, and what failed.
pub struct Record {
    conn: Connection,
}

impl Record {
    pub fn open(path: &Path) -> Result<Record, String> {
        let err = |e: rusqlite::Error| format!("previewregister openen mislukt: {e}");
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("map {} maken mislukt: {e}", dir.display()))?;
        }
        let conn = Connection::open(path).map_err(err)?;
        conn.busy_timeout(Duration::from_secs(3)).map_err(err)?;
        conn.execute_batch(SCHEMA).map_err(err)?;
        Ok(Record { conn })
    }

    fn all(&self) -> Result<HashMap<String, Entry>, String> {
        let err = |e: rusqlite::Error| format!("previewregister lezen mislukt: {e}");
        let mut stmt = self.conn.prepare("SELECT path, hash, made_at, error FROM previews").map_err(err)?;
        let rows = stmt
            .query_map([], |r| Ok((r.get(0)?, Entry { hash: r.get(1)?, made_at: r.get(2)?, error: r.get(3)? })))
            .map_err(err)?;
        rows.collect::<Result<_, _>>().map_err(err)
    }

    fn put(&mut self, rows: &[(&str, &str, Option<&str>)]) -> Result<(), String> {
        let err = |e: rusqlite::Error| format!("previewregister schrijven mislukt: {e}");
        let tx = self.conn.transaction().map_err(err)?;
        {
            let mut stmt = tx.prepare("INSERT OR REPLACE INTO previews VALUES (?1, ?2, ?3, ?4)").map_err(err)?;
            let at = now();
            for (path, hash, error) in rows {
                stmt.execute(params![path, hash, at, error]).map_err(err)?;
            }
        }
        tx.commit().map_err(err)
    }

    /// The preview of `path` was made from `hash` just now.
    pub fn made(&mut self, path: &str, hash: &str) -> Result<(), String> {
        self.put(&[(path, hash, None)])
    }

    /// Making the preview of `path` from `hash` failed; not tried again until the hash changes.
    pub fn failed(&mut self, path: &str, hash: &str, error: &str) -> Result<(), String> {
        self.put(&[(path, hash, Some(error))])
    }

    /// Forget the failures, so they are tried again (by hand, S04).
    pub fn clear_failures(&mut self) -> Result<usize, String> {
        self.conn
            .execute("DELETE FROM previews WHERE error IS NOT NULL", [])
            .map_err(|e| format!("previewregister schrijven mislukt: {e}"))
    }
}

/// Which file names a folder holds, listed once per folder. A folder that can't be read (not
/// there, or a year this user may not read on the NAS) holds nothing.
#[derive(Default)]
struct Listing {
    folders: HashMap<PathBuf, HashSet<OsString>>,
}

impl Listing {
    fn has(&mut self, file: &Path) -> bool {
        let (Some(dir), Some(name)) = (file.parent(), file.file_name()) else { return false };
        self.folders
            .entry(dir.to_path_buf())
            .or_insert_with(|| {
                fs::read_dir(dir)
                    .map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.file_name()).collect())
                    .unwrap_or_default()
            })
            .contains(name)
    }

    fn has_both(&mut self, root: &Path, path: &str) -> bool {
        let (out, thumb) = names(root, path);
        self.has(&out) && self.has(&thumb)
    }
}

/// `<root>/<path>.jpg` and `<root>/<path>.t.jpg` (data contract 4).
pub fn names(root: &Path, path: &str) -> (PathBuf, PathBuf) {
    (root.join(format!("{path}.jpg")), root.join(format!("{path}.t.jpg")))
}

fn modified(file: &Path) -> Option<i64> {
    let t = fs::metadata(file).ok()?.modified().ok()?;
    Some(t.duration_since(UNIX_EPOCH).ok()?.as_secs() as i64)
}

/// path, month, kind, hash, taken
type Row = (String, String, String, String, Option<String>);

/// The photos and videos in the app's index.
fn indexed(index: &Path) -> Result<Vec<Row>, String> {
    let err = |e: rusqlite::Error| format!("index lezen mislukt: {e}");
    let conn = Connection::open_with_flags(index, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX)
        .map_err(|e| format!("index openen mislukt: {e}"))?;
    conn.busy_timeout(Duration::from_secs(3)).map_err(err)?;
    let mut stmt = conn
        .prepare(
            "SELECT path, coalesce(month, folder), kind, coalesce(hash, ''), taken
             FROM items WHERE kind IN ('photo', 'video')",
        )
        .map_err(err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))
        .map_err(err)?;
    rows.collect::<Result<_, _>>().map_err(err)
}

/// One item's state, from what is local, what is on the NAS and the record.
fn decide(
    hash: &str,
    local: bool,
    on_nas: impl FnOnce() -> bool,
    entry: Option<&Entry>,
    local_modified: impl FnOnce() -> Option<i64>,
) -> State {
    match entry {
        Some(e) if e.error.is_some() && e.hash == hash => State::Failed,
        Some(e) if local && e.error.is_none() && e.hash == hash => State::Fine,
        // another Mac made it again after we recorded ours, and the sync brought it
        Some(e) if local && local_modified().is_some_and(|t| t > e.made_at) => State::Adopt,
        None if local => State::Adopt,
        _ if local => State::Make,
        _ if on_nas() => State::Copy,
        _ => State::Make,
    }
}

/// Puts the month on screen first, then the newest months first; within a month, by path.
pub fn order(todo: &mut [Wanted], focus: Option<&str>) {
    todo.sort_by(|a, b| {
        let fa = Some(a.month.as_str()) == focus;
        let fb = Some(b.month.as_str()) == focus;
        fb.cmp(&fa).then_with(|| b.month.cmp(&a.month)).then_with(|| a.path.cmp(&b.path))
    });
}

/// The plan for the whole library. Adopted previews are recorded right away; nothing is made.
/// `nas` is the `Previews` share when it is connected, checked only for what isn't local.
pub fn plan(
    index: &Path,
    record: &mut Record,
    previews: &Path,
    nas: Option<&Path>,
    focus: Option<&str>,
) -> Result<Plan, String> {
    let started = Instant::now();
    let known = record.all()?;
    let (mut local, mut remote) = (Listing::default(), Listing::default());
    let mut out = Plan::default();
    let mut adopt: Vec<(String, String)> = Vec::new();
    for (path, month, kind, hash, taken) in indexed(index)? {
        let have = local.has_both(previews, &path);
        let state = decide(
            &hash,
            have,
            || nas.is_some_and(|n| remote.has_both(n, &path)),
            known.get(&path),
            || modified(&names(previews, &path).0),
        );
        match state {
            State::Fine => out.counts.fine += 1,
            State::Adopt => {
                out.counts.adopted += 1;
                adopt.push((path, hash));
                continue;
            }
            State::Copy => out.counts.copy += 1,
            State::Make => out.counts.make += 1,
            State::Failed => out.counts.failed += 1,
        }
        if state != State::Fine {
            out.todo.push(Wanted { path, month, kind, hash, taken, state });
        }
    }
    let rows: Vec<_> = adopt.iter().map(|(p, h)| (p.as_str(), h.as_str(), None)).collect();
    record.put(&rows)?;
    order(&mut out.todo, focus);
    out.millis = started.elapsed().as_millis() as u64;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        _dir: tempfile::TempDir,
        index: PathBuf,
        record: PathBuf,
        previews: PathBuf,
        nas: PathBuf,
    }

    /// An index with these photos and videos: (path, month, hash).
    fn fixture(items: &[(&str, Option<&str>, &str)]) -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let index = dir.path().join("index.sqlite");
        let conn = Connection::open(&index).unwrap();
        conn.execute_batch(
            "CREATE TABLE items (path TEXT PRIMARY KEY, folder TEXT, month TEXT, size INTEGER,
                hash TEXT, taken TEXT, taken_from TEXT, kind TEXT, source TEXT, sidecars TEXT,
                local INTEGER, preview INTEGER);",
        )
        .unwrap();
        for (path, month, hash) in items {
            let folder = path.rsplit_once('/').map_or("", |(d, _)| d);
            let kind = if path.ends_with(".MOV") { "video" } else { "photo" };
            conn.execute(
                "INSERT INTO items VALUES (?1, ?2, ?3, 1, ?4, NULL, NULL, ?5, NULL, '[]', 0, 0)",
                params![path, folder, month, hash, kind],
            )
            .unwrap();
        }
        conn.execute("INSERT INTO items VALUES ('2026/07/a/notes.txt', '2026/07/a', '2026/07', 1, 'h', NULL, NULL, 'other', NULL, '[]', 0, 0)", []).unwrap();
        let (previews, nas, record) = (dir.path().join("Previews"), dir.path().join("nas"), dir.path().join("rec.sqlite"));
        Fixture { index, record, previews, nas, _dir: dir }
    }

    fn put(root: &Path, path: &str) {
        for f in [names(root, path).0, names(root, path).1] {
            fs::create_dir_all(f.parent().unwrap()).unwrap();
            fs::write(f, b"jpg").unwrap();
        }
    }

    fn states(plan: &Plan) -> Vec<(&str, State)> {
        plan.todo.iter().map(|w| (w.path.as_str(), w.state)).collect()
    }

    #[test]
    fn every_state() {
        let f = fixture(&[
            ("2026/07/a/fine.JPG", Some("2026/07"), "h1"),
            ("2026/07/a/adopt.JPG", Some("2026/07"), "h2"),
            ("2026/07/a/copy.JPG", Some("2026/07"), "h3"),
            ("2026/07/a/make.MOV", Some("2026/07"), "h4"),
            ("2026/07/a/only-preview.JPG", Some("2026/07"), "h5"),
            ("2026/07/a/failed.JPG", Some("2026/07"), "h6"),
        ]);
        let mut rec = Record::open(&f.record).unwrap();
        put(&f.previews, "2026/07/a/fine.JPG");
        rec.made("2026/07/a/fine.JPG", "h1").unwrap();
        put(&f.previews, "2026/07/a/adopt.JPG");
        put(&f.nas, "2026/07/a/copy.JPG");
        fs::write(names(&f.previews, "2026/07/a/only-preview.JPG").0, b"jpg").unwrap(); // no thumbnail
        rec.failed("2026/07/a/failed.JPG", "h6", "geen beeld").unwrap();

        let plan = plan(&f.index, &mut rec, &f.previews, Some(&f.nas), None).unwrap();
        assert_eq!(plan.counts, Counts { fine: 1, adopted: 1, copy: 1, make: 2, failed: 1 });
        assert_eq!(
            states(&plan),
            vec![
                ("2026/07/a/copy.JPG", State::Copy),
                ("2026/07/a/failed.JPG", State::Failed),
                ("2026/07/a/make.MOV", State::Make),
                ("2026/07/a/only-preview.JPG", State::Make),
            ]
        );
        let make = plan.todo.iter().find(|w| w.path.ends_with("make.MOV")).unwrap();
        assert_eq!((make.kind.as_str(), make.hash.as_str(), make.month.as_str()), ("video", "h4", "2026/07"));

        // adopted once, then fine
        let again = super::plan(&f.index, &mut rec, &f.previews, Some(&f.nas), None).unwrap();
        assert_eq!((again.counts.fine, again.counts.adopted), (2, 0));
    }

    #[test]
    fn a_changed_hash_is_made_again_and_a_failure_tried_again() {
        let f = fixture(&[("2026/07/a/x.JPG", Some("2026/07"), "new"), ("2026/07/a/y.JPG", Some("2026/07"), "new")]);
        let mut rec = Record::open(&f.record).unwrap();
        put(&f.previews, "2026/07/a/x.JPG");
        rec.made("2026/07/a/x.JPG", "old").unwrap();
        rec.failed("2026/07/a/y.JPG", "old", "kapot").unwrap();
        // the preview is older than the record: ours, made from the old original
        let past = SystemTime::now() - Duration::from_secs(3600);
        fs::File::options().write(true).open(names(&f.previews, "2026/07/a/x.JPG").0).unwrap().set_modified(past).unwrap();
        let plan = plan(&f.index, &mut rec, &f.previews, None, None).unwrap();
        assert_eq!(states(&plan), vec![("2026/07/a/x.JPG", State::Make), ("2026/07/a/y.JPG", State::Make)]);
    }

    #[test]
    fn a_newer_preview_from_another_mac_is_adopted() {
        let f = fixture(&[("2026/07/a/x.JPG", Some("2026/07"), "new")]);
        let mut rec = Record::open(&f.record).unwrap();
        rec.conn.execute("INSERT INTO previews VALUES ('2026/07/a/x.JPG', 'old', 1000, NULL)", []).unwrap();
        put(&f.previews, "2026/07/a/x.JPG"); // written now, long after 1000
        let plan = plan(&f.index, &mut rec, &f.previews, None, None).unwrap();
        assert_eq!((plan.counts.adopted, plan.todo.len()), (1, 0));
    }

    #[test]
    fn without_the_nas_what_isnt_local_is_made() {
        let f = fixture(&[("2026/07/a/x.JPG", Some("2026/07"), "h")]);
        put(&f.nas, "2026/07/a/x.JPG");
        let mut rec = Record::open(&f.record).unwrap();
        let plan = plan(&f.index, &mut rec, &f.previews, None, None).unwrap();
        assert_eq!(states(&plan), vec![("2026/07/a/x.JPG", State::Make)]);
    }

    #[test]
    fn an_unreadable_year_on_the_nas_counts_as_not_there() {
        let f = fixture(&[("2019/05/a/x.JPG", Some("2019/05"), "h")]);
        put(&f.nas, "2019/05/a/x.JPG");
        let year = f.nas.join("2019");
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&year, fs::Permissions::from_mode(0o000)).unwrap();
        let mut rec = Record::open(&f.record).unwrap();
        let result = plan(&f.index, &mut rec, &f.previews, Some(&f.nas), None);
        fs::set_permissions(&year, fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(states(&result.unwrap()), vec![("2019/05/a/x.JPG", State::Make)]);
    }

    #[test]
    fn the_month_on_screen_first_then_the_newest() {
        let f = fixture(&[
            ("2012/scans/a.JPG", None, "h"),
            ("2019/05/a/b.JPG", Some("2019/05"), "h"),
            ("2026/07/a/c.JPG", Some("2026/07"), "h"),
            ("2021/01/a/d.JPG", Some("2021/01"), "h"),
        ]);
        let mut rec = Record::open(&f.record).unwrap();
        let plan = plan(&f.index, &mut rec, &f.previews, None, Some("2019/05")).unwrap();
        let months: Vec<_> = plan.todo.iter().map(|w| w.month.as_str()).collect();
        assert_eq!(months, vec!["2019/05", "2026/07", "2021/01", "2012/scans"]);
    }

    #[test]
    fn clearing_failures_tries_them_again() {
        let f = fixture(&[("2026/07/a/x.JPG", Some("2026/07"), "h")]);
        let mut rec = Record::open(&f.record).unwrap();
        rec.failed("2026/07/a/x.JPG", "h", "kapot").unwrap();
        assert_eq!(plan(&f.index, &mut rec, &f.previews, None, None).unwrap().counts.failed, 1);
        assert_eq!(rec.clear_failures().unwrap(), 1);
        assert_eq!(plan(&f.index, &mut rec, &f.previews, None, None).unwrap().counts.make, 1);
    }
}
