//! Renaming sources (F022): a source, or several merged, gets another name in the library. This
//! is the plan (S01), read from the NAS, without writing anything: per month what moves where.
//! What is below the source's folder stays as it is, so months with and without day folders keep
//! theirs (data contract 3.1).

use crate::index::manifest_paths;
use crate::manifest;
use crate::sources::source_name_problem;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// One original or sidecar that moves.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Move {
    pub from: String,
    pub to: String,
    pub size: u64,
    /// A sidecar moves with its original; it has no row of its own.
    pub sidecar: bool,
    /// Whether its preview and thumbnail exist on the NAS, to move along.
    pub preview: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct MonthPlan {
    /// The year subfolder, `YYYY/MM`.
    pub month: String,
    pub moves: Vec<Move>,
    /// Originals whose content the month has already under the target: (path, where), left alone.
    pub duplicates: Vec<(String, String)>,
    /// Originals whose target name the month has with other content: they get `~<hash>`.
    pub conflicts: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct RenamePlan {
    pub from: Vec<String>,
    pub to: String,
    pub months: Vec<MonthPlan>,
    pub files: usize,
    pub sidecars: usize,
    pub previews: usize,
    pub bytes: u64,
    /// What the Drive copy uploads again: everything moved but `lrv` and `thm` (data contract 8).
    pub drive_bytes: u64,
    pub duplicates: usize,
    pub conflicts: usize,
}

/// `2020/08/Hero7/2020-08-09/a.MP4` with `to` = `GoPro-Hero7`: the source's folder renamed.
pub fn renamed_path(path: &str, to: &str) -> Option<String> {
    let mut parts: Vec<&str> = path.split('/').collect();
    if parts.len() < 4 {
        return None;
    }
    parts[2] = to;
    Some(parts.join("/"))
}

fn goes_to_drive(path: &str) -> bool {
    let ext = path.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default();
    ext != "lrv" && ext != "thm"
}

/// The plan for renaming the sources `from` to `to`, read from the `Library` and `Previews`
/// folders on the NAS.
pub fn plan(library: &Path, previews: &Path, from: &[String], to: &str) -> Result<RenamePlan, String> {
    if let Some(p) = source_name_problem(to) {
        return Err(p);
    }
    if from.is_empty() || from.iter().any(|f| f == to) {
        return Err(tr!("'{to}' kan niet tegelijk de oude en de nieuwe naam zijn", "'{to}' cannot be both the old and the new name"));
    }
    let from: HashSet<&str> = from.iter().map(String::as_str).collect();
    let mut out = RenamePlan { from: from.iter().map(|s| s.to_string()).collect(), to: to.to_string(), ..Default::default() };
    out.from.sort();
    for mf in manifest_paths(library)? {
        let dir = mf.parent().unwrap_or(&mf);
        let folder = dir.strip_prefix(library).unwrap_or(dir).to_string_lossy().replace('\\', "/");
        // only ordinary month folders; the old layout (`MM.<name>`) is read-only (contract 3.1)
        let ordinary = folder.len() == 7 && folder.as_bytes()[4] == b'/' && folder.bytes().filter(u8::is_ascii_digit).count() == 6;
        if !ordinary {
            continue;
        }
        let Some(m) = manifest::read(&mf) else { return Err(tr!("{folder}: manifest niet leesbaar", "{folder}: manifest not readable")) };
        let rows: Vec<&Value> = m["files"].as_array().map(|a| a.iter().collect()).unwrap_or_default();
        let ours = |r: &Value| r["source"].as_str().is_some_and(|s| from.contains(s));
        if !rows.iter().any(|r| ours(r)) {
            continue;
        }
        // what the month has already, outside the sources that move: path → hash
        let staying: HashMap<&str, &str> =
            rows.iter().filter(|r| !ours(r)).filter_map(|r| Some((r["path"].as_str()?, r["hash"].as_str().unwrap_or("")))).collect();
        // a duplicate is the same content under the target already; under another source it is
        // that source's business, and the file moves like any other
        let staying_by_hash: HashMap<&str, &str> = rows
            .iter()
            .filter(|r| r["source"].as_str() == Some(to))
            .filter_map(|r| Some((r["hash"].as_str().filter(|h| !h.is_empty())?, r["path"].as_str()?)))
            .collect();
        let mut month = MonthPlan { month: folder.clone(), ..Default::default() };
        for r in rows.iter().filter(|r| ours(r)) {
            let Some(path) = r["path"].as_str() else { continue };
            let hash = r["hash"].as_str().unwrap_or("");
            if let Some(there) = staying_by_hash.get(hash) {
                month.duplicates.push((path.to_string(), there.to_string()));
                continue;
            }
            let Some(mut target) = renamed_path(path, to) else { continue };
            if staying.get(target.as_str()).is_some_and(|h| *h != hash) {
                target = crate::import_plan::conflict_name(&target, hash);
                month.conflicts += 1;
            }
            let has_preview = previews.join(format!("{path}.jpg")).is_file();
            month.moves.push(Move { from: path.to_string(), to: target, size: r["size"].as_u64().unwrap_or(0), sidecar: false, preview: has_preview });
            for sc in r["sidecars"].as_array().into_iter().flatten().filter_map(Value::as_str) {
                if let Some(t) = renamed_path(sc, to) {
                    let size = std::fs::metadata(library.join(sc)).map(|m| m.len()).unwrap_or(0);
                    month.moves.push(Move { from: sc.to_string(), to: t, size, sidecar: true, preview: false });
                }
            }
        }
        for mv in &month.moves {
            if mv.sidecar {
                out.sidecars += 1;
            } else {
                out.files += 1;
            }
            out.previews += mv.preview as usize;
            out.bytes += mv.size;
            if goes_to_drive(&mv.from) {
                out.drive_bytes += mv.size;
            }
        }
        out.duplicates += month.duplicates.len();
        out.conflicts += month.conflicts;
        out.months.push(month);
    }
    Ok(out)
}

// ── executing (S03) and applying on each Mac (S04) ────────────────────────────────────────────

/// The folder of the renames: in `Lists` the records every Mac applies, in the app's state the
/// journal of a rename being executed.
pub const RENAMES_DIR: &str = "renames";

/// A rename done on the NAS (data contract 5.6): what every Mac moves in its own copies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub v: u32,
    pub id: String,
    pub ts: String,
    pub dev: String,
    pub who: String,
    pub from: Vec<String>,
    pub to: String,
    /// Every original and sidecar, from and to, relative to `Library`.
    pub moves: Vec<(String, String)>,
}

