//! What the app can do with each connected share, and registering this device in
//! `Lists/devices.json` (data contract 5.1; F006/S04). Writes only a short-lived test file in
//! `Lists` and `Lists/devices.json`; never anything in `Library`, `Previews` or `Exports`.

use crate::config::Device;
use serde::Serialize;
use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime};

pub const DEVICES_FILE: &str = "devices.json";
const TEST_PREFIX: &str = ".familylightbox-";

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Access {
    pub mounted: bool,
    pub readable: bool,
    /// Only checked for `Lists`.
    pub writable: Option<bool>,
    pub problem: Option<String>,
}

/// Lists the root of `mount`; with `write_test = Some(device)`, also creates and removes a test
/// file there, after clearing this device's test files older than an hour.
pub fn check_share(mount: &Path, write_test: Option<&str>) -> Access {
    let mut a = Access {
        mounted: true,
        ..Access::default()
    };
    let entries = match fs::read_dir(mount) {
        Ok(it) => it,
        Err(e) => {
            a.problem = Some(format!("niet leesbaar: {e}"));
            return a;
        }
    };
    a.readable = true;
    let Some(device) = write_test else { return a };

    let mine = format!("{TEST_PREFIX}{device}-");
    let hour_ago = SystemTime::now() - Duration::from_secs(3600);
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let stale = e
            .metadata()
            .and_then(|m| m.modified())
            .is_ok_and(|t| t < hour_ago);
        if name.starts_with(&mine) && name.ends_with(".tmp") && stale {
            let _ = fs::remove_file(e.path());
        }
    }

    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let test = mount.join(format!("{mine}{nonce}-{}.tmp", std::process::id()));
    let wrote = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&test)
        .and_then(|mut f| f.write_all(b"x"));
    let _ = fs::remove_file(&test);
    match wrote {
        Ok(()) => a.writable = Some(true),
        Err(e) => {
            a.writable = Some(false);
            a.problem = Some(format!("niet schrijfbaar: {e}"));
        }
    }
    a
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub name: String,
    pub owner: String,
    pub label: String,
    pub since: String,
}

/// Adds `me` to the contents of `devices.json`. `Ok(None)`: already registered, nothing to write.
/// Works on the JSON as it is, so other devices and unknown fields keep their content and order.
pub fn merge_device(existing: Option<&str>, me: &Entry) -> Result<Option<String>, String> {
    let entry = json!({"name": me.name, "owner": me.owner, "label": me.label, "since": me.since});
    let mut doc = match existing {
        None => json!({"contract_version": "1.0", "devices": []}),
        Some(text) => serde_json::from_str::<Value>(text)
            .map_err(|e| format!("devices.json is geen geldige JSON: {e}"))?,
    };
    let devices = doc
        .get_mut("devices")
        .and_then(Value::as_array_mut)
        .ok_or("devices.json heeft geen lijst 'devices'")?;
    if let Some(d) = devices
        .iter()
        .find(|d| d.get("name").and_then(Value::as_str) == Some(me.name.as_str()))
    {
        let owner = d.get("owner").and_then(Value::as_str).unwrap_or("?");
        if owner != me.owner {
            return Err(format!("toestelnaam '{}' is al van {owner}", me.name));
        }
        if d.get("retired").is_some() {
            return Err("dit toestel staat als buiten gebruik in devices.json".into());
        }
        return Ok(None);
    }
    devices.push(entry);
    Ok(Some(
        serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())? + "\n",
    ))
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum Registration {
    Registered,
    AlreadyRegistered,
    Refused(String),
    Failed(String),
    /// `Lists` not writable, or no device yet.
    NotTried(String),
}

