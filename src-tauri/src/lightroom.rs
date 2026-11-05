//! The file the Lightroom Classic plug-in reads (F023/S01): every list with its photos' paths in
//! the working copy and whether the original is here, so the plug-in only has to apply it.

use crate::lists::List;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

pub const LIGHTROOM_FILE: &str = "familylightbox-lightroom.json";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Photo {
    /// Absolute, in the working copy.
    pub path: String,
    pub here: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LrList {
    pub id: String,
    pub name: String,
    pub year: i32,
    pub archived: bool,
    pub photos: Vec<Photo>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LrFile {
    pub version: u32,
    pub written_at: String,
    pub lists: Vec<LrList>,
}

/// The file's content: per list, each photo by its current path in the library (`path_of`, by
/// hash, from the index), else the list's hint; photos without either are left out.
pub fn build(lists: &BTreeMap<String, List>, library: &Path, path_of: &HashMap<String, String>, now: &str) -> LrFile {
    let mut out: Vec<LrList> = lists
        .values()
        .map(|l| {
            let mut photos: Vec<Photo> = l
                .photos
                .values()
                .filter_map(|m| path_of.get(&m.photo).cloned().or_else(|| m.path.clone()))
                .map(|rel| {
                    let abs = library.join(&rel);
                    Photo { here: abs.is_file(), path: abs.display().to_string() }
                })
                .collect();
            photos.sort_by(|a, b| a.path.cmp(&b.path));
            LrList { id: l.id.clone(), name: l.name.clone(), year: l.year, archived: l.archived, photos }
        })
        .collect();
    out.sort_by(|a, b| b.year.cmp(&a.year).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    LrFile { version: 1, written_at: now.to_string(), lists: out }
}

/// Writes it in `config`, through a temporary name.
pub fn write(config: &Path, file: &LrFile) -> Result<(), String> {
    let v = serde_json::to_value(file).map_err(|e| e.to_string())?;
    crate::manifest::write(&config.join(LIGHTROOM_FILE), &v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lists::{merge, parse_file};
    use std::fs;

    #[test]
    fn the_file_of_two_lists() {
        let d = tempfile::tempdir().unwrap();
        let lib = d.path().join("LibraryLocal");
        fs::create_dir_all(lib.join("2026/10/EOSR")).unwrap();
        fs::write(lib.join("2026/10/EOSR/a.CR3"), b"a").unwrap();
        let ev = |ts: &str, op: &str, list: &str, extra: &str| format!("{{\"v\":1,\"ts\":\"{ts}\",\"dev\":\"m\",\"who\":\"peter\",\"op\":\"{op}\",\"list\":\"{list}\"{extra}}}\n");
        let text = [
            ev("2026-11-01T10:00:00+01:00", "create", "2026/herfst", ",\"name\":\"Herfst\""),
            ev("2026-11-01T10:01:00+01:00", "add", "2026/herfst", ",\"photo\":\"sha256:a\",\"path\":\"2026/09/old/a.CR3\""),
            ev("2026-11-01T10:02:00+01:00", "add", "2026/herfst", ",\"photo\":\"sha256:b\",\"path\":\"2019/07/X/b.JPG\""),
            ev("2026-11-01T10:03:00+01:00", "create", "2019/zomer", ",\"name\":\"Zomer\""),
            ev("2026-11-01T10:04:00+01:00", "archive", "2019/zomer", ""),
        ]
        .concat();
        let lists = merge(parse_file(&text, "m").0);
        let path_of: HashMap<String, String> = [("sha256:a".to_string(), "2026/10/EOSR/a.CR3".to_string())].into_iter().collect();
        let f = build(&lists, &lib, &path_of, "2026-11-05T12:00:00+01:00");
        assert_eq!(f.lists.iter().map(|l| (l.name.as_str(), l.archived)).collect::<Vec<_>>(), [("Herfst", false), ("Zomer", true)]);
        let herfst = &f.lists[0].photos;
        // the index's path wins over the list's hint; the hint when the index doesn't know it
        assert_eq!(herfst[0].path, lib.join("2019/07/X/b.JPG").display().to_string());
        assert!(!herfst[0].here);
        assert_eq!((herfst[1].path.clone(), herfst[1].here), (lib.join("2026/10/EOSR/a.CR3").display().to_string(), true));
        write(d.path(), &f).unwrap();
        let back: serde_json::Value = serde_json::from_slice(&fs::read(d.path().join(LIGHTROOM_FILE)).unwrap()).unwrap();
        assert_eq!(back["lists"][1]["archived"], true);
    }
}
