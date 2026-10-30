//! Where new photos come from on this Mac (F018/S03): the subfolders of a drop folder, folders
//! another app keeps in sync, and cards. No source names are in the code: a source's name and
//! its layout come from the library itself (decision 6), and a new source gets day folders.

use crate::capture;
use crate::config::Import;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// How a source's files are laid out below `<year>/<month>/<source>/` (data contract 3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Layout {
    /// `photos/` and `videos/` below the source.
    pub split: bool,
    /// A day folder `YYYY-MM-DD/` before the file.
    pub day: bool,
}

/// What a source the library doesn't have yet gets.
pub const NEW: Layout = Layout { split: false, day: true };

fn is_day(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10 && b[4] == b'-' && b[7] == b'-' && b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

/// The layout of each source, by what most of its files do; `paths` are `(source, path)`.
pub fn layouts(paths: &[(String, String)]) -> HashMap<String, Layout> {
    let mut count: HashMap<&str, (usize, usize, usize)> = HashMap::new();
    for (source, path) in paths {
        let parts: Vec<&str> = path.split('/').collect();
        // <year>/<month>/<source>/…/<file>
        if parts.len() < 4 || parts[2] != source {
            continue;
        }
        let mut middle = &parts[3..parts.len() - 1];
        let split = matches!(middle.first(), Some(&"photos") | Some(&"videos"));
        if split {
            middle = &middle[1..];
        }
        let day = middle.first().is_some_and(|d| is_day(d));
        let c = count.entry(source).or_default();
        c.0 += 1;
        c.1 += split as usize;
        c.2 += day as usize;
    }
    count.into_iter().map(|(s, (n, split, day))| (s.to_string(), Layout { split: split * 2 > n, day: day * 2 > n })).collect()
}

/// `^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$` (data contract 3.1); none when fine.
pub fn source_name_problem(name: &str) -> Option<String> {
    let ok = !name.is_empty()
        && name.len() <= 64
        && name.as_bytes()[0].is_ascii_alphanumeric()
        && name.bytes().all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c));
    (!ok).then(|| format!("'{name}': letters, cijfers, '.', '_' en '-', beginnend met een letter of cijfer, hoogstens 64"))
}

/// A source name from a camera's model: `GoPro HERO13 Black` → `GoPro-HERO13-Black`.
pub fn name_from_model(camera: &str) -> String {
    let mut s = String::new();
    for c in camera.chars() {
        if c.is_ascii_alphanumeric() || c == '.' || c == '_' {
            s.push(c);
        } else if !s.ends_with('-') {
            s.push('-');
        }
    }
    let s: String = s.trim_start_matches(|c: char| !c.is_ascii_alphanumeric()).chars().take(64).collect();
    s.trim_end_matches('-').to_string()
}

/// The source to propose for a card's camera: where the library has most of that camera's files,
/// among sources with day folders (as a card's are), else a name from the model.
pub fn propose(camera: &str, sources: &[(String, i64)], layouts: &HashMap<String, Layout>) -> String {
    sources
        .iter()
        .find(|(s, _)| layouts.get(s).is_some_and(|l| l.day))
        .map(|(s, _)| s.clone())
        .unwrap_or_else(|| name_from_model(camera))
}

/// Something the import can take files from.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Found {
    /// `drop`, `synced` or `card`.
    pub kind: &'static str,
    pub path: String,
    /// The source in the library (a synced folder's photos).
    pub name: String,
    /// A synced folder's source for videos; empty otherwise.
    pub videos: String,
    pub layout: Layout,
    /// The library has this source already.
    pub known: bool,
    pub camera: Option<String>,
    pub ignore: bool,
    /// A problem with the folder or the name; nothing can be imported from it then.
    pub problem: Option<String>,
}

/// The folder the import moves what it took to; never a source itself.
pub const IMPORTED: &str = "_imported";

