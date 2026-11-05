//! Lightroom's edited versions of photos in lists (F025/S03, data contract 4, 1.7): the plug-in
//! renders them into `~/.config/media/lightroom-out/` with a JSON per variant; the app puts them
//! next to the previews, on the NAS through a temporary name, the small one on this Mac too.

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The plug-in's folder, in the config folder.
pub const OUT_DIR: &str = "lightroom-out";

/// `lr`, or `lr-<slug>` for a virtual copy.
pub fn valid_variant(v: &str) -> bool {
    v == "lr"
        || v.strip_prefix("lr-").is_some_and(|s| {
            !s.is_empty() && !s.starts_with('-') && !s.ends_with('-') && s.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        })
}

/// The suffixes of `file`'s renditions in `dir` (`.lr.jpg`, `.lr-zwart-wit.full.jpg`, …).
pub fn suffixes(dir: &Path, file: &str) -> Vec<String> {
    let prefix = format!("{file}.");
    let mut out: Vec<String> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let rest = name.strip_prefix(&prefix)?.strip_suffix(".jpg")?;
            let variant = rest.strip_suffix(".full").unwrap_or(rest);
            valid_variant(variant).then(|| format!(".{rest}.jpg"))
        })
        .collect();
    out.sort();
    out
}

#[derive(Debug, Deserialize)]
struct Note {
    path: String,
    variant: String,
    #[serde(default)]
    edited: String,
    #[serde(default)]
    small: String,
    #[serde(default)]
    full: String,
    #[serde(default)]
    removed: bool,
}

#[derive(Debug, Default, PartialEq)]
pub struct TakenIn {
    pub taken: usize,
    pub removed: usize,
    pub problems: Vec<String>,
    /// Notes left for later: the NAS's previews aren't there.
    pub waiting: usize,
}

fn is_jpeg(p: &Path) -> bool {
    use std::io::Read;
    let mut b = [0u8; 3];
    fs::File::open(p).and_then(|mut f| f.read_exact(&mut b)).is_ok() && b == [0xFF, 0xD8, 0xFF]
}

fn copy_to(from: &Path, to: &Path, time: Option<SystemTime>) -> std::io::Result<()> {
    if let Some(d) = to.parent() {
        fs::create_dir_all(d)?;
    }
    let mut name = to.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{}.tmp", std::process::id()));
    let tmp = to.with_file_name(name);
    let r = fs::copy(from, &tmp).and_then(|_| {
        if let Some(t) = time {
            fs::File::options().write(true).open(&tmp)?.set_modified(t)?;
        }
        fs::rename(&tmp, to)
    });
    if r.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    r
}

fn remove_note(out: &Path, json: &Path, n: Option<&Note>) {
    if let Some(n) = n {
        for f in [&n.small, &n.full] {
            if !f.is_empty() && !f.contains('/') {
                let _ = fs::remove_file(out.join(f));
            }
        }
    }
    let _ = fs::remove_file(json);
}

/// One note: where it goes, or why not.
fn take_one(out: &Path, n: &Note, library: &Path, nas: &Path, local: &Path) -> Result<bool, String> {
    if !valid_variant(&n.variant) {
        return Err(format!("onbekende variant '{}'", n.variant));
    }
    let rel = Path::new(&n.path).strip_prefix(library).map_err(|_| format!("{} staat niet in de werkkopie", n.path))?;
    let rel = rel.to_string_lossy().into_owned();
    if !rel.split('/').next().is_some_and(|y| y.len() == 4 && y.bytes().all(|c| c.is_ascii_digit())) || rel.split('/').any(|c| c == "..") {
        return Err(format!("{rel}: geen pad in de bibliotheek"));
    }
    let small = |root: &Path| root.join(format!("{rel}.{}.jpg", n.variant));
    let full = |root: &Path| root.join(format!("{rel}.{}.full.jpg", n.variant));
    if n.removed {
        for p in [small(nas), full(nas), small(local)] {
            match fs::remove_file(&p) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(format!("{}: {e}", p.display())),
            }
        }
        return Ok(false);
    }
    let (s, f) = (out.join(&n.small), out.join(&n.full));
    if n.small.contains('/') || n.full.contains('/') || !is_jpeg(&s) || !is_jpeg(&f) {
        return Err(format!("{rel}: de bewerkte versie is geen JPEG"));
    }
    let time = chrono::DateTime::parse_from_rfc3339(&n.edited).ok().map(SystemTime::from);
    for (from, to) in [(&f, full(nas)), (&s, small(nas)), (&s, small(local))] {
        copy_to(from, &to, time).map_err(|e| format!("{}: {e}", to.display()))?;
    }
    Ok(true)
}