/// A rename being executed, kept in the app's state so it continues after an interruption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub record: Record,
    pub months: Vec<String>,
    pub done: Vec<String>,
    /// The record is in `Lists/renames/` on the NAS.
    pub published: bool,
}

impl Job {
    pub fn new(plan: &RenamePlan, dev: &str, who: &str) -> Job {
        let ts = crate::lists::now();
        let id = format!("{}-{}", dev, ts.get(..19).unwrap_or(&ts).replace([':', '-'], ""));
        let moves = plan.months.iter().flat_map(|m| m.moves.iter().map(|x| (x.from.clone(), x.to.clone()))).collect();
        let record = Record { v: 1, id, ts, dev: dev.to_string(), who: who.to_string(), from: plan.from.clone(), to: plan.to.clone(), moves };
        Job { record, months: plan.months.iter().map(|m| m.month.clone()).collect(), done: vec![], published: false }
    }

    fn path(config: &Path, id: &str) -> PathBuf {
        config.join(RENAMES_DIR).join(format!("{id}.json"))
    }

    pub fn save(&self, config: &Path) -> Result<(), String> {
        let p = Job::path(config, &self.record.id);
        fs::create_dir_all(p.parent().unwrap_or(config)).map_err(|e| e.to_string())?;
        manifest::write(&p, &serde_json::to_value(self).map_err(|e| e.to_string())?)
    }

