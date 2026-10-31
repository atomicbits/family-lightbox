//! What an import of one source would do (F018/S04), before anything is copied: per file its
//! date, its hash and its place in the library (data contract 3.1), and whether it is new, the
//! library has it already (by hash, wherever), or another file has its name there (3.3).

use crate::capture::{self, Taken};
use crate::{hash, manifest};
use crate::sources::Layout;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// Names the CLI never imports, next to the exclusions of section 1.
const NEVER: &[&str] = &[".DS_Store", "*.sh", "Icon\r", ".localized"];

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "what", rename_all = "snake_case")]
pub enum Action {
    New,
    /// The library (or an earlier file of this plan) has this content already, at `of`.
    Duplicate { of: String },
    /// Another file has this name at the target; the target got `~<hash>` (3.3).
    Conflict { existing: String },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Planned {
    pub src: String,
    /// Relative to the library root.
    pub target: String,
    pub size: u64,
    pub hash: String,
    pub kind: &'static str,
    /// As the manifest writes it.
    pub taken: String,
    pub taken_from: &'static str,
    pub action: Action,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SourcePlan {
    pub name: String,
    pub path: String,
    pub files: Vec<Planned>,
    /// New and renamed files, which an import copies.
    pub copy_files: usize,
    pub copy_bytes: u64,
    pub duplicates: usize,
    pub conflicts: usize,
    pub excluded: usize,
    /// Of the files to copy, per month.
    pub months: BTreeMap<String, usize>,
    /// Of the files to copy, how many dates don't come from the file itself, by why.
    pub not_from_file: BTreeMap<String, usize>,
    pub problems: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Progress {
    pub files_done: usize,
    pub files_total: usize,
    pub bytes_done: u64,
    pub bytes_total: u64,
}

/// `*` and `?` as in a shell's pattern.
pub fn glob(pattern: &str, name: &str) -> bool {
    fn go(p: &[char], n: &[char]) -> bool {
        match p.split_first() {
            None => n.is_empty(),
            Some(('*', rest)) => (0..=n.len()).any(|i| go(rest, &n[i..])),
            Some(('?', rest)) => !n.is_empty() && go(rest, &n[1..]),
            Some((c, rest)) => n.first() == Some(c) && go(rest, &n[1..]),
        }
    }
    go(&pattern.chars().collect::<Vec<_>>(), &name.chars().collect::<Vec<_>>())
}

fn left_out(name: &str, patterns: &[String]) -> bool {
    manifest::is_excluded(name) || NEVER.iter().any(|p| glob(p, name)) || patterns.iter().any(|p| glob(p, name))
}

/// The files below `root`, sorted, and how many were left out.
pub fn files(root: &Path, patterns: &[String]) -> (Vec<PathBuf>, usize) {
    fn walk(dir: &Path, patterns: &[String], out: &mut Vec<PathBuf>, left: &mut usize) {
        let Ok(rd) = fs::read_dir(dir) else { return };
        let mut entries: Vec<_> = rd.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let name = e.file_name().to_string_lossy().into_owned();
            if left_out(&name, patterns) {
                *left += 1;
                continue;
            }
            match e.file_type() {
                Ok(t) if t.is_dir() => walk(&e.path(), patterns, out, left),
                Ok(t) if t.is_file() => out.push(e.path()),
                _ => {}
            }
        }
    }
    let (mut out, mut left) = (Vec::new(), 0);
    walk(root, patterns, &mut out, &mut left);
    (out, left)
}

/// Hashes kept per file between plans: the same file isn't read twice.
pub struct HashCache {
    conn: Connection,
}

impl HashCache {
    pub fn open(path: &Path) -> Result<HashCache, String> {
        let conn = Connection::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        conn.execute_batch("CREATE TABLE IF NOT EXISTS hashes (path TEXT PRIMARY KEY, size INTEGER, mtime_ns INTEGER, hash TEXT)")
            .map_err(|e| e.to_string())?;
        Ok(HashCache { conn })
    }

    fn stamp(p: &Path) -> Option<(u64, i64)> {
        let m = fs::metadata(p).ok()?;
        let ns = m.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_nanos() as i64;
        Some((m.len(), ns))
    }

    /// The file's hash, from the cache when its size and time are unchanged.
    pub fn hash(&self, p: &Path) -> Result<String, String> {
        let (size, ns) = Self::stamp(p).ok_or_else(|| format!("{}: niet leesbaar", p.display()))?;
        let key = p.display().to_string();
        let kept: Option<String> = self
            .conn
            .query_row("SELECT hash FROM hashes WHERE path = ?1 AND size = ?2 AND mtime_ns = ?3", params![key, size as i64, ns], |r| r.get(0))
            .ok();
        if let Some(h) = kept {
            return Ok(h);
        }
        let h = hash::sha256_file(p)?;
        self.conn
            .execute("INSERT OR REPLACE INTO hashes VALUES (?1, ?2, ?3, ?4)", params![key, size as i64, ns, h])
            .map_err(|e| e.to_string())?;
        Ok(h)
    }
}

/// `2026-07-30T16:58:12+02:00` → ("2026", "07", "2026-07-30").
fn date_parts(t: &Taken) -> (String, String, String) {
    let s = t.manifest_time();
    (s[..4].to_string(), s[5..7].to_string(), s[..10].to_string())
}

/// Where a file goes, below the library root.
pub fn target(name: &str, layout: Layout, kind: &str, taken: &Taken, file: &str) -> String {
    let (y, m, day) = date_parts(taken);
    let mut t = format!("{y}/{m}/{name}");
    if layout.split {
        t.push_str(if kind == "video" { "/videos" } else { "/photos" });
    }
    if layout.day {
        t.push('/');
        t.push_str(&day);
    }
    format!("{t}/{file}")
}

/// `IMG_0955.JPG` with `sha256:3f9a1c2e…` → `IMG_0955~3f9a1c2e.JPG` (3.3).
pub fn conflict_name(target: &str, hash: &str) -> String {
    let short: String = hash.trim_start_matches("sha256:").chars().take(8).collect();
    let (dir, file) = target.rsplit_once('/').unwrap_or(("", target));
    let renamed = match file.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => format!("{stem}~{short}.{ext}"),
        _ => format!("{file}~{short}"),
    };
    if dir.is_empty() { renamed } else { format!("{dir}/{renamed}") }
}

/// The plan for the files under `root` going to source `name` with `layout`. `known` is every
/// path in the library with its hash.
pub fn plan(
    root: &Path,
    name: &str,
    layout: Layout,
    patterns: &[String],
    known: &HashMap<String, String>,
    cache: &HashCache,
    progress: &mut dyn FnMut(&Progress),
) -> SourcePlan {
    let mut out = SourcePlan { name: name.to_string(), path: root.display().to_string(), ..Default::default() };
    let (list, excluded) = files(root, patterns);
    out.excluded = excluded;
    let by_hash: HashMap<&str, &str> = known.iter().map(|(p, h)| (h.as_str(), p.as_str())).collect();
    let mut pr = Progress { files_total: list.len(), bytes_total: list.iter().filter_map(|p| fs::metadata(p).ok()).map(|m| m.len()).sum(), ..Default::default() };
    progress(&pr);

    let dates = capture::resolve_with(&list, |p| capture::read(p).date);
    let key = |p: &Path| (p.parent().map(Path::to_path_buf), p.file_stem().map(|s| s.to_os_string()));
    // a sidecar goes where its file goes
    let mut dirs: HashMap<_, String> = HashMap::new();
    for p in list.iter().filter(|p| capture::kind_of(p) != "sidecar") {
        let file = p.file_name().unwrap_or_default().to_string_lossy();
        let (taken, _) = dates[p];
        let t = target(name, layout, capture::kind_of(p), &taken, &file);
        dirs.entry(key(p)).or_insert_with(|| t.rsplit_once('/').map(|(d, _)| d.to_string()).unwrap_or_default());
    }
    let mut seen: HashMap<String, String> = HashMap::new(); // hash → target in this plan
    let mut taken_targets: HashMap<String, String> = HashMap::new(); // target → hash in this plan
    for p in &list {
        let size = fs::metadata(p).map(|m| m.len()).unwrap_or(0);
        let h = match cache.hash(p) {
            Ok(h) => h,
            Err(e) => {
                out.problems.push(e);
                continue;
            }
        };
        pr.files_done += 1;
        pr.bytes_done += size;
        progress(&pr);
        let kind = capture::kind_of(p);
        let file = p.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let (taken, how) = dates[p];
        let mut t = match (kind, dirs.get(&key(p))) {
            ("sidecar", Some(dir)) => format!("{dir}/{file}"),
            _ => target(name, layout, kind, &taken, &file),
        };
        let action = if let Some(at) = by_hash.get(h.as_str()) {
            Action::Duplicate { of: at.to_string() }
        } else if let Some(at) = seen.get(&h) {
            Action::Duplicate { of: at.clone() }
        } else if known.get(&t).is_some_and(|other| *other != h) || taken_targets.get(&t).is_some_and(|other| *other != h) {
            let existing = t.clone();
            t = conflict_name(&t, &h);
            Action::Conflict { existing }
        } else {
            Action::New
        };
        match &action {
            Action::Duplicate { .. } => out.duplicates += 1,
            other => {
                if matches!(other, Action::Conflict { .. }) {
                    out.conflicts += 1;
                }
                out.copy_files += 1;
                out.copy_bytes += size;
                *out.months.entry(t[..7].to_string()).or_default() += 1;
                if how != "exif" {
                    *out.not_from_file.entry(how.to_string()).or_default() += 1;
                }
                seen.insert(h.clone(), t.clone());
                taken_targets.insert(t.clone(), h.clone());
            }
        }
        out.files.push(Planned { src: p.display().to_string(), target: t, size, hash: h, kind, taken: taken.manifest_time(), taken_from: how, action });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::fixtures::{jpeg_file, tiff, CANON};
    use chrono::NaiveDateTime;

    fn mk(root: &Path, rel: &str, bytes: &[u8]) -> PathBuf {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, bytes).unwrap();
        p
    }

    fn clock(s: &str) -> Taken {
        Taken::Clock(NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap())
    }

    #[test]
    fn patterns_and_targets() {
        assert!(glob("*-mask.png", "a-mask.png") && glob("Icon\r", "Icon\r") && glob("?.jpg", "a.jpg"));
        assert!(!glob("*-mask.png", "mask.png.jpg") && !glob("?.jpg", "ab.jpg"));
        let t = clock("2026-07-30 16:58:12");
        assert_eq!(target("Hero13", Layout { split: false, day: true }, "video", &t, "GX01.MP4"), "2026/07/Hero13/2026-07-30/GX01.MP4");
        assert_eq!(target("Cam", Layout { split: true, day: true }, "photo", &t, "a.CR3"), "2026/07/Cam/photos/2026-07-30/a.CR3");
        assert_eq!(target("Cam", Layout { split: true, day: true }, "video", &t, "a.MP4"), "2026/07/Cam/videos/2026-07-30/a.MP4");
        assert_eq!(target("Phone", Layout { split: false, day: false }, "photo", &t, "IMG_1.HEIC"), "2026/07/Phone/IMG_1.HEIC");
        assert_eq!(conflict_name("2026/07/P/IMG_0955.JPG", "sha256:3f9a1c2e99"), "2026/07/P/IMG_0955~3f9a1c2e.JPG");
        assert_eq!(conflict_name("README", "sha256:abcdef0123"), "README~abcdef01");
    }

    #[test]
    fn new_duplicates_and_conflicts() {
        let d = tempfile::tempdir().unwrap();
        let src = d.path().join("Offload/Hero13");
        let photo = jpeg_file(&tiff(true, CANON, &[(0x9003, "2026:07:30 10:00:00")]));
        mk(&src, "a/GOPR0001.JPG", &photo);
        mk(&src, "a/GOPR0001.THM", b"thumb"); // a sidecar follows its file
        mk(&src, "b/GOPR0001.JPG", &photo); // the same content twice in this source
        mk(&src, "c/IMG_0955.JPG", b"another photo"); // a name the library has with other content
        mk(&src, "c/old.jpg", b"already in the library");
        mk(&src, "c/x-mask.png", b"left out by the source's pattern");
        mk(&src, "c/.DS_Store", b"never");
        // modification time decides the date of the files without metadata: pin it
        let t = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_785_000_000);
        for f in ["c/IMG_0955.JPG", "c/old.jpg", "a/GOPR0001.THM"] {
            fs::File::options().write(true).open(src.join(f)).unwrap().set_modified(t).unwrap();
        }
        let mtime_target = target("Hero13", Layout { split: false, day: true }, "photo", &Taken::Instant(chrono::DateTime::from_timestamp(1_785_000_000, 0).unwrap().fixed_offset()), "IMG_0955.JPG");
        let old_hash = hash::sha256_file(&src.join("c/old.jpg")).unwrap();
        let known: HashMap<String, String> = [(mtime_target.clone(), "sha256:somethingelse".to_string()), ("2019/01/X/old.jpg".to_string(), old_hash)].into_iter().collect();
        let cache = HashCache::open(&d.path().join("h.sqlite")).unwrap();
        let mut seen = Vec::new();
        let p = plan(&src, "Hero13", Layout { split: false, day: true }, &["*-mask.png".to_string()], &known, &cache, &mut |pr| seen.push(pr.files_done));

        let by = |suffix: &str| p.files.iter().find(|f| f.src.ends_with(suffix)).unwrap();
        assert_eq!(by("a/GOPR0001.JPG").target, "2026/07/Hero13/2026-07-30/GOPR0001.JPG");
        assert_eq!((by("a/GOPR0001.JPG").action.clone(), by("a/GOPR0001.JPG").taken_from), (Action::New, "exif"));
        assert_eq!(by("a/GOPR0001.THM").target, "2026/07/Hero13/2026-07-30/GOPR0001.THM");
        assert_eq!(by("b/GOPR0001.JPG").action, Action::Duplicate { of: "2026/07/Hero13/2026-07-30/GOPR0001.JPG".into() });
        assert!(matches!(&by("c/IMG_0955.JPG").action, Action::Conflict { existing } if *existing == mtime_target));
        assert!(by("c/IMG_0955.JPG").target.contains("IMG_0955~"));
        assert_eq!(by("c/old.jpg").action, Action::Duplicate { of: "2019/01/X/old.jpg".into() });
        assert_eq!((p.copy_files, p.duplicates, p.conflicts, p.excluded), (3, 2, 1, 2));
        assert_eq!(p.not_from_file.get("partner"), Some(&1)); // the THM takes its photo's date
        assert_eq!(p.not_from_file.get("mtime"), Some(&1));
        assert_eq!(seen.last(), Some(&5));

        // planning again reads the hashes from the cache
        fs::write(src.join("c/old.jpg"), b"already in the library").unwrap(); // a new mtime: hashed again
        let again = plan(&src, "Hero13", Layout { split: false, day: true }, &["*-mask.png".to_string()], &known, &cache, &mut |_| {});
        assert_eq!(again.files.len(), 5);
        assert_eq!(cache.conn.query_row("SELECT count(*) FROM hashes", [], |r| r.get::<_, i64>(0)).unwrap(), 5);
    }
}
