//! The app's own settings (`familylightbox.json`) and the device identity (`device.json`,
//! data contract section 7), both in the config folder next to the index.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const SETTINGS_FILE: &str = "familylightbox.json";
pub const DEVICE_FILE: &str = "device.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Shares {
    pub library: String,
    pub previews: String,
    pub lists: String,
    pub exports: String,
}

impl Default for Shares {
    fn default() -> Self {
        Shares {
            library: "Library".into(),
            previews: "Previews".into(),
            lists: "Lists".into(),
            exports: "Exports".into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Empty until the user names a NAS or one is found (F006/S06): the app knows no family's NAS.
    pub nas_host: String,
    pub shares: Shares,
    /// Where this Mac keeps its local copies (F006/S07); empty until chosen or adopted.
    pub local: Local,
    /// Where new photos come from on this Mac (F018/S03).
    pub import: Import,
}

/// The import's sources on this Mac (F018/S03).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Import {
    /// Folders whose subfolders are sources (the offload folder).
    pub drop_folders: Vec<String>,
    /// Folders another app keeps in sync, never written to.
    pub synced: Vec<Synced>,
    /// Per subfolder of a drop folder, by its folder name.
    pub sources: std::collections::BTreeMap<String, SourceSettings>,
}

/// A synced folder and the sources its photos and its videos go to.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Synced {
    pub path: String,
    pub photos: String,
    pub videos: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SourceSettings {
    /// Another name in the library than the folder's; empty: the folder's name.
    pub name: String,
    /// Names left out (`*-mask.png`).
    pub exclude: Vec<String>,
    pub ignore: bool,
    /// After an import verified on the NAS, delete from `_imported/` (off by default).
    pub delete_imported: bool,
}

/// The local working copy of `Library`, the copy of `Previews` and the read copy of `Lists`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Local {
    pub library: String,
    pub previews: String,
    pub lists: String,
}

impl Local {
    /// The three places inside one chosen folder.
    pub fn inside(folder: &Path) -> Local {
        let p = |n: &str| folder.join(n).display().to_string();
        Local { library: p("Library"), previews: p("Previews"), lists: p("Lists") }
    }

    pub fn is_set(&self) -> bool {
        !self.library.is_empty() && !self.previews.is_empty() && !self.lists.is_empty()
    }
}

/// The folder proposed on a new Mac (F006/S07).
pub fn proposed_folder(home: &Path) -> PathBuf {
    home.join("Pictures").join("FamilyLightbox")
}

/// The CLI's places, which an existing installation keeps (data contract 7).
pub fn cli_local(home: &Path) -> Local {
    let p = |n: &str| home.join("Pictures").join(n).display().to_string();
    Local { library: p("LibraryLocal"), previews: p("Previews"), lists: p("Lists") }
}

/// Without folders in the settings and with the CLI's working copy present, the CLI's places are
/// written to the settings once, as they are, so nothing moves (F006/S07).
pub fn adopt_local(dir: &Path, home: &Path) -> Option<Local> {
    let (mut s, err) = load_settings(dir);
    let cli = cli_local(home);
    if err.is_some() || s.local.is_set() || !Path::new(&cli.library).is_dir() {
        return None;
    }
    s.local = cli.clone();
    write_settings(dir, &s).ok()?;
    Some(cli)
}

/// Checks that `folder` can hold the local copies (it is created if needed and a test file can be
/// written), then keeps it in the settings.
pub fn choose_local(dir: &Path, folder: &Path) -> Result<Local, String> {
    if !folder.is_absolute() {
        return Err(format!("{}: geen volledig pad", folder.display()));
    }
    let err = |e: std::io::Error| format!("{}: {e}", folder.display());
    fs::create_dir_all(folder).map_err(err)?;
    let test = folder.join(format!(".familylightbox-{}.tmp", std::process::id()));
    fs::write(&test, b"x").map_err(err)?;
    let _ = fs::remove_file(&test);
    let (mut s, problem) = load_settings(dir);
    if let Some(p) = problem {
        return Err(p);
    }
    s.local = Local::inside(folder);
    write_settings(dir, &s)?;
    Ok(s.local)
}