    /// The rename this Mac didn't finish, if any.
    pub fn pending(config: &Path) -> Option<Job> {
        let rd = fs::read_dir(config.join(RENAMES_DIR)).ok()?;
        rd.flatten().filter_map(|e| serde_json::from_slice::<Job>(&fs::read(e.path()).ok()?).ok()).find(|j| !j.published)
    }

    fn finish(&self, config: &Path) {
        let _ = fs::remove_file(Job::path(config, &self.record.id));
    }
}

/// Moves `from` to `to`, never over a file; done already when only `to` is there.
fn move_one(from: &Path, to: &Path) -> Result<(), String> {
    match (from.exists(), to.exists()) {
        (false, true) => Ok(()),
        (false, false) => Err(tr!("{} niet gevonden", "{} not found", from.display())),
        (true, true) => Err(tr!("{} bestaat al", "{} already exists", to.display())),
        (true, false) => {
            if let Some(d) = to.parent() {
                fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
            }
            fs::rename(from, to).map_err(|e| format!("{} → {}: {e}", from.display(), to.display()))
        }
    }
}

/// A preview and thumbnail, and Lightroom's renditions (F025), when there.
fn move_previews(previews: &Path, from: &str, to: &str, mover: fn(&Path, &Path) -> Result<(), String>) -> Result<(), String> {
    let f = previews.join(from);
    let renditions = match (f.parent(), f.file_name()) {
        (Some(dir), Some(name)) => crate::renditions::suffixes(dir, &name.to_string_lossy()),
        _ => Vec::new(),
    };
    for ext in [".jpg", ".t.jpg"].into_iter().chain(renditions.iter().map(String::as_str)) {
        let (f, t) = (previews.join(format!("{from}{ext}")), previews.join(format!("{to}{ext}")));
        if f.exists() || t.exists() {
            mover(&f, &t)?;
        }
    }
    Ok(())
}

/// On a Mac's own copies (S04): as `move_one`, but when the sync already brought the file to its
/// new place, the old copy of the same size is a twin and goes. Never used on the NAS.
fn move_here(from: &Path, to: &Path) -> Result<(), String> {
    if from.exists() && to.exists() {
        let size = |p: &Path| fs::metadata(p).map(|m| m.len()).ok();
        if size(from) == size(to) {
            return fs::remove_file(from).map_err(|e| format!("{}: {e}", from.display()));
        }
    }
    move_one(from, to)
}

/// Removes the empty folders the move left below `<month>/<source>`, the source's folder too.
fn prune(root: &Path, month: &str, sources: &[String]) {
    fn go(dir: &Path) -> bool {
        let Ok(rd) = fs::read_dir(dir) else { return false };
        let mut empty = true;
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if go(&p) {
                    let _ = fs::remove_dir(&p);
                } else {
                    empty = false;
                }
            } else if e.file_name() == ".DS_Store" {
                let _ = fs::remove_file(&p);
            } else {
                empty = false;
            }
        }
        empty
    }
    for s in sources {
        let d = root.join(month).join(s);
        if go(&d) {
            let _ = fs::remove_dir(&d);
        }
    }
}

/// The month's manifest with the moved rows' paths, sidecars and source changed.
fn renamed_manifest(m: &Value, moves: &HashMap<&str, &str>, to: &str) -> Value {
    let mut m = m.clone();
    for r in m["files"].as_array_mut().into_iter().flatten() {
        let Some(new) = r["path"].as_str().and_then(|p| moves.get(p)).map(|s| s.to_string()) else { continue };
        r["path"] = Value::String(new);
        r["source"] = Value::String(to.to_string());
        if let Some(sc) = r["sidecars"].as_array_mut() {
            for x in sc.iter_mut() {
                if let Some(n) = x.as_str().and_then(|p| moves.get(p)) {
                    *x = Value::String(n.to_string());
                }
            }
        }
    }
    m
}

