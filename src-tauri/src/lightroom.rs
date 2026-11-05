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

/// A photo's rating for Lightroom (S04): a heart is 5 stars, else its stars.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LrRating {
    pub path: String,
    pub rating: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LrFile {
    pub version: u32,
    pub written_at: String,
    pub lists: Vec<LrList>,
    /// The photos with stars or a heart whose original is here, by path (S04).
    pub ratings: Vec<LrRating>,
}

/// The ratings (S04): every photo with a heart (5 stars) or stars, whose original is in the
/// working copy; hidden photos aren't passed on, and a photo without either is left out, so it
/// keeps its stars in Lightroom.
pub fn ratings(marks: &crate::marks::Marks, library: &Path, path_of: &HashMap<String, String>) -> Vec<LrRating> {
    let mut out: Vec<LrRating> = marks
        .ratings
        .keys()
        .chain(marks.hearts.keys())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .filter_map(|hash| {
            let hearted = marks.hearts.get(hash).is_some_and(|who| !who.is_empty());
            let stars = marks.ratings.get(hash).map_or(0, |r| r.stars);
            let rating = if hearted { 5 } else { stars };
            let abs = library.join(path_of.get(hash)?);
            (rating > 0 && abs.is_file()).then(|| LrRating { path: abs.display().to_string(), rating })
        })
        .collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
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
    LrFile { version: 1, written_at: now.to_string(), lists: out, ratings: vec![] }
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

    #[test]
    fn ratings_with_hearts_as_five_stars() {
        use crate::marks::{Marks, Rating};
        let d = tempfile::tempdir().unwrap();
        let lib = d.path().join("LibraryLocal");
        for f in ["2026/10/a.JPG", "2026/10/b.JPG", "2026/10/c.JPG"] {
            fs::create_dir_all(lib.join(f).parent().unwrap()).unwrap();
            fs::write(lib.join(f), b"x").unwrap();
        }
        let r = |stars: u8| Rating { stars, by: "peter".into(), at: "".into() };
        let mut m = Marks::default();
        m.ratings.insert("sha256:a".into(), r(3));
        m.ratings.insert("sha256:b".into(), r(2));
        m.hearts.insert("sha256:b".into(), [("lynn".to_string(), "".to_string())].into());
        m.hearts.insert("sha256:c".into(), [("peter".to_string(), "".to_string())].into());
        m.ratings.insert("sha256:away".into(), r(4));
        m.ratings.insert("sha256:unknown".into(), r(5));
        m.ratings.insert("sha256:none".into(), r(0));
        let path_of: HashMap<String, String> = [("a", "2026/10/a.JPG"), ("b", "2026/10/b.JPG"), ("c", "2026/10/c.JPG"), ("away", "2019/01/away.JPG"), ("none", "2026/10/a.JPG")]
            .iter()
            .map(|(h, p)| (format!("sha256:{h}"), p.to_string()))
            .collect();
        let got: Vec<(String, u8)> = ratings(&m, &lib, &path_of).into_iter().map(|x| (x.path.rsplit('/').next().unwrap().to_string(), x.rating)).collect();
        // stars; a heart is 5, also with stars; not here, unknown or without stars: left out
        assert_eq!(got, [("a.JPG".to_string(), 3), ("b.JPG".to_string(), 5), ("c.JPG".to_string(), 5)]);
    }
}