/// What the import can take from on this Mac: the drop folders' subfolders, the synced folders,
/// and the cards mounted under `volumes` (a folder with `DCIM`). `camera_sources` gives, for a
/// camera, the sources it is under in the library.
pub fn discover(import: &Import, volumes: &Path, layouts: &HashMap<String, Layout>, camera_sources: &dyn Fn(&str) -> Vec<(String, i64)>) -> Vec<Found> {
    let layout = |name: &str| layouts.get(name).copied().unwrap_or(NEW);
    let mut out = Vec::new();
    for folder in &import.drop_folders {
        let Ok(rd) = fs::read_dir(folder) else {
            out.push(Found { kind: "drop", path: folder.clone(), name: String::new(), videos: String::new(), layout: NEW, known: false, camera: None, ignore: false, problem: Some("map niet gevonden".into()) });
            continue;
        };
        let mut subs: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
        subs.sort();
        for sub in subs {
            let folder_name = sub.file_name().unwrap_or_default().to_string_lossy().into_owned();
            if folder_name == IMPORTED || folder_name.starts_with('.') {
                continue;
            }
            let set = import.sources.get(&folder_name).cloned().unwrap_or_default();
            let name = if set.name.is_empty() { folder_name.clone() } else { set.name.clone() };
            out.push(Found {
                kind: "drop",
                path: sub.display().to_string(),
                layout: layout(&name),
                known: layouts.contains_key(&name),
                problem: source_name_problem(&name),
                name,
                videos: String::new(),
                camera: None,
                ignore: set.ignore,
            });
        }
    }
    for s in &import.synced {
        let problem = if !Path::new(&s.path).is_dir() {
            Some("map niet gevonden".into())
        } else {
            source_name_problem(&s.photos).or_else(|| source_name_problem(&s.videos))
        };
        out.push(Found { kind: "synced", path: s.path.clone(), name: s.photos.clone(), videos: s.videos.clone(), layout: layout(&s.photos), known: layouts.contains_key(&s.photos), camera: None, ignore: false, problem });
    }
    let mut cards: Vec<PathBuf> = fs::read_dir(volumes).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.join("DCIM").is_dir()).collect();
    cards.sort();
    for card in cards {
        let camera = card_camera(&card.join("DCIM"));
        let name = camera.as_deref().map(|c| propose(c, &camera_sources(c), layouts)).unwrap_or_default();
        let problem = if name.is_empty() { Some("geen camera herkend: kies een naam".into()) } else { source_name_problem(&name) };
        out.push(Found { kind: "card", path: card.display().to_string(), layout: layout(&name), known: layouts.contains_key(&name), name, videos: String::new(), camera, ignore: false, problem });
    }
    out
}