/// Executes a rename on the NAS (S03), month by month: originals and sidecars, previews and
/// thumbnails, then the month's manifest, verified; the journal is saved after every month, so an
/// interruption continues where it was. At the end the record goes to `Lists/renames/`.
pub fn execute(job: &mut Job, library: &Path, previews: &Path, lists: &Path, config: &Path, progress: &mut dyn FnMut(usize, usize)) -> Result<(), String> {
    job.save(config)?;
    let total = job.months.len();
    let moves: HashMap<&str, &str> = job.record.moves.iter().map(|(f, t)| (f.as_str(), t.as_str())).collect();
    let months: Vec<String> = job.months.iter().filter(|m| !job.done.contains(m)).cloned().collect();
    for month in months {
        let prefix = format!("{month}/");
        let ours: Vec<(&str, &str)> = job.record.moves.iter().filter(|(f, _)| f.starts_with(&prefix)).map(|(f, t)| (f.as_str(), t.as_str())).collect();
        for (from, to) in &ours {
            move_one(&library.join(from), &library.join(to))?;
            move_previews(previews, from, to, move_one)?;
        }
        let mf = library.join(&month).join(manifest::MANIFEST_NAME);
        let m = manifest::read(&mf).ok_or_else(|| tr!("{month}: manifest niet leesbaar", "{month}: manifest not readable"))?;
        manifest::write(&mf, &renamed_manifest(&m, &moves, &job.record.to))?;
        // verified: every move at its new place, none at its old one, the manifest naming them
        let written = manifest::read(&mf).ok_or_else(|| tr!("{month}: manifest niet leesbaar na het schrijven", "{month}: manifest not readable after writing"))?;
        let rows: HashSet<&str> = written["files"].as_array().into_iter().flatten().filter_map(|r| r["path"].as_str()).collect();
        for (from, to) in &ours {
            if library.join(from).exists() || !library.join(to).is_file() {
                return Err(tr!("{month}: {from} staat niet op zijn nieuwe plaats", "{month}: {from} is not in its new place"));
            }
            if rows.contains(from) {
                return Err(tr!("{month}: het manifest noemt {from} nog", "{month}: the manifest still lists {from}"));
            }
        }
        prune(library, &month, &job.record.from);
        prune(previews, &month, &job.record.from);
        job.done.push(month);
        job.save(config)?;
        progress(job.done.len(), total);
    }
    let dir = lists.join(RENAMES_DIR);
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = dir.join(format!("{}.json", job.record.id));
    let tmp = dir.join(format!("{}.json.tmp-{}", job.record.id, job.record.dev));
    let body = serde_json::to_vec_pretty(&job.record).map_err(|e| e.to_string())?;
    crate::lists::write_synced(&tmp, &body).map_err(|e| format!("{}: {e}", tmp.display()))?;
    fs::rename(&tmp, &path).map_err(|e| format!("{}: {e}", path.display()))?;
    job.published = true;
    job.finish(config);
    Ok(())
}

/// The renames in a copy of `Lists` this Mac hasn't applied yet, oldest first.
pub fn records(lists: &Path) -> Vec<Record> {
    let mut out: Vec<Record> = fs::read_dir(lists.join(RENAMES_DIR))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        .filter_map(|e| serde_json::from_slice(&fs::read(e.path()).ok()?).ok())
        .collect();
    out.sort_by(|a, b| a.ts.cmp(&b.ts).then_with(|| a.id.cmp(&b.id)));
    out
}