fn read_opt(path: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(t) => Ok(Some(t)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

fn registered(text: Option<&str>, me: &Entry) -> bool {
    text.and_then(|t| serde_json::from_str::<Value>(t).ok())
        .and_then(|v| v.get("devices").and_then(Value::as_array).cloned())
        .is_some_and(|ds| {
            ds.iter().any(|d| {
                d.get("name").and_then(Value::as_str) == Some(&me.name)
                    && d.get("owner").and_then(Value::as_str) == Some(&me.owner)
            })
        })
}

/// Read, merge, write a temporary file, check that nobody changed `devices.json` meanwhile, rename,
/// read back. A concurrent change means merging again, up to three times. `before_rename` is a
/// test hook.
pub fn register(lists: &Path, me: &Entry, before_rename: &dyn Fn()) -> Registration {
    let path = lists.join(DEVICES_FILE);
    let tmp = lists.join(format!("{DEVICES_FILE}.tmp-{}", me.name));
    for attempt in 0..3 {
        let base = match read_opt(&path) {
            Ok(t) => t,
            Err(e) => return Registration::Failed(e),
        };
        let text = match merge_device(base.as_deref(), me) {
            Err(e) => return Registration::Refused(e),
            Ok(None) if attempt == 0 => return Registration::AlreadyRegistered,
            Ok(None) => return Registration::Registered,
            Ok(Some(t)) => t,
        };
        if let Err(e) = fs::write(&tmp, &text) {
            return Registration::Failed(format!("{}: {e}", tmp.display()));
        }
        before_rename();
        if read_opt(&path).ok() != Some(base) {
            let _ = fs::remove_file(&tmp);
            continue;
        }
        if let Err(e) = fs::rename(&tmp, &path) {
            let _ = fs::remove_file(&tmp);
            return Registration::Failed(format!("{}: {e}", path.display()));
        }
        if registered(read_opt(&path).ok().flatten().as_deref(), me) {
            return Registration::Registered;
        }
    }
    Registration::Failed(
        "devices.json werd tegelijk door een ander toestel gewijzigd; probeer opnieuw".into(),
    )
}

/// The Mac's local host name without `.local`, or "Mac van <owner>".
pub fn local_label(owner: &str) -> String {
    #[cfg(target_os = "macos")]
    {
        let mut buf = [0 as libc::c_char; 256];
        // SAFETY: gethostname writes at most buf.len() bytes, NUL-terminated on success.
        if unsafe { libc::gethostname(buf.as_mut_ptr(), buf.len()) } == 0 {
            let name = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) }
                .to_string_lossy()
                .into_owned();
            let name = name.strip_suffix(".local").unwrap_or(&name).to_string();
            if !name.is_empty() {
                return name;
            }
        }
    }
    format!("Mac van {owner}")
}

/// Now, ISO 8601 with the local offset: `2026-10-26T10:00:00+01:00`.
pub fn now_local() -> String {
    chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
}