/// Settings from `dir`, or the defaults. A broken file is reported, never replaced.
pub fn load_settings(dir: &Path) -> (Settings, Option<String>) {
    let path = dir.join(SETTINGS_FILE);
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (Settings::default(), None),
        Err(e) => return (Settings::default(), Some(format!("{}: {e}", path.display()))),
    };
    match serde_json::from_str(&text) {
        Ok(s) => (s, None),
        Err(e) => (Settings::default(), Some(format!("{}: {e}", path.display()))),
    }
}

pub fn host_problem(host: &str) -> Option<String> {
    if host.is_empty() {
        return Some("de naam van de NAS is leeg".into());
    }
    if host.contains("://") {
        return Some(format!("geef alleen de naam van de NAS, zonder '{}'", &host[..host.find("://").unwrap() + 3]));
    }
    if host.chars().any(|c| c == '/' || c.is_whitespace()) {
        return Some("de naam van de NAS mag geen '/' of spaties bevatten".into());
    }
    None
}

pub fn share_problem(share: &str) -> Option<String> {
    if share.is_empty() {
        return Some("de naam van een gedeelde map is leeg".into());
    }
    if share.contains('/') {
        return Some(format!("'{share}': een gedeelde map mag geen '/' bevatten"));
    }
    None
}

/// Validates, then writes atomically: a temporary file in the same folder, renamed over the original.
pub fn save_settings(dir: &Path, s: &Settings) -> Result<(), String> {
    if let Some(p) = host_problem(&s.nas_host) {
        return Err(p);
    }
    let sh = &s.shares;
    for name in [&sh.library, &sh.previews, &sh.lists, &sh.exports] {
        if let Some(p) = share_problem(name) {
            return Err(p);
        }
    }
    write_settings(dir, s)
}

/// Writes the settings without checking the NAS name, for what the app adopts or the user picks
/// before a NAS is set up.
fn write_settings(dir: &Path, s: &Settings) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let tmp = dir.join(format!("{SETTINGS_FILE}.tmp"));
    let body = serde_json::to_string_pretty(s).map_err(|e| e.to_string())? + "\n";
    fs::write(&tmp, body).map_err(|e| format!("{}: {e}", tmp.display()))?;
    fs::rename(&tmp, dir.join(SETTINGS_FILE)).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("{}: {e}", dir.join(SETTINGS_FILE).display())
    })
}

/// Without a NAS in the settings: the NAS of a `Lists` share that is mounted and holds
/// `devices.json`, or else the host of the CLI's `nas.library` in `state.json` (data contract 7),
/// written once to the settings so an existing installation keeps its NAS (F006/S06). `None` when
/// a NAS is set already, the settings file is broken, or nothing was found.
pub fn adopt_host(dir: &Path, mounts: &[crate::mounts::RawMount]) -> Option<String> {
    let (mut s, err) = load_settings(dir);
    if err.is_some() || !s.nas_host.is_empty() {
        return None;
    }
    let lists = Shares::default().lists;
    let from_mount = mounts.iter().filter(|m| m.fs_type == "smbfs" || m.fs_type == "afpfs").find_map(|m| {
        let s = crate::mounts::parse_source(&m.from)?;
        (s.share.eq_ignore_ascii_case(&lists) && m.on.join(crate::nas::DEVICES_FILE).is_file()).then(|| crate::mounts::bare_host(&s.server))
    });
    let from_cli = || {
        let state: serde_json::Value = serde_json::from_str(&fs::read_to_string(dir.join("state.json")).ok()?).ok()?;
        let spec = state["nas"]["library"].as_str()?;
        let (host, _) = spec.split_once(':').filter(|_| !spec.starts_with('/'))?;
        Some(host.to_string())
    };
    let host = from_mount.or_else(from_cli).filter(|h| host_problem(h).is_none())?;
    s.nas_host = host.clone();
    write_settings(dir, &s).ok()?;
    Some(host)
}

pub const STATE_FILE: &str = "familylightbox-state.json";

/// The app's own state, kept apart from the settings because saving the settings writes that whole
/// file (F007/S04). Not part of the data contract.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    /// When a full previews pass was first confirmed on this Mac.
    pub previews_confirmed_at: Option<String>,
    /// Making previews was paused by hand (F012/S03); stays paused until resumed.
    pub previews_paused: bool,
    /// The filter bar's filter (F002/S03), as the frontend keeps it; kept after a restart.
    pub filter: Option<serde_json::Value>,
}