/// Applies a rename to this Mac's working copy and previews (S04): what is there under the old
/// name moves; what isn't (a month not kept here) is left. Returns how many files moved.
pub fn apply(record: &Record, library: &Path, previews: &Path) -> Result<usize, String> {
    let mut moved = 0;
    let mut months = std::collections::BTreeSet::new();
    for (from, to) in &record.moves {
        if library.join(from).exists() {
            move_here(&library.join(from), &library.join(to))?;
            moved += 1;
        }
        move_previews(previews, from, to, move_here)?;
        if let Some(m) = from.get(..7) {
            months.insert(m.to_string());
        }
    }
    for m in &months {
        prune(library, m, &record.from);
        prune(previews, m, &record.from);
    }
    Ok(moved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    fn put(root: &Path, rel: &str, body: &[u8]) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }

    fn write_manifest(lib: &Path, folder: &str, rows: Vec<Value>) {
        fs::create_dir_all(lib.join(folder)).unwrap();
        manifest::write(&lib.join(folder).join(".manifest.json"), &json!({"folder": folder, "files": rows})).unwrap();
    }

    fn row(path: &str, source: &str, hash: &str, sidecars: &[&str]) -> Value {
        json!({"path": path, "size": 10, "hash": hash, "kind": "video", "source": source, "sidecars": sidecars})
    }

    #[test]
    fn a_merge_planned_month_by_month() {
        let d = tempfile::tempdir().unwrap();
        let (lib, prev) = (d.path().join("Library"), d.path().join("Previews"));
        write_manifest(&lib, "2020/08", vec![
            row("2020/08/Hero7/2020-08-09/GX01.MP4", "Hero7", "sha256:a", &["2020/08/Hero7/2020-08-09/GX01.THM"]),
            row("2020/08/Hero7/2020-08-09/GL01.LRV", "Hero7", "sha256:l", &[]),
            row("2020/08/Hero7/2020-08-09/SAME.MP4", "Hero7", "sha256:same", &[]),
            row("2020/08/Hero7/2020-08-09/CLASH.MP4", "Hero7", "sha256:mine", &[]),
            row("2020/08/GoPro-Hero7/2020-08-09/CLASH.MP4", "GoPro-Hero7", "sha256:theirs", &[]),
            row("2020/08/GoPro-Hero7/x.MP4", "GoPro-Hero7", "sha256:same", &[]),
            row("2020/08/EOSR/photos/2020-08-09/a.CR3", "EOSR", "sha256:e", &[]),
            // the same content in an unrelated source of the month: no duplicate, it moves
            row("2020/08/Hero7/2020-08-09/ALSO.MP4", "Hero7", "sha256:shared", &[]),
            row("2020/08/Trip/ALSO.MP4", "Trip", "sha256:shared", &[]),
        ]);
        put(&lib, "2020/08/Hero7/2020-08-09/GX01.THM", b"thumbnail");
        write_manifest(&lib, "2019/02", vec![row("2019/02/GoProH7/GX0107.MP4", "GoProH7", "sha256:g", &[])]);
        // the old layout is never touched, even with the name in it
        write_manifest(&lib, "2019/02.Ski", vec![row("2019/02.Ski/GoProH7-nodate/a.LRV", "GoProH7", "sha256:o", &[])]);
        put(&prev, "2020/08/Hero7/2020-08-09/GX01.MP4.jpg", b"p");

        let p = plan(&lib, &prev, &["Hero7".into(), "GoProH7".into()], "GoPro-Hero7").unwrap();
        assert_eq!(p.months.iter().map(|m| m.month.as_str()).collect::<Vec<_>>(), ["2019/02", "2020/08"]);
        let aug = &p.months[1];
        let to = |from: &str| aug.moves.iter().find(|m| m.from.ends_with(from)).map(|m| m.to.clone());
        assert_eq!(to("GX01.MP4").as_deref(), Some("2020/08/GoPro-Hero7/2020-08-09/GX01.MP4"));
        assert_eq!(to("GX01.THM").as_deref(), Some("2020/08/GoPro-Hero7/2020-08-09/GX01.THM"));
        assert!(to("CLASH.MP4").unwrap().contains("CLASH~mine"));
        assert_eq!(aug.duplicates, [("2020/08/Hero7/2020-08-09/SAME.MP4".to_string(), "2020/08/GoPro-Hero7/x.MP4".to_string())]);
        assert!(aug.moves.iter().find(|m| m.from.ends_with("GX01.MP4")).unwrap().preview);
        assert_eq!(to("ALSO.MP4").as_deref(), Some("2020/08/GoPro-Hero7/2020-08-09/ALSO.MP4"));
        assert_eq!((p.files, p.sidecars, p.previews, p.duplicates, p.conflicts), (5, 1, 1, 1, 1));
        // the LRV and the THM don't go to Drive
        assert_eq!((p.bytes, p.drive_bytes), (10 * 5 + 9, 10 * 4));
        assert!(!p.months.iter().any(|m| m.month.contains('.')));
    }

    #[test]
    fn a_target_name_that_cant_be() {
        let d = tempfile::tempdir().unwrap();
        assert!(plan(d.path(), d.path(), &["Hero7".into()], "GoPro Hero7").is_err(), "a space");
        assert!(plan(d.path(), d.path(), &["Hero7".into()], "Hero7").is_err(), "the same");
        assert!(plan(d.path(), d.path(), &[], "X").is_err(), "nothing to rename");
        assert_eq!(renamed_path("2020/08/Hero7/a.MP4", "Y").as_deref(), Some("2020/08/Y/a.MP4"));
        assert_eq!(renamed_path("2020/08/a.MP4", "Y"), None);
    }

    #[test]
    fn executed_on_the_nas_then_applied_on_another_mac() {
        let d = tempfile::tempdir().unwrap();
        let (lib, prev, lists, cfg) = (d.path().join("Library"), d.path().join("Previews"), d.path().join("Lists"), d.path().join("config"));
        write_manifest(&lib, "2020/08", vec![
            row("2020/08/Hero7/2020-08-09/GX01.MP4", "Hero7", "sha256:a", &["2020/08/Hero7/2020-08-09/GX01.THM"]),
            row("2020/08/EOSR/a.CR3", "EOSR", "sha256:e", &[]),
        ]);
        write_manifest(&lib, "2019/02", vec![row("2019/02/GoProH7/GX0107.MP4", "GoProH7", "sha256:g", &[])]);
        for f in ["2020/08/Hero7/2020-08-09/GX01.MP4", "2020/08/Hero7/2020-08-09/GX01.THM", "2020/08/EOSR/a.CR3", "2019/02/GoProH7/GX0107.MP4"] {
            put(&lib, f, b"x");
        }
        fs::create_dir_all(lib.join("2019/02/GoProH7-videos")).unwrap(); // an empty folder of another name stays
        put(&prev, "2020/08/Hero7/2020-08-09/GX01.MP4.jpg", b"p");
        put(&prev, "2020/08/Hero7/2020-08-09/GX01.MP4.t.jpg", b"t");
        put(&prev, "2020/08/Hero7/2020-08-09/GX01.MP4.lr.jpg", b"lr");
        // the other Mac's copies, before the rename
        let (other_lib, other_prev) = (d.path().join("other/Library"), d.path().join("other/Previews"));
        put(&other_lib, "2020/08/Hero7/2020-08-09/GX01.MP4", b"x");
        put(&other_prev, "2020/08/Hero7/2020-08-09/GX01.MP4.jpg", b"p");
        // the other Mac's sync brought the thumbnail at its new place already
        put(&other_prev, "2020/08/GoPro-Hero7/2020-08-09/GX01.MP4.t.jpg", b"t");
        put(&other_prev, "2020/08/Hero7/2020-08-09/GX01.MP4.t.jpg", b"t");

        let plan = plan(&lib, &prev, &["Hero7".into(), "GoProH7".into()], "GoPro-Hero7").unwrap();
        let mut job = Job::new(&plan, "mbp-peter", "peter");
        // as if an earlier run moved one file and stopped
        move_one(&lib.join("2019/02/GoProH7/GX0107.MP4"), &lib.join("2019/02/GoPro-Hero7/GX0107.MP4")).unwrap();
        let mut seen = Vec::new();
        execute(&mut job, &lib, &prev, &lists, &cfg, &mut |n, t| seen.push((n, t))).unwrap();
        assert_eq!(seen, [(1, 2), (2, 2)]);
        for f in ["2020/08/GoPro-Hero7/2020-08-09/GX01.MP4", "2020/08/GoPro-Hero7/2020-08-09/GX01.THM", "2019/02/GoPro-Hero7/GX0107.MP4"] {
            assert!(lib.join(f).is_file(), "{f}");
        }
        assert!(!lib.join("2020/08/Hero7").exists() && !lib.join("2019/02/GoProH7").exists(), "the old folders are gone when empty");
        assert!(lib.join("2019/02/GoProH7-videos").is_dir() && lib.join("2020/08/EOSR/a.CR3").is_file());
        assert!(prev.join("2020/08/GoPro-Hero7/2020-08-09/GX01.MP4.t.jpg").is_file());
        assert!(prev.join("2020/08/GoPro-Hero7/2020-08-09/GX01.MP4.lr.jpg").is_file(), "a rendition moves along (F025)");
        let m = manifest::read(&lib.join("2020/08/.manifest.json")).unwrap();
        let r = &m["files"][0];
        assert_eq!((r["path"].as_str(), r["source"].as_str()), (Some("2020/08/GoPro-Hero7/2020-08-09/GX01.MP4"), Some("GoPro-Hero7")));
        assert_eq!(r["sidecars"][0].as_str(), Some("2020/08/GoPro-Hero7/2020-08-09/GX01.THM"));
        assert_eq!(m["files"][1]["source"].as_str(), Some("EOSR"));
        // published, the journal gone
        assert!(job.published && Job::pending(&cfg).is_none());
        let recs = records(&lists);
        assert_eq!((recs.len(), recs[0].to.as_str(), recs[0].moves.len()), (1, "GoPro-Hero7", 3));
        // the other Mac moves what it has; a month it doesn't keep is left
        assert_eq!(apply(&recs[0], &other_lib, &other_prev).unwrap(), 1);
        assert!(other_lib.join("2020/08/GoPro-Hero7/2020-08-09/GX01.MP4").is_file());
        assert!(other_prev.join("2020/08/GoPro-Hero7/2020-08-09/GX01.MP4.jpg").is_file());
        assert!(!other_prev.join("2020/08/Hero7").exists(), "the twin thumbnail went too");
        assert!(!other_lib.join("2020/08/Hero7").exists());
        assert_eq!(apply(&recs[0], &other_lib, &other_prev).unwrap(), 0, "applying again changes nothing");
    }

    #[test]
    fn a_file_in_the_way_stops_the_month_and_the_journal_stays() {
        let d = tempfile::tempdir().unwrap();
        let (lib, prev, lists, cfg) = (d.path().join("Library"), d.path().join("Previews"), d.path().join("Lists"), d.path().join("config"));
        write_manifest(&lib, "2019/05", vec![row("2019/05/Hero7/a.MP4", "Hero7", "sha256:a", &[])]);
        put(&lib, "2019/05/Hero7/a.MP4", b"x");
        let plan = plan(&lib, &prev, &["Hero7".into()], "GoPro-Hero7").unwrap();
        put(&lib, "2019/05/GoPro-Hero7/a.MP4", b"in the way, not in any manifest");
        let mut job = Job::new(&plan, "mbp-peter", "peter");
        assert!(execute(&mut job, &lib, &prev, &lists, &cfg, &mut |_, _| {}).unwrap_err().contains("bestaat al"));
        assert!(lib.join("2019/05/Hero7/a.MP4").is_file(), "nothing was moved over it");
        assert!(Job::pending(&cfg).is_some() && records(&lists).is_empty());
    }
}
