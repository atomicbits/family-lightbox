//! The manifest of a year subfolder (data contract 3.5), as the app writes it when it imports
//! (F018/S02): built from the files, verified against what is on the NAS, merged with the NAS's
//! manifest and written atomically. A port of `build_manifest`, `verify_month` and
//! `merge_manifest` in the `media` CLI, with the hash reuse of `media-manifest-nas.py`.

use crate::{capture, hash};
use chrono::{DateTime, Local};
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const CONTRACT_VERSION: &str = "1.3";
pub const MANIFEST_NAME: &str = ".manifest.json";

/// Names no component copies or lists (data contract section 1).
pub fn is_excluded(name: &str) -> bool {
    const EXACT: &[&str] = &["@eaDir", "#recycle", "#snapshot", ".DS_Store", "Thumbs.db", ".TemporaryItems", ".Spotlight-V100", ".fseventsd"];
    EXACT.contains(&name) || name.starts_with("._") || name.ends_with(".tmp") || name.ends_with(".partial")
}

/// `2012/07.Trouw` → `2012/07`; a subfolder without a month gives none.
pub fn folder_month(folder: &str) -> Option<String> {
    let (year, sub) = folder.split_once('/')?;
    let b = sub.as_bytes();
    let month_ok = b.len() >= 2 && b[0].is_ascii_digit() && b[1].is_ascii_digit() && (b.len() == 2 || b[2] == b'.' || b[2] == b'-');
    let m: u32 = sub.get(..2)?.parse().ok()?;
    (year.len() == 4 && year.bytes().all(|c| c.is_ascii_digit()) && month_ok && (1..=12).contains(&m)).then(|| format!("{year}/{:02}", m))
}

/// A moment as the contract writes it: this Mac's zone, seconds and offset.
pub fn local_iso(t: impl Into<DateTime<Local>>) -> String {
    t.into().format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

/// The files of a year subfolder, sorted, without the exclusions and the manifest itself.
pub fn folder_files(root: &Path, folder: &str) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = fs::read_dir(dir) else { return };
        let mut entries: Vec<_> = rd.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let name = e.file_name().to_string_lossy().into_owned();
            if is_excluded(&name) {
                continue;
            }
            match e.file_type() {
                Ok(t) if t.is_dir() => walk(&e.path(), out),
                Ok(t) if t.is_file() && name != MANIFEST_NAME => out.push(e.path()),
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    walk(&root.join(folder), &mut out);
    out
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root).unwrap_or(p).to_string_lossy().replace('\\', "/")
}

fn mtime_of(meta: &fs::Metadata) -> String {
    meta.modified().map(local_iso).unwrap_or_default()
}

/// The manifest of `folder` under `root`, for this `device`. A file with the same size and
/// modification time as in `existing` keeps that hash, so nothing is hashed twice.
pub fn build(root: &Path, folder: &str, device: &str, existing: Option<&Value>) -> Result<Value, String> {
    let files = folder_files(root, folder);
    let key = |p: &Path| (p.parent().map(Path::to_path_buf), p.file_stem().map(|s| s.to_os_string()));
    let originals: Vec<&PathBuf> = files.iter().filter(|p| capture::kind_of(p) != "sidecar").collect();
    let stems: HashSet<_> = originals.iter().map(|p| key(p)).collect();
    let mut sidecars: HashMap<_, Vec<String>> = HashMap::new();
    let mut orphans = Vec::new();
    for p in files.iter().filter(|p| capture::kind_of(p) == "sidecar") {
        if stems.contains(&key(p)) {
            sidecars.entry(key(p)).or_default().push(rel(root, p));
        } else {
            // a sidecar without its file gets a row of its own, so it doesn't drop out of sight
            orphans.push(p);
        }
    }
    let entries: Vec<PathBuf> = originals.into_iter().chain(orphans).cloned().collect();

    let mut cameras = HashMap::new();
    let dates = capture::resolve_with(&entries, |p| {
        let c = capture::read(p);
        cameras.insert(p.to_path_buf(), c.camera);
        c.date
    });
    let known: HashMap<&str, &Value> = existing
        .and_then(|m| m["files"].as_array())
        .into_iter()
        .flatten()
        .filter(|r| r["hash"].is_string())
        .filter_map(|r| Some((r["path"].as_str()?, r)))
        .collect();
    let is_month = folder_month(folder).as_deref() == Some(folder);

    let mut rows = Vec::new();
    for p in &entries {
        let meta = fs::metadata(p).map_err(|e| format!("{}: {e}", p.display()))?;
        let path = rel(root, p);
        let mtime = mtime_of(&meta);
        let reused = known.get(path.as_str()).filter(|r| r["size"].as_u64() == Some(meta.len()) && r["mtime"].as_str() == Some(mtime.as_str()));
        let hash = match reused {
            Some(r) => r["hash"].as_str().unwrap_or_default().to_string(),
            None => hash::sha256_file(p)?,
        };
        let parts: Vec<&str> = path.split('/').collect();
        let (taken, how) = dates[p];
        let mut side = sidecars.remove(&key(p)).unwrap_or_default();
        side.sort();
        let kind = capture::kind_of(p);
        let mut row = Map::new();
        row.insert("path".into(), json!(path));
        row.insert("size".into(), json!(meta.len()));
        row.insert("hash".into(), json!(hash));
        row.insert("kind".into(), json!(kind));
        row.insert("source".into(), if is_month && parts.len() > 3 { json!(parts[2]) } else { Value::Null });
        row.insert("taken".into(), json!(taken.manifest_time()));
        row.insert("taken_from".into(), json!(how));
        row.insert("mtime".into(), json!(mtime));
        row.insert("sidecars".into(), json!(side));
        if let (true, Some(Some(cam))) = (kind == "photo" || kind == "video", cameras.get(p)) {
            row.insert("camera".into(), json!(cam));
        }
        rows.push(Value::Object(row));
    }
    rows.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    Ok(json!({
        "contract_version": CONTRACT_VERSION,
        "folder": folder,
        "month": folder_month(folder),
        "generated_at": local_iso(Local::now()),
        "generated_by": device,
        "files": rows,
    }))
}

