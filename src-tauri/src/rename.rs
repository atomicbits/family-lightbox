//! Renaming sources (F022): a source, or several merged, gets another name in the library. This
//! is the plan (S01), read from the NAS, without writing anything: per month what moves where.
//! What is below the source's folder stays as it is, so months with and without day folders keep
//! theirs (data contract 3.1).

use crate::index::manifest_paths;
use crate::manifest;
use crate::sources::source_name_problem;
use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;

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
        return Err(format!("'{to}' kan niet tegelijk de oude en de nieuwe naam zijn"));
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
        let Some(m) = manifest::read(&mf) else { return Err(format!("{folder}: manifest niet leesbaar")) };
        let rows: Vec<&Value> = m["files"].as_array().map(|a| a.iter().collect()).unwrap_or_default();
        let ours = |r: &Value| r["source"].as_str().is_some_and(|s| from.contains(s));
        if !rows.iter().any(|r| ours(r)) {
            continue;
        }
        // what the month has already, outside the sources that move: path → hash
        let staying: HashMap<&str, &str> =
            rows.iter().filter(|r| !ours(r)).filter_map(|r| Some((r["path"].as_str()?, r["hash"].as_str().unwrap_or("")))).collect();
        let staying_by_hash: HashMap<&str, &str> = staying.iter().map(|(p, h)| (*h, *p)).filter(|(h, _)| !h.is_empty()).collect();
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
        assert_eq!((p.files, p.sidecars, p.previews, p.duplicates, p.conflicts), (4, 1, 1, 1, 1));
        // the LRV and the THM don't go to Drive
        assert_eq!((p.bytes, p.drive_bytes), (10 * 4 + 9, 10 * 3));
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
}