/// The camera of the first photo or video on a card that names one (at most 20 looked at).
fn card_camera(dcim: &Path) -> Option<String> {
    let mut seen = 0;
    let mut stack = vec![dcim.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let mut entries: Vec<PathBuf> = fs::read_dir(&dir).ok()?.flatten().map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                stack.push(p);
            } else if matches!(capture::kind_of(&p), "photo" | "video") {
                if let Some(c) = capture::read(&p).camera {
                    return Some(c);
                }
                seen += 1;
                if seen >= 20 {
                    return None;
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::fixtures::{jpeg_file, tiff};
    use crate::config::{SourceSettings, Synced};

    fn p(source: &str, path: &str) -> (String, String) {
        (source.to_string(), path.to_string())
    }

    #[test]
    fn the_layout_comes_from_where_the_files_are() {
        let paths = [
            p("Cam", "2026/07/Cam/photos/2026-07-30/a.CR3"),
            p("Cam", "2026/07/Cam/videos/2026-07-30/a.MP4"),
            p("Cam", "2026/07/Cam/b.CR3"), // the odd one out doesn't decide
            p("Action", "2026/07/Action/2026-07-12/GX01.MP4"),
            p("Phone", "2026/07/Phone/IMG_1.HEIC"),
            p("Old", "2014/01/Old/photos/x.NEF"),
            p("Elsewhere", "2026/07/Other/x.jpg"), // the source isn't in the path: ignored
        ];
        let l = layouts(&paths);
        assert_eq!(l["Cam"], Layout { split: true, day: true });
        assert_eq!(l["Action"], Layout { split: false, day: true });
        assert_eq!(l["Phone"], Layout { split: false, day: false });
        assert_eq!(l["Old"], Layout { split: true, day: false });
        assert!(!l.contains_key("Elsewhere"));
    }

    #[test]
    fn names_and_proposals() {
        assert_eq!(name_from_model("GoPro HERO13 Black"), "GoPro-HERO13-Black");
        assert_eq!(name_from_model("  Insta360 X5 (2) "), "Insta360-X5-2");
        assert!(source_name_problem("Hero13").is_none() && source_name_problem("Insta360-X5").is_none());
        for bad in ["", "-x", "a/b", "a b", &"a".repeat(65)] {
            assert!(source_name_problem(bad).is_some(), "{bad}");
        }
        let layouts = layouts(&[p("Hero13", "2026/08/Hero13/2026-08-03/a.MP4"), p("Phone", "2026/08/Phone/a.MP4")]);
        let sources = vec![("Phone".to_string(), 50), ("Hero13".to_string(), 10)];
        // a card goes where the camera's files with day folders are, not to a phone's stream
        assert_eq!(propose("GoPro HERO13 Black", &sources, &layouts), "Hero13");
        assert_eq!(propose("GoPro HERO13 Black", &sources[..1], &layouts), "GoPro-HERO13-Black");
    }

    #[test]
    fn what_is_found_on_this_mac() {
        let d = tempfile::tempdir().unwrap();
        let drop = d.path().join("Offload");
        for sub in ["Hero13", "Downloads 2026", "_imported", ".hidden", "camping"] {
            fs::create_dir_all(drop.join(sub)).unwrap();
        }
        let synced = d.path().join("Phone sync");
        fs::create_dir_all(&synced).unwrap();
        let volumes = d.path().join("Volumes");
        let card = volumes.join("GOPRO");
        fs::create_dir_all(card.join("DCIM/100GOPRO")).unwrap();
        fs::write(card.join("DCIM/100GOPRO/GOPR0001.JPG"), jpeg_file(&tiff(true, &[(0x010F, "GoPro"), (0x0110, "HERO13 Black")], &[]))).unwrap();
        fs::create_dir_all(volumes.join("Library")).unwrap(); // a NAS share, no DCIM

        let mut import = Import { drop_folders: vec![drop.display().to_string(), d.path().join("gone").display().to_string()], ..Default::default() };
        import.synced.push(Synced { path: synced.display().to_string(), photos: "Phone-photos".into(), videos: "Phone-videos".into() });
        import.sources.insert("camping".into(), SourceSettings { name: "camping-2026".into(), ignore: true, ..Default::default() });
        let layouts = layouts(&[p("Hero13", "2026/08/Hero13/2026-08-03/a.MP4"), p("Phone-photos", "2026/08/Phone-photos/a.HEIC")]);
        let found = discover(&import, &volumes, &layouts, &|c| if c == "GoPro HERO13 Black" { vec![("Hero13".into(), 81)] } else { vec![] });

        let names: Vec<(&str, &str)> = found.iter().map(|f| (f.kind, f.name.as_str())).collect();
        assert_eq!(names, [("drop", "Downloads 2026"), ("drop", "Hero13"), ("drop", "camping-2026"), ("drop", ""), ("synced", "Phone-photos"), ("card", "Hero13")]);
        assert!(found[0].problem.is_some(), "a space isn't allowed in a source name");
        assert_eq!((found[1].known, found[1].layout), (true, Layout { split: false, day: true }));
        assert!(found[2].ignore && !found[2].known && found[2].layout == NEW);
        assert_eq!(found[3].problem.as_deref(), Some("map niet gevonden"));
        assert_eq!((found[4].videos.as_str(), found[4].layout.day), ("Phone-videos", false));
        assert_eq!(found[5].camera.as_deref(), Some("GoPro HERO13 Black"));
    }
}
