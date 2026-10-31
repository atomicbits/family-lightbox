//! Where the library, the previews, the lists and the exports are on the NAS (F019/S01, data
//! contract 2): `familylightbox.json` in the root of the home share, read by every Mac, with a
//! copy kept in the config folder for while the NAS is away. Without the file, the share names in
//! the Mac's own settings apply.

use crate::config::Shares;
use crate::mounts::{self, RawMount};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// On the NAS, in the root of the home share.
pub const LAYOUT_FILE: &str = "familylightbox.json";
/// The copy kept on this Mac.
pub const LAYOUT_COPY: &str = "familylightbox-layout.json";

/// A share and a folder inside it (`""`: its root).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub share: String,
    #[serde(default)]
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NasLayout {
    #[serde(default)]
    pub contract_version: String,
    #[serde(default)]
    pub name: String,
    /// The share this file lives in from now on (F019/S04): a Mac whose home share is another one
    /// moves its home there once that share has a valid file. Empty: stays where it is.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub home: String,
    pub library: Location,
    pub previews: Location,
    pub lists: Location,
    pub exports: Location,
}

impl NasLayout {
    /// Four shares, each at its root: the layout before this file existed.
    pub fn from_shares(s: &Shares) -> NasLayout {
        let at = |share: &str| Location { share: share.to_string(), path: String::new() };
        NasLayout {
            contract_version: String::new(),
            name: String::new(),
            home: String::new(),
            library: at(&s.library),
            previews: at(&s.previews),
            lists: at(&s.lists),
            exports: at(&s.exports),
        }
    }

    /// One share holding the four as subfolders: a new library (F019/S02).
    pub fn in_one_share(share: &str, name: &str) -> NasLayout {
        let at = |path: &str| Location { share: share.to_string(), path: path.to_string() };
        NasLayout {
            contract_version: "1.4".into(),
            name: name.to_string(),
            home: String::new(),
            library: at("Library"),
            previews: at("Previews"),
            lists: at("Lists"),
            exports: at("Exports"),
        }
    }

    /// The locations in connect order: the lists first, because writing them needs it.
    pub fn locations(&self) -> [(&'static str, &Location); 4] {
        [("lists", &self.lists), ("library", &self.library), ("previews", &self.previews), ("exports", &self.exports)]
    }
}

fn location_problem(what: &str, l: &Location) -> Option<String> {
    if l.share.trim().is_empty() || l.share.contains('/') {
        return Some(format!("{what}: de gedeelde map '{}' is geen naam", l.share));
    }
    let bad = l.path.starts_with('/') || l.path.contains('\\') || l.path.split('/').any(|c| c == "..");
    bad.then(|| format!("{what}: het pad '{}' moet binnen de gedeelde map blijven, met '/'", l.path))
}

/// The file's content, checked: every location a share and a path inside it.
pub fn parse(text: &str) -> Result<NasLayout, String> {
    let l: NasLayout = serde_json::from_str(text).map_err(|e| format!("{LAYOUT_FILE} is niet geldig: {e}"))?;
    for (what, loc) in l.locations() {
        if let Some(p) = location_problem(what, loc) {
            return Err(format!("{LAYOUT_FILE}: {p}"));
        }
    }
    Ok(l)
}

/// The copy kept on this Mac, if there is a valid one.
pub fn read_copy(config: &Path) -> Option<NasLayout> {
    parse(&fs::read_to_string(config.join(LAYOUT_COPY)).ok()?).ok()
}

/// Where the layout in use came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// Read from the NAS just now.
    Nas,
    /// The home share isn't connected: the copy read last.
    Copy,
    /// No file on the NAS (or none read yet): the Mac's own share names.
    Settings,
}