pub fn entry_for(device: &Device) -> Entry {
    Entry {
        name: device.name.clone(),
        owner: device.owner.clone(),
        label: local_label(&device.owner),
        since: now_local(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn me() -> Entry {
        Entry {
            name: "mbp-lynn".into(),
            owner: "lynn".into(),
            label: "Lynns-MacBook-Pro".into(),
            since: "2026-10-26T10:00:00+01:00".into(),
        }
    }

    const PETER: &str = r#"{
  "contract_version": "1.0",
  "devices": [
    {"name": "mbp-peter", "owner": "peter", "label": "MacBook Pro Peter", "since": "2026-10-20T18:00:00+02:00", "x-note": 1}
  ],
  "x-extra": true
}"#;

    #[cfg(unix)]
    #[test]
    fn access_readable_unreadable_writable_readonly() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let ok = dir.path().join("Library");
        fs::create_dir(&ok).unwrap();
        assert_eq!(
            check_share(&ok, None),
            Access {
                mounted: true,
                readable: true,
                writable: None,
                problem: None
            }
        );

        let locked = dir.path().join("locked");
        fs::create_dir(&locked).unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
        let a = check_share(&locked, None);
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(!a.readable && a.problem.unwrap().starts_with("niet leesbaar"));

        let lists = dir.path().join("Lists");
        fs::create_dir(&lists).unwrap();
        assert_eq!(check_share(&lists, Some("mbp-lynn")).writable, Some(true));
        assert_eq!(
            fs::read_dir(&lists).unwrap().count(),
            0,
            "test file removed"
        );

        fs::set_permissions(&lists, fs::Permissions::from_mode(0o555)).unwrap();
        let a = check_share(&lists, Some("mbp-lynn"));
        fs::set_permissions(&lists, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(
            a.readable
                && a.writable == Some(false)
                && a.problem.unwrap().starts_with("niet schrijfbaar")
        );
        assert_eq!(fs::read_dir(&lists).unwrap().count(), 0);
    }

    #[test]
    fn stale_test_files_of_this_device_are_cleared() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join(".familylightbox-mbp-lynn-1-1.tmp");
        let other = dir.path().join(".familylightbox-mbp-peter-1-1.tmp");
        let fresh = dir.path().join(".familylightbox-mbp-lynn-2-2.tmp");
        for p in [&old, &other, &fresh] {
            fs::write(p, b"x").unwrap();
        }
        let two_hours_ago = SystemTime::now() - Duration::from_secs(7200);
        for p in [&old, &other] {
            fs::File::options()
                .write(true)
                .open(p)
                .unwrap()
                .set_modified(two_hours_ago)
                .unwrap();
        }
        check_share(dir.path(), Some("mbp-lynn"));
        assert!(!old.exists() && other.exists() && fresh.exists());
    }

    #[test]
    fn merge_new_file_append_same_and_refusals() {
        let new: Value =
            serde_json::from_str(&merge_device(None, &me()).unwrap().unwrap()).unwrap();
        assert_eq!(new["contract_version"], "1.0");
        assert_eq!(new["devices"][0]["name"], "mbp-lynn");
        assert_eq!(new["devices"][0]["since"], "2026-10-26T10:00:00+01:00");

        let merged = merge_device(Some(PETER), &me()).unwrap().unwrap();
        let v: Value = serde_json::from_str(&merged).unwrap();
        let before: Value = serde_json::from_str(PETER).unwrap();
        assert_eq!(
            v["devices"][0], before["devices"][0],
            "other device unchanged"
        );
        assert_eq!(v["devices"][1]["owner"], "lynn");
        assert_eq!(v["x-extra"], true);
        let keys: Vec<_> = v.as_object().unwrap().keys().cloned().collect();
        assert_eq!(
            keys,
            vec!["contract_version", "devices", "x-extra"],
            "order kept"
        );
        let peter_keys: Vec<_> = v["devices"][0]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        assert_eq!(
            peter_keys,
            vec!["name", "owner", "label", "since", "x-note"]
        );

        assert_eq!(
            merge_device(Some(&merged), &me()).unwrap(),
            None,
            "same name and owner"
        );
        let theft = Entry {
            owner: "karel".into(),
            ..me()
        };
        assert_eq!(
            merge_device(Some(&merged), &theft).unwrap_err(),
            "toestelnaam 'mbp-lynn' is al van lynn"
        );
        let retired = merged.replace(
            r#""owner": "lynn","#,
            r#""owner": "lynn", "retired": "2026-11-01T00:00:00+01:00","#,
        );
        assert!(
            merge_device(Some(&retired), &me())
                .unwrap_err()
                .contains("buiten gebruik")
        );
        assert!(
            merge_device(Some("{ kapot"), &me())
                .unwrap_err()
                .contains("geen geldige JSON")
        );
        assert!(
            merge_device(Some(r#"{"devices": {}}"#), &me())
                .unwrap_err()
                .contains("geen lijst")
        );
    }

    #[test]
    fn register_writes_once_and_leaves_no_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            register(dir.path(), &me(), &|| {}),
            Registration::Registered
        );
        assert_eq!(
            register(dir.path(), &me(), &|| {}),
            Registration::AlreadyRegistered
        );
        let names: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(names, vec![std::ffi::OsString::from(DEVICES_FILE)]);
        let broken = dir.path().join(DEVICES_FILE);
        fs::write(&broken, "{ kapot").unwrap();
        assert!(matches!(
            register(dir.path(), &me(), &|| {}),
            Registration::Refused(_)
        ));
        assert_eq!(fs::read_to_string(&broken).unwrap(), "{ kapot");
    }

    #[test]
    fn concurrent_registration_keeps_both_devices() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(DEVICES_FILE);
        fs::write(&path, PETER).unwrap();
        let fired = std::cell::Cell::new(false);
        let competitor = || {
            if !fired.replace(true) {
                let karel = Entry {
                    name: "mbp-karel".into(),
                    owner: "karel".into(),
                    label: "K".into(),
                    since: "t".into(),
                };
                fs::write(&path, merge_device(Some(PETER), &karel).unwrap().unwrap()).unwrap();
            }
        };
        assert_eq!(
            register(dir.path(), &me(), &competitor),
            Registration::Registered
        );
        let v: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let names: Vec<_> = v["devices"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d["name"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["mbp-peter", "mbp-karel", "mbp-lynn"]);
        assert!(
            !dir.path()
                .join(format!("{DEVICES_FILE}.tmp-mbp-lynn"))
                .exists()
        );
    }

    #[test]
    fn label_and_time_have_the_right_shape() {
        assert!(!local_label("lynn").is_empty());
        let t = now_local();
        assert_eq!(t.len(), 25, "{t}");
        assert!(chrono::DateTime::parse_from_rfc3339(&t).is_ok());
    }
}