/// Takes in what the plug-in rendered (S03). Without the NAS's previews (`nas` None) nothing is
/// taken and the notes wait; a note that can't be taken is reported and removed, so it doesn't
/// come back each time.
pub fn take_in(config: &Path, library: &Path, nas: Option<&Path>, local: &Path) -> TakenIn {
    let out = config.join(OUT_DIR);
    let mut r = TakenIn::default();
    let mut notes: Vec<PathBuf> = fs::read_dir(&out)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    notes.sort();
    let Some(nas) = nas else {
        r.waiting = notes.len();
        return r;
    };
    for json in notes {
        let note = fs::read_to_string(&json).ok().and_then(|t| serde_json::from_str::<Note>(&t).ok());
        let Some(n) = note else {
            r.problems.push(format!("{}: niet leesbaar", json.display()));
            remove_note(&out, &json, None);
            continue;
        };
        match take_one(&out, &n, library, nas, local) {
            Ok(true) => r.taken += 1,
            Ok(false) => r.removed += 1,
            Err(e) => r.problems.push(e),
        }
        remove_note(&out, &json, Some(&n));
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 1, 2, 3];

    fn note(out: &Path, id: &str, path: &str, variant: &str) {
        fs::create_dir_all(out).unwrap();
        fs::write(out.join(format!("{id}.jpg")), JPEG).unwrap();
        fs::write(out.join(format!("{id}.full.jpg")), JPEG).unwrap();
        let j = serde_json::json!({"version": 1, "path": path, "variant": variant, "edited": "2026-11-05T10:00:00+01:00",
            "small": format!("{id}.jpg"), "full": format!("{id}.full.jpg")});
        fs::write(out.join(format!("{id}.json")), j.to_string()).unwrap();
    }

    #[test]
    fn variants_and_suffixes() {
        for ok in ["lr", "lr-zwart-wit", "lr-copy-2"] {
            assert!(valid_variant(ok), "{ok}");
        }
        for bad in ["", "lr-", "lr-Zwart", "lr--x-", "x", "lr-a/b", "lr-a.b"] {
            assert!(!valid_variant(bad), "{bad}");
        }
        let d = tempfile::tempdir().unwrap();
        for f in ["A.CR3.jpg", "A.CR3.t.jpg", "A.CR3.lr.jpg", "A.CR3.lr.full.jpg", "A.CR3.lr-zw.jpg", "A.CR3.lr-zw.full.jpg", "A.CR3.x.jpg", "AB.CR3.lr.jpg"] {
            fs::write(d.path().join(f), b"x").unwrap();
        }
        assert_eq!(suffixes(d.path(), "A.CR3"), [".lr-zw.full.jpg", ".lr-zw.jpg", ".lr.full.jpg", ".lr.jpg"]);
    }

    #[test]
    fn taken_in_to_the_nas_and_this_mac() {
        let d = tempfile::tempdir().unwrap();
        let (config, lib, nas, local) = (d.path().join("config"), d.path().join("LibraryLocal"), d.path().join("nas/Previews"), d.path().join("Previews"));
        let out = config.join(OUT_DIR);
        let path = lib.join("2026/07/EOSR/A.CR3");
        note(&out, "1-1", path.to_str().unwrap(), "lr");
        note(&out, "1-2", path.to_str().unwrap(), "lr-zwart-wit");
        // without the NAS: they wait
        assert_eq!(take_in(&config, &lib, None, &local), TakenIn { waiting: 2, ..Default::default() });
        let r = take_in(&config, &lib, Some(&nas), &local);
        assert_eq!((r.taken, r.problems.len()), (2, 0), "{:?}", r.problems);
        for p in ["2026/07/EOSR/A.CR3.lr.jpg", "2026/07/EOSR/A.CR3.lr.full.jpg", "2026/07/EOSR/A.CR3.lr-zwart-wit.full.jpg"] {
            assert!(nas.join(p).is_file(), "{p}");
        }
        assert!(local.join("2026/07/EOSR/A.CR3.lr.jpg").is_file() && !local.join("2026/07/EOSR/A.CR3.lr.full.jpg").exists());
        let t = fs::metadata(nas.join("2026/07/EOSR/A.CR3.lr.jpg")).unwrap().modified().unwrap();
        assert_eq!(chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339(), "2026-11-05T09:00:00+00:00");
        assert_eq!(fs::read_dir(&out).unwrap().count(), 0, "the folder is empty after");
        // the copy's edit gone
        fs::write(out.join("2-r1.json"), serde_json::json!({"version": 1, "path": path, "variant": "lr-zwart-wit", "removed": true}).to_string()).unwrap();
        assert_eq!(take_in(&config, &lib, Some(&nas), &local).removed, 1);
        assert!(!nas.join("2026/07/EOSR/A.CR3.lr-zwart-wit.jpg").exists() && nas.join("2026/07/EOSR/A.CR3.lr.jpg").exists());
    }

    #[test]
    fn what_isnt_taken() {
        let d = tempfile::tempdir().unwrap();
        let (config, lib, nas, local) = (d.path().join("config"), d.path().join("LibraryLocal"), d.path().join("nas"), d.path().join("pv"));
        let out = config.join(OUT_DIR);
        note(&out, "1", "/elders/2026/07/A.CR3", "lr");
        note(&out, "2", lib.join("2026/07/B.CR3").to_str().unwrap(), "LR");
        note(&out, "3", lib.join("2026/07/C.CR3").to_str().unwrap(), "lr");
        fs::write(out.join("3.full.jpg"), b"not a jpeg").unwrap();
        fs::write(out.join("4.json"), "{").unwrap();
        let r = take_in(&config, &lib, Some(&nas), &local);
        assert_eq!((r.taken, r.problems.len()), (0, 4), "{:?}", r.problems);
        assert!(!nas.exists(), "nothing written");
        assert_eq!(fs::read_dir(&out).unwrap().count(), 0, "reported once, then gone");
    }
}