/// Reads the file from the home share when it is mounted and keeps the copy; the file gone from
/// the NAS removes the copy, so the Mac's settings apply again. A broken file is an error and the
/// copy stays.
pub fn refresh(config: &Path, raw: &[RawMount], host: &str, home: &str) -> Result<Source, String> {
    let list = mounts::mounts_of(raw, host, home);
    let Some(m) = mounts::NasMounts::chosen(&list) else {
        return Ok(if read_copy(config).is_some() { Source::Copy } else { Source::Settings });
    };
    let file = Path::new(&m.mount_point).join(LAYOUT_FILE);
    match fs::read_to_string(&file) {
        Ok(text) => {
            let layout = parse(&text)?;
            let copy = serde_json::to_string_pretty(&layout).map_err(|e| e.to_string())? + "\n";
            fs::create_dir_all(config).map_err(|e| format!("{}: {e}", config.display()))?;
            let tmp = config.join(format!("{LAYOUT_COPY}.tmp"));
            fs::write(&tmp, copy).and_then(|_| fs::rename(&tmp, config.join(LAYOUT_COPY))).map_err(|e| format!("{}: {e}", config.display()))?;
            Ok(Source::Nas)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let _ = fs::remove_file(config.join(LAYOUT_COPY));
            Ok(Source::Settings)
        }
        Err(e) => Err(format!("{}: {e}", file.display())),
    }
}

/// The share the copy says the file lives in now, when that isn't this Mac's home share yet and
/// that share is mounted with a valid file: the home to move to (F019/S04).
pub fn next_home(config: &Path, raw: &[RawMount], host: &str, home: &str) -> Option<String> {
    let next = read_copy(config)?.home;
    if next.is_empty() || next == home {
        return None;
    }
    let list = mounts::mounts_of(raw, host, &next);
    let m = mounts::NasMounts::chosen(&list)?;
    parse(&fs::read_to_string(Path::new(&m.mount_point).join(LAYOUT_FILE)).ok()?).ok()?;
    Some(next)
}

/// One location as a move would find it (F019/S04).
#[derive(Debug, Clone, Serialize)]
pub struct Found {
    pub what: &'static str,
    pub location: Location,
    /// The folder it resolves to, if its share is mounted.
    pub root: Option<String>,
    /// What is there, or what is missing.
    pub note: String,
    pub ok: bool,
}

/// A device and whether its app reads this file.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceVersion {
    pub name: String,
    pub owner: String,
    pub app: String,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DryCheck {
    pub problem: Option<String>,
    pub name: String,
    pub found: Vec<Found>,
    pub devices: Vec<DeviceVersion>,
}

/// The first version that reads `familylightbox.json`.
pub const FIRST_READER: &str = "0.5.0";

fn version_at_least(v: &str, min: &str) -> bool {
    let parts = |s: &str| s.split('.').map(|p| p.parse::<u64>().unwrap_or(0)).collect::<Vec<_>>();
    !v.is_empty() && parts(v) >= parts(min)
}

