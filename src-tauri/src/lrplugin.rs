//! The Lightroom Classic plug-in carried by the app and installed from its settings (F023/S03).

use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const FILES: &[(&str, &str)] = &[
    ("Info.lua", include_str!("../../lightroom/FamilyLightbox.lrplugin/Info.lua")),
    ("Json.lua", include_str!("../../lightroom/FamilyLightbox.lrplugin/Json.lua")),
    ("UpdateLists.lua", include_str!("../../lightroom/FamilyLightbox.lrplugin/UpdateLists.lua")),
    ("Renditions.lua", include_str!("../../lightroom/FamilyLightbox.lrplugin/Renditions.lua")),
];

pub const FOLDER: &str = "FamilyLightbox.lrplugin";

/// Lightroom's modules folder for plug-ins found without adding them by hand.
pub fn modules(home: &Path) -> PathBuf {
    home.join("Library/Application Support/Adobe/Lightroom/Modules")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Missing,
    Current,
    Different,
}

/// Whether the plug-in in `modules` is this app's.
pub fn status(modules: &Path) -> Status {
    let dir = modules.join(FOLDER);
    if !dir.is_dir() {
        return Status::Missing;
    }
    let same = FILES.iter().all(|(name, text)| fs::read_to_string(dir.join(name)).is_ok_and(|t| t == *text));
    if same { Status::Current } else { Status::Different }
}

/// Puts this app's plug-in in `modules`, replacing what is there under its name, through a
/// temporary folder.
pub fn install(modules: &Path) -> Result<(), String> {
    fs::create_dir_all(modules).map_err(|e| format!("{}: {e}", modules.display()))?;
    let tmp = modules.join(format!("{FOLDER}.tmp-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).map_err(|e| format!("{}: {e}", tmp.display()))?;
    for (name, text) in FILES {
        fs::write(tmp.join(name), text).map_err(|e| format!("{name}: {e}"))?;
    }
    let dir = modules.join(FOLDER);
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    fs::rename(&tmp, &dir).map_err(|e| format!("{}: {e}", dir.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_and_kept_current() {
        let d = tempfile::tempdir().unwrap();
        let m = d.path().join("Modules");
        assert_eq!(status(&m), Status::Missing);
        install(&m).unwrap();
        assert_eq!(status(&m), Status::Current);
        fs::write(m.join(FOLDER).join("UpdateLists.lua"), "-- an older one").unwrap();
        fs::write(m.join(FOLDER).join("extra.lua"), "-- left by an older one").unwrap();
        assert_eq!(status(&m), Status::Different);
        install(&m).unwrap();
        assert_eq!(status(&m), Status::Current);
        assert!(!m.join(FOLDER).join("extra.lua").exists(), "replaced as a whole");
        assert_eq!(fs::read_dir(&m).unwrap().count(), 1, "no temporary folder left");
    }
}