/// What verification found.
#[derive(Debug, Default, PartialEq)]
pub struct Verified {
    /// The local rows whose file is on the NAS with their size.
    pub ok: Vec<Value>,
    /// Paths (originals or sidecars) not on the NAS.
    pub missing: Vec<String>,
    /// Paths on the NAS with another size, or another hash in the NAS's manifest.
    pub conflicts: Vec<String>,
}

/// Which local rows are now really on the NAS, by the sizes read back there.
pub fn verify(local: &Value, remote: Option<&Value>, sizes: &HashMap<String, u64>) -> Verified {
    let remote_by: HashMap<&str, &Value> = remote
        .and_then(|m| m["files"].as_array())
        .into_iter()
        .flatten()
        .filter_map(|r| Some((r["path"].as_str()?, r)))
        .collect();
    let mut v = Verified::default();
    for r in local["files"].as_array().into_iter().flatten() {
        let path = r["path"].as_str().unwrap_or_default();
        let Some(&size) = sizes.get(path) else {
            v.missing.push(path.to_string());
            continue;
        };
        if Some(size) != r["size"].as_u64() {
            v.conflicts.push(path.to_string());
            continue;
        }
        if let Some(theirs) = remote_by.get(path).and_then(|x| x["hash"].as_str())
            && Some(theirs) != r["hash"].as_str() {
                v.conflicts.push(path.to_string());
                continue;
            }
        for s in r["sidecars"].as_array().into_iter().flatten().filter_map(Value::as_str) {
            if !sizes.contains_key(s) {
                v.missing.push(s.to_string());
            }
        }
        v.ok.push(r.clone());
    }
    v
}

/// The NAS's manifest plus the verified local rows; rows of paths gone from the NAS drop out.
pub fn merge(local: &Value, remote: Option<&Value>, ok: &[Value], sizes: &HashMap<String, u64>) -> Value {
    let mut merged: HashMap<String, Value> = HashMap::new();
    for r in remote.and_then(|m| m["files"].as_array()).into_iter().flatten() {
        let Some(path) = r["path"].as_str() else { continue };
        if sizes.get(path).copied() == r["size"].as_u64() && r["size"].is_u64() {
            merged.insert(path.to_string(), r.clone());
        }
    }
    for r in ok {
        merged.insert(r["path"].as_str().unwrap_or_default().to_string(), r.clone());
    }
    let mut files: Vec<Value> = merged.into_values().collect();
    files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let mut out = local.clone();
    out["files"] = Value::Array(files);
    out
}

/// The sizes of the files of `folder` on the share, by path relative to its root.
pub fn sizes(root: &Path, folder: &str) -> HashMap<String, u64> {
    folder_files(root, folder)
        .into_iter()
        .filter_map(|p| Some((rel(root, &p), fs::metadata(&p).ok()?.len())))
        .collect()
}