/// What the app would find with the file in `share` (F019/S04), without using it: each location's
/// root and what is in it, and which devices still have an app that doesn't read the file.
pub fn dry_check(raw: &[RawMount], host: &str, share: &str) -> DryCheck {
    let mut out = DryCheck { problem: None, name: String::new(), found: Vec::new(), devices: Vec::new() };
    let list = mounts::mounts_of(raw, host, share);
    let Some(m) = mounts::NasMounts::chosen(&list) else {
        out.problem = Some(format!("de gedeelde map '{share}' is niet verbonden"));
        return out;
    };
    let layout = match fs::read_to_string(Path::new(&m.mount_point).join(LAYOUT_FILE)).map_err(|e| format!("{LAYOUT_FILE} in '{share}': {e}")).and_then(|t| parse(&t)) {
        Ok(l) => l,
        Err(e) => {
            out.problem = Some(e);
            return out;
        }
    };
    out.name = layout.name.clone();
    for (what, loc) in layout.locations() {
        let root = mounts::NasMounts::chosen(&mounts::mounts_of(raw, host, &loc.share)).map(|m| Path::new(&m.mount_point).join(&loc.path));
        let (note, ok) = match &root {
            None => (format!("de gedeelde map '{}' is niet verbonden", loc.share), false),
            Some(r) if !r.is_dir() => ("de map bestaat niet".to_string(), false),
            Some(r) => match what {
                "library" => {
                    let years = fs::read_dir(r).into_iter().flatten().flatten().filter(|e| {
                        let n = e.file_name().to_string_lossy().into_owned();
                        n.len() == 4 && n.bytes().all(|c| c.is_ascii_digit())
                    }).count();
                    (format!("{years} jaarmappen"), years > 0)
                }
                "lists" if r.join(crate::nas::DEVICES_FILE).is_file() => ("devices.json gevonden".into(), true),
                "lists" => ("geen devices.json".into(), false),
                _ => ("gevonden".into(), true),
            },
        };
        if what == "lists"
            && let Some(r) = root.as_ref().filter(|_| ok) {
                out.devices = crate::nas::known_devices(r)
                    .into_iter()
                    .map(|k| DeviceVersion { ok: version_at_least(&k.app, FIRST_READER), name: k.name, owner: k.owner, app: k.app })
                    .collect();
            }
        out.found.push(Found { what, location: loc.clone(), root: root.map(|r| r.display().to_string()), note, ok });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const ONE: &str = r#"{"contract_version": "1.4", "name": "Onze foto's", "x-note": 1,
        "library": {"share": "Fotos", "path": "Library"}, "previews": {"share": "Fotos", "path": "Previews"},
        "lists": {"share": "Fotos", "path": "Lists"}, "exports": {"share": "Fotos", "path": "Exports"}}"#;

    #[test]
    fn reading_the_file() {
        let l = parse(ONE).unwrap();
        assert_eq!((l.name.as_str(), l.library.share.as_str(), l.library.path.as_str()), ("Onze foto's", "Fotos", "Library"));
        // a share's root needs no path; another share per location is fine
        let four = r#"{"library": {"share": "Library"}, "previews": {"share": "Previews"}, "lists": {"share": "Lists"}, "exports": {"share": "photo", "path": "FamilyLightbox/Exports"}}"#;
        let l = parse(four).unwrap();
        assert_eq!((l.library.path.as_str(), l.exports.share.as_str()), ("", "photo"));
        assert_eq!(NasLayout::from_shares(&Shares::default()).lists, Location { share: "Lists".into(), path: "".into() });
        for bad in [r#""path": "../x""#, r#""path": "/abs""#, r#""path": "a\\b""#] {
            let text = ONE.replace(r#""path": "Library""#, bad);
            assert!(parse(&text).is_err(), "{bad}");
        }
        assert!(parse(&ONE.replace(r#"{"share": "Fotos", "path": "Lists"}"#, r#"{"share": ""}"#)).is_err());
        assert!(parse("{broken").unwrap_err().contains("niet geldig"));
    }

    fn mount(from: &str, on: &Path) -> RawMount {
        RawMount { from: from.into(), on: on.to_path_buf(), fs_type: "smbfs".into() }
    }

    #[test]
    fn read_from_the_home_share_and_kept() {
        let d = tempfile::tempdir().unwrap();
        let (config, share) = (d.path().join("config"), d.path().join("mnt/Fotos"));
        fs::create_dir_all(&share).unwrap();
        let raw = [mount("//anna@nas1/Fotos", &share)];
        // no file yet: the settings apply
        assert_eq!(refresh(&config, &raw, "nas1", "Fotos").unwrap(), Source::Settings);
        assert!(read_copy(&config).is_none());
        fs::write(share.join(LAYOUT_FILE), ONE).unwrap();
        assert_eq!(refresh(&config, &raw, "nas1", "Fotos").unwrap(), Source::Nas);
        assert_eq!(read_copy(&config).unwrap().name, "Onze foto's");
        // the NAS away: the copy
        assert_eq!(refresh(&config, &[], "nas1", "Fotos").unwrap(), Source::Copy);
        // broken on the NAS: an error, the copy stays
        fs::write(share.join(LAYOUT_FILE), "{").unwrap();
        assert!(refresh(&config, &raw, "nas1", "Fotos").is_err());
        assert!(read_copy(&config).is_some());
        // gone from the NAS: the copy goes, the settings apply again
        fs::remove_file(share.join(LAYOUT_FILE)).unwrap();
        assert_eq!(refresh(&config, &raw, "nas1", "Fotos").unwrap(), Source::Settings);
        assert!(!config.join(LAYOUT_COPY).exists());
    }

    #[test]
    fn a_moved_library_takes_its_macs_along() {
        let d = tempfile::tempdir().unwrap();
        let (config, lists, fotos) = (d.path().join("config"), d.path().join("mnt/Lists"), d.path().join("mnt/Fotos"));
        fs::create_dir_all(&lists).unwrap();
        // the old home's file points at the new share, and says it lives there now
        let moved = ONE.replacen("{\"contract_version\": \"1.4\",", "{\"contract_version\": \"1.4\", \"home\": \"Fotos\",", 1);
        fs::write(lists.join(LAYOUT_FILE), &moved).unwrap();
        let raw_old = [mount("//anna@nas1/Lists", &lists)];
        refresh(&config, &raw_old, "nas1", "Lists").unwrap();
        // the new share isn't mounted yet, or has no file yet: stay
        assert_eq!(next_home(&config, &raw_old, "nas1", "Lists"), None);
        fs::create_dir_all(&fotos).unwrap();
        let raw = [mount("//anna@nas1/Lists", &lists), mount("//anna@nas1/Fotos", &fotos)];
        assert_eq!(next_home(&config, &raw, "nas1", "Lists"), None);
        fs::write(fotos.join(LAYOUT_FILE), &moved).unwrap();
        assert_eq!(next_home(&config, &raw, "nas1", "Lists").as_deref(), Some("Fotos"));
        assert_eq!(next_home(&config, &raw, "nas1", "Fotos"), None, "already there");
    }

    #[test]
    fn the_check_before_a_move() {
        let d = tempfile::tempdir().unwrap();
        let fotos = d.path().join("mnt/Fotos");
        fs::create_dir_all(fotos.join("Library/2026/07")).unwrap();
        fs::create_dir_all(fotos.join("Previews")).unwrap();
        fs::create_dir_all(fotos.join("Lists")).unwrap();
        fs::write(
            fotos.join("Lists/devices.json"),
            r#"{"devices": [{"name": "mbp-anna", "owner": "anna", "app": "0.5.0"}, {"name": "mbp-jan", "owner": "jan"}]}"#,
        )
        .unwrap();
        fs::write(fotos.join(LAYOUT_FILE), ONE).unwrap();
        let raw = [mount("//anna@nas1/Fotos", &fotos)];
        let c = dry_check(&raw, "nas1", "Fotos");
        assert!(c.problem.is_none(), "{:?}", c.problem);
        let by = |w: &str| c.found.iter().find(|f| f.what == w).unwrap();
        assert!(by("library").ok && by("library").note == "1 jaarmappen");
        assert!(by("lists").ok && by("previews").ok);
        assert!(!by("exports").ok && by("exports").note == "de map bestaat niet");
        let versions: Vec<(&str, bool)> = c.devices.iter().map(|d| (d.name.as_str(), d.ok)).collect();
        assert_eq!(versions, [("mbp-anna", true), ("mbp-jan", false)]);
        assert!(dry_check(&[], "nas1", "Fotos").problem.unwrap().contains("niet verbonden"));
        assert!(version_at_least("0.10.0", "0.5.0") && !version_at_least("0.4.4", "0.5.0") && !version_at_least("", "0.5.0"));
    }
}