/// The state, or the default when it is missing or broken (a broken file is left alone).
pub fn load_state(dir: &Path) -> AppState {
    fs::read_to_string(dir.join(STATE_FILE)).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

pub fn save_state(dir: &Path, state: &AppState) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let tmp = dir.join(format!("{STATE_FILE}.tmp"));
    let body = serde_json::to_string_pretty(state).map_err(|e| e.to_string())? + "\n";
    fs::write(&tmp, body).map_err(|e| format!("{}: {e}", tmp.display()))?;
    fs::rename(&tmp, dir.join(STATE_FILE)).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("{}: {e}", dir.join(STATE_FILE).display())
    })
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Device {
    pub name: String,
    pub owner: String,
}

/// `^[a-z][a-z0-9-]{1,31}$` (data contract 1).
pub fn device_name_problem(name: &str) -> Option<String> {
    let ok_len = (2..=32).contains(&name.len());
    let ok_first = name.chars().next().is_some_and(|c| c.is_ascii_lowercase());
    let ok_rest = name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if ok_len && ok_first && ok_rest {
        None
    } else {
        Some("2 tot 32 tekens: kleine letters, cijfers en '-', beginnend met een letter".into())
    }
}

/// `^[a-z][a-z0-9]{1,31}$`: a short lowercase name such as `lynn`.
pub fn owner_problem(owner: &str) -> Option<String> {
    let ok_len = (2..=32).contains(&owner.len());
    let ok_first = owner.chars().next().is_some_and(|c| c.is_ascii_lowercase());
    let ok_rest = owner.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    if ok_len && ok_first && ok_rest {
        None
    } else {
        Some("2 tot 32 tekens: kleine letters en cijfers, beginnend met een letter".into())
    }
}

/// `Ok(None)` when there is no `device.json` yet. The file is only read, never re-serialised.
pub fn load_device(dir: &Path) -> Result<Option<Device>, String> {
    let path = dir.join(DEVICE_FILE);
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let field = |k: &str| v.get(k).and_then(|x| x.as_str()).map(str::to_owned);
    match (field("name"), field("owner")) {
        (Some(name), Some(owner)) if device_name_problem(&name).is_none() && owner_problem(&owner).is_none() => {
            Ok(Some(Device { name, owner }))
        }
        _ => Err(format!("{}: geen geldige 'name' en 'owner'", path.display())),
    }
}