/// The manifest at `path`, if there is a readable one.
pub fn read(path: &Path) -> Option<Value> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

/// JSON as Python's `json.dumps(indent=1, ensure_ascii=False)` plus a newline, the CLI's layout.
pub fn to_text(manifest: &Value) -> String {
    let mut buf = Vec::new();
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b" ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
    serde::Serialize::serialize(manifest, &mut ser).expect("a Value always serializes");
    buf.push(b'\n');
    String::from_utf8(buf).expect("serde_json writes UTF-8")
}

/// Written atomically (data contract section 1): a temporary file in the same folder, `fsync`,
/// rename.
pub fn write(path: &Path, manifest: &Value) -> Result<(), String> {
    let err = |e: std::io::Error| format!("{}: {e}", path.display());
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{}.tmp", std::process::id()));
    let tmp = path.with_file_name(name);
    let result = (|| {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(to_text(manifest).as_bytes())?;
        f.sync_all()?;
        fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result.map_err(err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::fixtures::{jpeg_file, tiff, CANON};

    fn mk(root: &Path, rel: &str, bytes: &[u8]) -> PathBuf {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, bytes).unwrap();
        p
    }

    fn sample(root: &Path) {
        mk(root, "2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3", b"raw");
        mk(root, "2026/07/EOSR/photos/2026-07-30/CO7A9123.xmp", b"<xmp/>");
        mk(root, "2026/07/Hero13/2026-07-12/GX010001.JPG", &jpeg_file(&tiff(true, CANON, &[])));
        mk(root, "2026/07/iPhone-photos/wees.xmp", b"orphan");
        mk(root, "2026/07/extra/2026-07-01/notities.txt", b"other");
        mk(root, "2026/07/extra/.DS_Store", b"junk");
        mk(root, "2026/07/extra/@eaDir/x.jpg", b"junk");
        mk(root, "2026/07/extra/x.JPG.123.tmp", b"junk");
        mk(root, "2026/07/.manifest.json", b"{}");
    }

    #[test]
    fn exclusions_and_folder_months() {
        assert!(["@eaDir", "#recycle", ".DS_Store", "._x", "a.tmp", "b.partial", "Thumbs.db"].iter().all(|n| is_excluded(n)));
        assert!(!is_excluded("IMG_1.JPG"));
        assert_eq!(folder_month("2026/07").as_deref(), Some("2026/07"));
        assert_eq!(folder_month("2012/07.Trouw").as_deref(), Some("2012/07"));
        assert_eq!(folder_month("2007/04-07.Within Temptation").as_deref(), Some("2007/04"));
        assert_eq!(folder_month("2012/scans Peter"), None);
        assert_eq!(folder_month("2012/13.Nee"), None);
        assert_eq!(folder_month("2012/00"), None);
    }

    #[test]
    fn build_follows_the_contract() {
        let d = tempfile::tempdir().unwrap();
        sample(d.path());
        let m = build(d.path(), "2026/07", "mbp-test", None).unwrap();
        assert_eq!((m["contract_version"].as_str(), m["folder"].as_str(), m["month"].as_str(), m["generated_by"].as_str()), (Some("1.3"), Some("2026/07"), Some("2026/07"), Some("mbp-test")));
        let files = m["files"].as_array().unwrap();
        let paths: Vec<&str> = files.iter().map(|r| r["path"].as_str().unwrap()).collect();
        assert_eq!(
            paths,
            [
                "2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3",
                "2026/07/Hero13/2026-07-12/GX010001.JPG",
                "2026/07/extra/2026-07-01/notities.txt",
                "2026/07/iPhone-photos/wees.xmp"
            ]
        );
        let cr3 = &files[0];
        assert_eq!(cr3["sidecars"], json!(["2026/07/EOSR/photos/2026-07-30/CO7A9123.xmp"]));
        assert_eq!(cr3["source"], "EOSR");
        assert_eq!(cr3["hash"], hash::sha256_file(&d.path().join(cr3["path"].as_str().unwrap())).unwrap());
        assert_eq!(cr3["taken_from"], "mtime"); // no metadata in this one
        assert!(cr3.get("camera").is_none());
        let jpg = &files[1];
        assert_eq!((jpg["taken_from"].as_str(), jpg["camera"].as_str(), jpg["kind"].as_str()), (Some("exif"), Some("Canon EOS R"), Some("photo")));
        assert!(jpg["taken"].as_str().unwrap().starts_with("2026-07-30T16:58:12"));
        assert_eq!(files[3]["kind"], "sidecar");
        let keys: Vec<&String> = cr3.as_object().unwrap().keys().collect();
        assert_eq!(keys, ["path", "size", "hash", "kind", "source", "taken", "taken_from", "mtime", "sidecars"]);
        // a folder without a month: no source, month null
        mk(d.path(), "2012/scans Peter/a/scan1.tif", b"x");
        let s = build(d.path(), "2012/scans Peter", "mbp-test", None).unwrap();
        assert_eq!((s["month"].clone(), s["files"][0]["source"].clone()), (Value::Null, Value::Null));
    }

    #[test]
    fn a_known_hash_is_reused_and_a_changed_file_hashed_again() {
        let d = tempfile::tempdir().unwrap();
        sample(d.path());
        let mut old = build(d.path(), "2026/07", "mbp-test", None).unwrap();
        old["files"][0]["hash"] = json!("sha256:kept");
        let again = build(d.path(), "2026/07", "mbp-test", Some(&old)).unwrap();
        assert_eq!(again["files"][0]["hash"], "sha256:kept");
        old["files"][0]["mtime"] = json!("2000-01-01T00:00:00+01:00");
        let changed = build(d.path(), "2026/07", "mbp-test", Some(&old)).unwrap();
        assert_ne!(changed["files"][0]["hash"], "sha256:kept");
    }

    #[test]
    fn verify_and_merge() {
        let local = json!({"contract_version": "1.3", "folder": "2026/07", "files": [
            {"path": "2026/07/a.jpg", "size": 3, "hash": "sha256:a", "sidecars": ["2026/07/a.xmp"]},
            {"path": "2026/07/b.jpg", "size": 3, "hash": "sha256:b", "sidecars": []},
            {"path": "2026/07/c.jpg", "size": 3, "hash": "sha256:c", "sidecars": []},
            {"path": "2026/07/d.jpg", "size": 3, "hash": "sha256:d", "sidecars": []},
        ]});
        let remote = json!({"files": [
            {"path": "2026/07/c.jpg", "size": 3, "hash": "sha256:other", "sidecars": []},
            {"path": "2026/07/old.jpg", "size": 5, "hash": "sha256:o", "sidecars": []},
            {"path": "2026/07/gone.jpg", "size": 5, "hash": "sha256:g", "sidecars": []},
        ]});
        let sizes: HashMap<String, u64> = [("2026/07/a.jpg", 3), ("2026/07/b.jpg", 4), ("2026/07/c.jpg", 3), ("2026/07/old.jpg", 5)]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        let v = verify(&local, Some(&remote), &sizes);
        assert_eq!(v.ok.len(), 1);
        assert_eq!(v.missing, ["2026/07/a.xmp", "2026/07/d.jpg"]);
        assert_eq!(v.conflicts, ["2026/07/b.jpg", "2026/07/c.jpg"]);
        let m = merge(&local, Some(&remote), &v.ok, &sizes);
        let paths: Vec<&str> = m["files"].as_array().unwrap().iter().map(|r| r["path"].as_str().unwrap()).collect();
        assert_eq!(paths, ["2026/07/a.jpg", "2026/07/c.jpg", "2026/07/old.jpg"]);
        assert_eq!(m["files"][1]["hash"], "sha256:other"); // the NAS's row of a conflict stays
        assert_eq!(m["contract_version"], "1.3");
    }

    #[test]
    fn written_as_the_cli_writes_it() {
        let d = tempfile::tempdir().unwrap();
        let m = json!({"contract_version": "1.3", "month": null, "files": [{"path": "2026/07/é.jpg", "sidecars": []}]});
        let p = d.path().join(MANIFEST_NAME);
        write(&p, &m).unwrap();
        assert_eq!(
            fs::read_to_string(&p).unwrap(),
            "{\n \"contract_version\": \"1.3\",\n \"month\": null,\n \"files\": [\n  {\n   \"path\": \"2026/07/é.jpg\",\n   \"sidecars\": []\n  }\n ]\n}\n"
        );
        assert_eq!(read(&p), Some(m));
        assert_eq!(fs::read_dir(d.path()).unwrap().count(), 1); // no temporary file left
        assert!(write(&d.path().join("missing/x.json"), &json!({})).is_err());
    }
}
