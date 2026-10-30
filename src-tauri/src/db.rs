//! Leest de index (datacontract sectie 7): tabel `items` met één regel per bestand uit de
//! manifesten. Sinds F008 de eigen index van de app, die `index.rs` bouwt met hetzelfde schema.

use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use std::path::Path;
use std::time::Duration;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct MonthSummary {
    /// `2026/07`, of de mapnaam als het manifest geen maand heeft (`2012/scans Peter`).
    pub month: String,
    pub photos: i64,
    pub videos: i64,
    /// Hoeveel van die foto's en video's een preview hebben op deze laptop.
    pub previews: i64,
    /// Hoeveel originelen lokaal staan.
    pub local: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Item {
    pub path: String,
    pub folder: String,
    pub kind: String,
    pub taken: Option<String>,
    pub taken_from: Option<String>,
    pub source: Option<String>,
    pub size: i64,
    pub hash: String,
    pub local: bool,
    pub preview: bool,
    /// For the video of a Live Photo: the path of its photo (F017/S01).
    pub live: Option<String>,
    /// Make and model from the manifest, or none (F002/S01).
    pub camera: Option<String>,
}

/// The columns of an `Item`, from `items` joined with `live_photos` as `lp` and `cameras` as `cam`
/// where the index has them (see `Db::joins`).
fn item_columns(has_live: bool, has_cameras: bool) -> String {
    let live = if has_live { "lp.photo" } else { "NULL" };
    let camera = if has_cameras { "cam.camera" } else { "NULL" };
    format!("items.path, folder, kind, taken, taken_from, source, size, hash, local, preview, {live}, {camera}")
}

fn item_from(r: &rusqlite::Row, at: usize) -> rusqlite::Result<Item> {
    Ok(Item {
        path: r.get(at)?,
        folder: r.get(at + 1)?,
        kind: r.get(at + 2)?,
        taken: r.get(at + 3)?,
        taken_from: r.get(at + 4)?,
        source: r.get(at + 5)?,
        size: r.get(at + 6)?,
        hash: r.get(at + 7)?,
        local: r.get::<_, i64>(at + 8)? != 0,
        preview: r.get::<_, i64>(at + 9)? != 0,
        live: r.get(at + 10)?,
        camera: r.get(at + 11)?,
    })
}

#[derive(Debug)]
pub struct Db {
    conn: Connection,
    /// An index built before F017 has no `live_photos` table, before F002/S01 no `cameras`.
    has_live: bool,
    has_cameras: bool,
}

impl Db {
    /// The joins that `item_columns` reads from.
    fn joins(&self) -> String {
        let mut j = String::new();
        if self.has_live {
            j.push_str(" LEFT JOIN live_photos lp ON lp.video = items.path");
        }
        if self.has_cameras {
            j.push_str(" LEFT JOIN cameras cam ON cam.path = items.path");
        }
        j
    }

    /// Per source, the paths of its originals in ordinary month folders (F018/S03), for
    /// `sources::layouts`.
    pub fn source_paths(&self) -> Result<Vec<(String, String)>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT source, path FROM items WHERE source IS NOT NULL AND kind IN ('photo', 'video') AND folder GLOB '[0-9][0-9][0-9][0-9]/[0-9][0-9]'")
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    /// The sources a camera's photos and videos are under, with how many each, most first
    /// (F018/S03).
    pub fn sources_of_camera(&self, camera: &str) -> Result<Vec<(String, i64)>, String> {
        if !self.has_cameras {
            return Ok(vec![]);
        }
        let mut stmt = self
            .conn
            .prepare("SELECT source, count(*) FROM items JOIN cameras cam ON cam.path = items.path WHERE cam.camera = ?1 AND source IS NOT NULL GROUP BY 1 ORDER BY 2 DESC, 1")
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map([camera], |r| Ok((r.get(0)?, r.get(1)?))).map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    /// The cameras of the photos and videos with how many each and when last used (the latest
    /// `taken`), most recent first; "" for unknown, always last (F002/S04).
    pub fn camera_counts(&self) -> Result<Vec<(String, i64, Option<String>)>, String> {
        if !self.has_cameras {
            return Ok(vec![]);
        }
        let mut stmt = self
            .conn
            .prepare(
                "SELECT coalesce(cam.camera, '') AS c, count(*), max(taken) AS last FROM items
                 LEFT JOIN cameras cam ON cam.path = items.path
                 WHERE kind IN ('photo', 'video') GROUP BY 1 ORDER BY c = '', last IS NULL, last DESC, 2 DESC, 1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    pub fn open(path: &Path) -> Result<Db, String> {
        if !path.is_file() {
            return Err(format!(
                "nog geen index: {}\nde app maakt ze uit de manifesten in de werkkopie",
                path.display()
            ));
        }
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| format!("index openen mislukt: {e}"))?;
        // the app may be replacing the index at this moment (the rename is atomic)
        conn.busy_timeout(Duration::from_secs(3)).map_err(|e| e.to_string())?;
        let table = |name: &str| -> Result<bool, String> {
            conn.query_row("SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1", [name], |r| r.get::<_, i64>(0))
                .map(|n| n > 0)
                .map_err(|e| e.to_string())
        };
        let (has_live, has_cameras) = (table("live_photos")?, table("cameras")?);
        Ok(Db { conn, has_live, has_cameras })
    }

    /// Alle maanden (nieuwste eerst) met tellingen; alleen foto's en video's tellen mee.
    pub fn months(&self) -> Result<Vec<MonthSummary>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT coalesce(month, folder) AS m,
                        sum(kind = 'photo'), sum(kind = 'video'), sum(preview), sum(local)
                 FROM items WHERE kind IN ('photo', 'video')
                 GROUP BY m ORDER BY m DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(MonthSummary {
                    month: r.get(0)?,
                    photos: r.get(1)?,
                    videos: r.get(2)?,
                    previews: r.get(3)?,
                    local: r.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    /// How many photos and videos have no preview on this Mac (F012/S04).
    pub fn missing_previews(&self) -> Result<i64, String> {
        self.conn
            .query_row("SELECT count(*) FROM items WHERE kind IN ('photo', 'video') AND NOT preview", [], |r| r.get(0))
            .map_err(|e| e.to_string())
    }

    /// How many photos and videos with these hashes each month has (F014/S02): the hidden ones,
    /// to leave out of the counts. One pass over the index.
    pub fn count_per_month(&self, hashes: &std::collections::HashSet<String>) -> Result<std::collections::HashMap<String, i64>, String> {
        let mut out = std::collections::HashMap::new();
        if hashes.is_empty() {
            return Ok(out);
        }
        let mut stmt = self
            .conn
            .prepare("SELECT coalesce(month, folder), hash FROM items WHERE kind IN ('photo', 'video')")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)))
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (month, hash) = row.map_err(|e| e.to_string())?;
            if hash.is_some_and(|h| hashes.contains(&h)) {
                *out.entry(month).or_insert(0) += 1;
            }
        }
        Ok(out)
    }

    /// Each month with how many of its photos and videos match the filter (F002/S02), newest
    /// first; months with none are kept with 0, so the month list can dim them.
    pub fn filtered_months(&self, f: &crate::filter::Filter) -> Result<Vec<(String, i64)>, String> {
        let join = self.joins();
        let live = if self.has_live { "lp.photo IS NOT NULL" } else { "0" };
        let camera = if self.has_cameras { "cam.camera" } else { "NULL" };
        let sql = format!(
            "SELECT coalesce(month, folder), coalesce(hash, ''), kind, {live}, {camera} FROM items {join}
             WHERE kind IN ('photo', 'video')"
        );
        let mut stmt = self.conn.prepare(&sql).map_err(|e| e.to_string())?;
        let mut counts: std::collections::BTreeMap<String, i64> = std::collections::BTreeMap::new();
        let rows = stmt
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, bool>(3)?, r.get::<_, Option<String>>(4)?))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (month, hash, kind, live, camera) = row.map_err(|e| e.to_string())?;
            let hit = f.matches(&crate::filter::Candidate { hash: &hash, kind: &kind, live, month: &month, camera: camera.as_deref() });
            *counts.entry(month).or_insert(0) += hit as i64;
        }
        Ok(counts.into_iter().rev().collect())
    }

    /// A month's photos and videos that match the filter.
    pub fn filtered_items(&self, month: &str, f: &crate::filter::Filter) -> Result<Vec<Item>, String> {
        Ok(self
            .items(month)?
            .into_iter()
            .filter(|i| {
                f.matches(&crate::filter::Candidate { hash: &i.hash, kind: &i.kind, live: i.live.is_some(), month, camera: i.camera.as_deref() })
            })
            .collect())
    }

    /// The photos and videos with these hashes, each with the month the grid groups it under,
    /// by date taken (F003/S04). A hash that is in several places gives each of them.
    pub fn items_by_hash(&self, hashes: &[String]) -> Result<Vec<(String, Item)>, String> {
        let join = self.joins();
        let sql = format!(
            "SELECT coalesce(month, folder), {} FROM items {join} WHERE kind IN ('photo', 'video') AND hash = ?1",
            item_columns(self.has_live, self.has_cameras)
        );
        let mut stmt = self.conn.prepare(&sql).map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for h in hashes {
            let rows = stmt
                .query_map([h], |r| Ok((r.get::<_, String>(0)?, item_from(r, 1)?)))
                .map_err(|e| e.to_string())?;
            for row in rows {
                out.push(row.map_err(|e| e.to_string())?);
            }
        }
        out.sort_by(|a, b| a.1.taken.cmp(&b.1.taken).then_with(|| a.1.path.cmp(&b.1.path)));
        Ok(out)
    }

    /// De foto's en video's van één maand, op opnametijd.
    pub fn items(&self, month: &str) -> Result<Vec<Item>, String> {
        let join = self.joins();
        let sql = format!(
            "SELECT {} FROM items {join}
             WHERE kind IN ('photo', 'video') AND coalesce(month, folder) = ?1
             ORDER BY taken, items.path",
            item_columns(self.has_live, self.has_cameras)
        );
        let mut stmt = self.conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([month], |r| item_from(r, 0)).map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Zelfde schema als `media index` in de CLI.
    fn fixture() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("index.sqlite");
        let c = Connection::open(&path).unwrap();
        c.execute_batch(
            "CREATE TABLE folders (folder TEXT PRIMARY KEY, month TEXT, contract_version TEXT,
                generated_at TEXT, generated_by TEXT, n_files INTEGER);
             CREATE TABLE items (path TEXT PRIMARY KEY, folder TEXT, month TEXT, size INTEGER,
                hash TEXT, taken TEXT, taken_from TEXT, kind TEXT, source TEXT, sidecars TEXT,
                local INTEGER, preview INTEGER);",
        )
        .unwrap();
        let rows = [
            ("2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3", "2026/07", "2026/07", "photo", "EOSR", "2026-07-30T10:00:00+02:00", 1, 1),
            ("2026/07/EOSR/videos/2026-07-30/CO7A9140.MP4", "2026/07", "2026/07", "video", "EOSR", "2026-07-30T10:05:00+02:00", 0, 1),
            ("2026/07/iPhone-photos-Lynn/IMG_0812.JPG", "2026/07", "2026/07", "photo", "iPhone-photos-Lynn", "2026-07-02T09:00:00+02:00", 1, 0),
            ("2026/07/EOSR/photos/2026-07-30/CO7A9123.xmp", "2026/07", "2026/07", "sidecar", "EOSR", "2026-07-30T10:00:00+02:00", 1, 0),
            ("2007/02.Sevilla/IMG_1.jpg", "2007/02.Sevilla", "2007/02", "photo", "", "2007-02-10T12:00:00+01:00", 0, 1),
            ("2012/scans Peter/scan1.tif", "2012/scans Peter", "", "photo", "", "2012-01-01T00:00:00+01:00", 0, 0),
        ];
        for (path, folder, month, kind, source, taken, local, preview) in rows {
            c.execute(
                "INSERT INTO items VALUES (?1, ?2, ?3, 100, 'sha256:x', ?4, 'exif', ?5, ?6, '[]', ?7, ?8)",
                rusqlite::params![
                    path,
                    folder,
                    if month.is_empty() { None } else { Some(month) },
                    taken,
                    kind,
                    if source.is_empty() { None } else { Some(source) },
                    local,
                    preview
                ],
            )
            .unwrap();
        }
        (dir, path)
    }

    #[test]
    fn months_count_photos_videos_previews_and_group_month_null_by_folder() {
        let (_d, p) = fixture();
        let m = Db::open(&p).unwrap().months().unwrap();
        assert_eq!(m.iter().map(|x| x.month.as_str()).collect::<Vec<_>>(), ["2026/07", "2012/scans Peter", "2007/02"]);
        assert_eq!(m[0], MonthSummary { month: "2026/07".into(), photos: 2, videos: 1, previews: 2, local: 2 });
        assert_eq!(m[2].photos, 1);
    }

    #[test]
    fn items_are_sorted_by_taken_and_skip_sidecars() {
        let (_d, p) = fixture();
        let items = Db::open(&p).unwrap().items("2026/07").unwrap();
        assert_eq!(items.len(), 3);
        assert!(items[0].path.ends_with("IMG_0812.JPG"));
        assert_eq!(items[1].kind, "photo");
        assert!(items[1].local && items[1].preview);
        assert_eq!(items[2].kind, "video");
        assert!(!items[2].local);
        assert_eq!(items[0].source.as_deref(), Some("iPhone-photos-Lynn"));
        assert!(Db::open(&p).unwrap().items("1999/01").unwrap().is_empty());
    }

    #[test]
    fn missing_index_gives_a_helpful_error() {
        let err = Db::open(Path::new("/nope/index.sqlite")).unwrap_err();
        assert!(err.starts_with("nog geen index"));
    }

    #[test]
    fn items_by_hash_give_their_month_by_date_taken_without_sidecars() {
        let (_d, p) = fixture();
        let c = Connection::open(&p).unwrap();
        c.execute("UPDATE items SET hash = 'sha256:a' WHERE path LIKE '2026/07/EOSR/photos/%'", []).unwrap();
        c.execute("UPDATE items SET hash = 'sha256:b' WHERE path = '2012/scans Peter/scan1.tif'", []).unwrap();
        let rows = Db::open(&p).unwrap().items_by_hash(&["sha256:a".into(), "sha256:b".into(), "sha256:none".into()]).unwrap();
        let got: Vec<_> = rows.iter().map(|(m, i)| (m.as_str(), i.path.as_str())).collect();
        assert_eq!(got, [("2012/scans Peter", "2012/scans Peter/scan1.tif"), ("2026/07", "2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3")]);
    }

    #[test]
    fn count_per_month_counts_photos_and_videos_with_those_hashes() {
        let (_d, p) = fixture();
        let c = Connection::open(&p).unwrap();
        c.execute("UPDATE items SET hash = 'sha256:h' WHERE path LIKE '2026/07/EOSR/%' OR path = '2012/scans Peter/scan1.tif'", []).unwrap();
        let db = Db::open(&p).unwrap();
        let hashes: std::collections::HashSet<String> = ["sha256:h".to_string()].into();
        let n = db.count_per_month(&hashes).unwrap();
        // the CR3 and the MP4 in 2026/07 (not the sidecar), the scan under its folder
        assert_eq!((n["2026/07"], n["2012/scans Peter"], n.len()), (2, 1, 2));
        assert!(db.count_per_month(&Default::default()).unwrap().is_empty());
    }

    #[test]
    fn filtered_months_and_items() {
        let (_d, p) = fixture();
        let c = Connection::open(&p).unwrap();
        c.execute("UPDATE items SET hash = 'sha256:' || substr(path, -8)", []).unwrap();
        let db = Db::open(&p).unwrap();
        let videos = crate::filter::Filter { kinds: vec!["video".into()], ..Default::default() };
        let m = db.filtered_months(&videos).unwrap();
        // newest first, every month kept; only 2026/07 has a video
        assert_eq!(m, vec![("2026/07".to_string(), 1), ("2012/scans Peter".into(), 0), ("2007/02".into(), 0)]);
        let items = db.filtered_items("2026/07", &videos).unwrap();
        assert_eq!(items.iter().map(|i| i.kind.as_str()).collect::<Vec<_>>(), vec!["video"]);
        let none = crate::filter::Filter::default();
        assert_eq!(db.filtered_items("2026/07", &none).unwrap().len(), 3);
    }
}