/// Creates `device.json`; refuses when one exists, so an identity is never replaced.
pub fn create_device(dir: &Path, name: &str, owner: &str) -> Result<Device, String> {
    if let Some(p) = device_name_problem(name) {
        return Err(format!("toestelnaam: {p}"));
    }
    if let Some(p) = owner_problem(owner) {
        return Err(format!("eigenaar: {p}"));
    }
    fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = dir.join(DEVICE_FILE);
    let mut f = fs::OpenOptions::new().write(true).create_new(true).open(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            format!("{} bestaat al; een toestel krijgt maar één keer een naam", path.display())
        } else {
            format!("{}: {e}", path.display())
        }
    })?;
    let device = Device { name: name.into(), owner: owner.into() };
    let body = serde_json::to_string_pretty(&device).map_err(|e| e.to_string())? + "\n";
    f.write_all(body.as_bytes()).and_then(|_| f.sync_all()).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(device)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_host_is_adopted_once_from_a_mount_or_the_cli() {
        use crate::mounts::RawMount;
        let dir = tempfile::tempdir().unwrap();
        let lists = dir.path().join("mnt/Lists");
        fs::create_dir_all(&lists).unwrap();
        let mount = |from: &str| RawMount { from: from.into(), on: lists.clone(), fs_type: "smbfs".into() };
        // a Lists share without devices.json isn't the NAS of this app
        assert_eq!(adopt_host(dir.path(), &[mount("//anna@nas1._smb._tcp.local/Lists")]), None);
        fs::write(lists.join("devices.json"), "{}").unwrap();
        assert_eq!(adopt_host(dir.path(), &[mount("//anna@nas1._smb._tcp.local/Lists")]).as_deref(), Some("nas1"));
        assert_eq!(load_settings(dir.path()).0.nas_host, "nas1");
        // with a NAS set nothing is adopted again
        assert_eq!(adopt_host(dir.path(), &[mount("//anna@other/Lists")]), None);
        assert_eq!(load_settings(dir.path()).0.nas_host, "nas1");

        let cli = tempfile::tempdir().unwrap();
        fs::write(cli.path().join("state.json"), r#"{"nas": {"library": "box:/volume1/Library"}}"#).unwrap();
        assert_eq!(adopt_host(cli.path(), &[]).as_deref(), Some("box"));
        let path_only = tempfile::tempdir().unwrap();
        fs::write(path_only.path().join("state.json"), r#"{"nas": {"library": "/Volumes/Library"}}"#).unwrap();
        assert_eq!(adopt_host(path_only.path(), &[]), None);
        assert!(!path_only.path().join(SETTINGS_FILE).exists());
    }

    #[test]
    fn the_local_folders_are_adopted_or_chosen() {
        let home = tempfile::tempdir().unwrap();
        let dir = home.path().join(".config/media");
        // a new Mac: nothing to adopt, nothing written
        assert_eq!(adopt_local(&dir, home.path()), None);
        assert!(!dir.join(SETTINGS_FILE).exists());
        // the CLI's working copy is there: its three places are kept, next to a host adopted later
        fs::create_dir_all(home.path().join("Pictures/LibraryLocal")).unwrap();
        let cli = adopt_local(&dir, home.path()).unwrap();
        assert!(cli.library.ends_with("Pictures/LibraryLocal") && cli.previews.ends_with("Pictures/Previews"));
        assert_eq!(adopt_local(&dir, home.path()), None);
        fs::write(dir.join("state.json"), r#"{"nas": {"library": "box:/volume1/Library"}}"#).unwrap();
        assert_eq!(adopt_host(&dir, &[]).as_deref(), Some("box"));
        let s = load_settings(&dir).0;
        assert_eq!((s.nas_host.as_str(), s.local), ("box", cli));

        // a chosen folder: the three places inside it, kept with what was there
        let chosen = home.path().join("Disk/Foto's");
        let l = choose_local(&dir, &chosen).unwrap();
        assert_eq!(l, Local::inside(&chosen));
        assert!(chosen.is_dir() && fs::read_dir(&chosen).unwrap().count() == 0);
        assert_eq!(load_settings(&dir).0.nas_host, "box");
        assert!(choose_local(&dir, Path::new("relative")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn a_folder_that_cannot_be_written_is_refused() {
        use std::os::unix::fs::PermissionsExt;
        let home = tempfile::tempdir().unwrap();
        let ro = home.path().join("ro");
        fs::create_dir_all(&ro).unwrap();
        fs::set_permissions(&ro, fs::Permissions::from_mode(0o555)).unwrap();
        let r = choose_local(&home.path().join("cfg"), &ro.join("x"));
        fs::set_permissions(&ro, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(r.is_err());
        assert!(!home.path().join("cfg").join(SETTINGS_FILE).exists());
    }

    #[test]
    fn defaults_when_missing_and_nothing_written() {
        let dir = tempfile::tempdir().unwrap();
        let (s, err) = load_settings(dir.path());
        assert_eq!(s, Settings::default());
        assert_eq!(s.nas_host, "");
        assert_eq!(s.shares.lists, "Lists");
        assert!(err.is_none());
        assert!(!dir.path().join(SETTINGS_FILE).exists());
    }

    #[test]
    fn partial_file_is_filled_with_defaults() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(SETTINGS_FILE), r#"{"shares": {"library": "Fotos"}}"#).unwrap();
        let (s, err) = load_settings(dir.path());
        assert!(err.is_none());
        assert_eq!(s.nas_host, "");
        assert_eq!(s.shares.library, "Fotos");
        assert_eq!(s.shares.previews, "Previews");
    }

    #[test]
    fn broken_file_is_reported_and_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(SETTINGS_FILE);
        fs::write(&path, "{ kapot").unwrap();
        let (s, err) = load_settings(dir.path());
        assert_eq!(s, Settings::default());
        assert!(err.unwrap().contains(SETTINGS_FILE));
        assert_eq!(fs::read_to_string(&path).unwrap(), "{ kapot");
    }

    #[test]
    fn save_round_trips_without_leaving_a_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("nog-niet");
        let s = Settings { nas_host: "raven.local".into(), ..Settings::default() };
        save_settings(&sub, &s).unwrap();
        assert_eq!(load_settings(&sub), (s, None));
        let names: Vec<_> = fs::read_dir(&sub).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(names, vec![std::ffi::OsString::from(SETTINGS_FILE)]);
    }

    #[test]
    fn invalid_settings_are_refused_and_the_file_stays() {
        let dir = tempfile::tempdir().unwrap();
        save_settings(dir.path(), &Settings { nas_host: "raven".into(), ..Settings::default() }).unwrap();
        let before = fs::read_to_string(dir.path().join(SETTINGS_FILE)).unwrap();
        for host in ["", "smb://raven", "raven/Library", "ra ven"] {
            let s = Settings { nas_host: host.into(), ..Settings::default() };
            assert!(save_settings(dir.path(), &s).is_err(), "{host:?}");
        }
        let err = save_settings(dir.path(), &Settings { nas_host: "smb://raven".into(), ..Settings::default() }).unwrap_err();
        assert!(err.contains("smb://"), "{err}");
        let mut s = Settings::default();
        s.shares.lists = "Lists/sub".into();
        assert!(save_settings(dir.path(), &s).is_err());
        s.shares.lists = "".into();
        assert!(save_settings(dir.path(), &s).is_err());
        assert_eq!(fs::read_to_string(dir.path().join(SETTINGS_FILE)).unwrap(), before);
    }

    #[test]
    fn state_file_round_trip_and_broken_file() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load_state(dir.path()), AppState::default());
        let st = AppState { previews_confirmed_at: Some("2026-10-26T23:00:00+01:00".into()), ..Default::default() };
        save_state(dir.path(), &st).unwrap();
        assert_eq!(load_state(dir.path()), st);
        assert!(!dir.path().join(format!("{STATE_FILE}.tmp")).exists());
        fs::write(dir.path().join(STATE_FILE), "{ kapot").unwrap();
        assert_eq!(load_state(dir.path()), AppState::default());
        assert_eq!(fs::read_to_string(dir.path().join(STATE_FILE)).unwrap(), "{ kapot", "left alone");
    }

    #[test]
    fn device_name_and_owner_rules() {
        for good in ["mbp-lynn", "mbp-2", "ab"] {
            assert!(device_name_problem(good).is_none(), "{good}");
        }
        for bad in ["Mbp-lynn", "m", "-x", "mbp_lynn", "2mbp", "mbp lynn", &"a".repeat(33)] {
            assert!(device_name_problem(bad).is_some(), "{bad}");
        }
        assert!(device_name_problem(&"a".repeat(32)).is_none());
        for good in ["lynn", "peter", "karel2"] {
            assert!(owner_problem(good).is_none(), "{good}");
        }
        for bad in ["Lynn", "l", "lynn-r", "", "2lynn"] {
            assert!(owner_problem(bad).is_some(), "{bad}");
        }
    }

    #[test]
    fn existing_device_is_read_and_stays_byte_identical() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(DEVICE_FILE);
        let cli = "{\n  \"name\": \"mbp-peter\",\n  \"owner\": \"peter\",\n  \"extra\": 1\n}";
        fs::write(&path, cli).unwrap();
        let d = load_device(dir.path()).unwrap().unwrap();
        assert_eq!(d, Device { name: "mbp-peter".into(), owner: "peter".into() });
        assert!(create_device(dir.path(), "mbp-lynn", "lynn").unwrap_err().contains("bestaat al"));
        assert_eq!(fs::read_to_string(&path).unwrap(), cli);
    }

    #[test]
    fn missing_device_then_created_once() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load_device(dir.path()).unwrap(), None);
        assert!(create_device(dir.path(), "Mbp-lynn", "lynn").is_err());
        assert!(create_device(dir.path(), "mbp-lynn", "Lynn").is_err());
        assert!(!dir.path().join(DEVICE_FILE).exists());
        let d = create_device(dir.path(), "mbp-lynn", "lynn").unwrap();
        assert_eq!(load_device(dir.path()).unwrap(), Some(d));
        let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(dir.path().join(DEVICE_FILE)).unwrap()).unwrap();
        assert_eq!(v, serde_json::json!({"name": "mbp-lynn", "owner": "lynn"}));
        assert!(create_device(dir.path(), "mbp-other", "other").is_err());
    }

    #[test]
    fn invalid_device_file_is_an_error_not_replaced() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(DEVICE_FILE), r#"{"name": "MBP"}"#).unwrap();
        assert!(load_device(dir.path()).is_err());
        assert!(create_device(dir.path(), "mbp-lynn", "lynn").is_err());
    }
}
