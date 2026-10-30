//! A year subfolder onto the `Library` share (F018/S02), as `media push` does over the SMB mount:
//! the originals and sidecars copied without ever overwriting an original, the sizes read back,
//! the month's manifest merged with the NAS's and written next to the files on both sides.

use crate::manifest::{self, MANIFEST_NAME};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// What happened to one file.
#[derive(Debug, PartialEq)]
pub enum Copied {
    /// Copied now.
    New,
    /// There already, with the same size.
    There,
    /// There with another size: left untouched (data contract 3.3).
    Conflict,
}

/// Copies `from` to `to` through `<name>.<process id>.tmp`, with the time of `from`. An existing
/// `to` is never overwritten unless `replace` (Lightroom's `.xmp` may change); with the same
/// size it counts as there.
pub fn copy_file(from: &Path, to: &Path, replace: bool) -> Result<Copied, String> {
    let err = |e: std::io::Error| format!("{}: {e}", to.display());
    let size = fs::metadata(from).map_err(|e| format!("{}: {e}", from.display()))?.len();
    if let Ok(there) = fs::metadata(to) {
        // like rsync's quick check: a sidecar with the same size and time is unchanged
        let same_time = || fs::metadata(from).and_then(|m| m.modified()).ok() == there.modified().ok();
        if there.len() == size && (!replace || same_time()) {
            return Ok(Copied::There);
        }
        if !replace {
            return Ok(Copied::Conflict);
        }
    }
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir).map_err(err)?;
    }
    let mut name = to.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{}.tmp", std::process::id()));
    let tmp = to.with_file_name(name);
    let result = fs::copy(from, &tmp).map_err(err).and_then(|_| {
        if let Ok(t) = fs::metadata(from).and_then(|m| m.modified()) {
            let _ = fs::File::options().write(true).open(&tmp).and_then(|f| f.set_modified(t));
        }
        if replace {
            fs::rename(&tmp, to).map_err(err).map(|_| Copied::New)
        } else {
            rename_new(&tmp, to)
        }
    });
    let _ = fs::remove_file(&tmp);
    result
}

/// Puts `tmp` at `to` only if no file has that name: a file that another Mac put there meanwhile
/// is never replaced.
fn rename_new(tmp: &Path, to: &Path) -> Result<Copied, String> {
    #[cfg(target_os = "macos")]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        let c = |p: &Path| CString::new(p.as_os_str().as_bytes()).map_err(|e| e.to_string());
        let (from, dest) = (c(tmp)?, c(to)?);
        // SAFETY: two valid NUL-terminated paths
        if unsafe { libc::renamex_np(from.as_ptr(), dest.as_ptr(), libc::RENAME_EXCL) } == 0 {
            return Ok(Copied::New);
        }
        let e = std::io::Error::last_os_error();
        match e.raw_os_error() {
            Some(libc::EEXIST) => return Ok(if same_size(tmp, to) { Copied::There } else { Copied::Conflict }),
            // a file system without an exclusive rename: look, then rename
            Some(libc::ENOTSUP) | Some(libc::EINVAL) => {}
            _ => return Err(format!("{}: {e}", to.display())),
        }
    }
    if fs::metadata(to).is_ok() {
        return Ok(if same_size(tmp, to) { Copied::There } else { Copied::Conflict });
    }
    fs::rename(tmp, to).map(|_| Copied::New).map_err(|e| format!("{}: {e}", to.display()))
}

fn same_size(a: &Path, b: &Path) -> bool {
    matches!((fs::metadata(a), fs::metadata(b)), (Ok(x), Ok(y)) if x.len() == y.len())
}

/// What a push of one year subfolder did.
#[derive(Debug, Default)]
pub struct Pushed {
    pub copied: usize,
    pub there: usize,
    /// Rows verified on the NAS.
    pub ok: usize,
    pub missing: Vec<String>,
    pub conflicts: Vec<String>,
    /// Files that couldn't be copied, with the reason.
    pub failed: Vec<String>,
    /// Rows in the manifest written on the NAS.
    pub rows: usize,
}

/// Pushes the files of `local_manifest` (a year subfolder under `local`) to the share at `share`:
/// copy, read back, verify, merge with the NAS's manifest, write it on the share and under
/// `local`. The share's manifest is written only after every copy has been tried.
pub fn push_folder(local: &Path, share: &Path, local_manifest: &Value) -> Result<Pushed, String> {
    let folder = local_manifest["folder"].as_str().ok_or("manifest without folder")?;
    let mut out = Pushed::default();
    for row in local_manifest["files"].as_array().into_iter().flatten() {
        let path = row["path"].as_str().unwrap_or_default();
        let sidecars = row["sidecars"].as_array().into_iter().flatten().filter_map(Value::as_str);
        for p in std::iter::once(path).chain(sidecars) {
            let replace = p.to_ascii_lowercase().ends_with(".xmp");
            match copy_file(&local.join(p), &share.join(p), replace) {
                Ok(Copied::New) => out.copied += 1,
                Ok(Copied::There) => out.there += 1,
                Ok(Copied::Conflict) => {}
                Err(e) => out.failed.push(e),
            }
        }
    }
    let sizes = manifest::sizes(share, folder);
    let remote = manifest::read(&share.join(folder).join(MANIFEST_NAME));
    let v = manifest::verify(local_manifest, remote.as_ref(), &sizes);
    let merged = manifest::merge(local_manifest, remote.as_ref(), &v.ok, &sizes);
    out.rows = merged["files"].as_array().map_or(0, Vec::len);
    // nothing arrived in a folder the NAS doesn't have: no empty manifest there
    if out.rows == 0 && !share.join(folder).is_dir() {
        out.missing = v.missing;
        return Ok(out);
    }
    manifest::write(&share.join(folder).join(MANIFEST_NAME), &merged)?;
    if local.join(folder).is_dir() {
        manifest::write(&local.join(folder).join(MANIFEST_NAME), &merged)?;
    }
    out.ok = v.ok.len();
    out.missing = v.missing;
    out.conflicts = v.conflicts;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mk(root: &Path, rel: &str, bytes: &[u8]) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, bytes).unwrap();
    }

    fn tmp_files(root: &Path) -> Vec<String> {
        walk_all(root).into_iter().filter(|p| p.ends_with(".tmp")).collect()
    }

    fn walk_all(dir: &Path) -> Vec<String> {
        let mut out = Vec::new();
        for e in fs::read_dir(dir).into_iter().flatten().flatten() {
            if e.path().is_dir() {
                out.extend(walk_all(&e.path()));
            } else {
                out.push(e.path().to_string_lossy().into_owned());
            }
        }
        out
    }

    #[test]
    fn copying_never_overwrites_an_original() {
        let d = tempfile::tempdir().unwrap();
        let (a, b) = (d.path().join("a"), d.path().join("share/x/a"));
        fs::write(&a, b"new").unwrap();
        assert_eq!(copy_file(&a, &b, false).unwrap(), Copied::New);
        assert_eq!(fs::read(&b).unwrap(), b"new");
        assert_eq!(fs::metadata(&a).unwrap().modified().unwrap(), fs::metadata(&b).unwrap().modified().unwrap());
        assert_eq!(copy_file(&a, &b, false).unwrap(), Copied::There);
        fs::write(&a, b"longer").unwrap();
        assert_eq!(copy_file(&a, &b, false).unwrap(), Copied::Conflict);
        assert_eq!(fs::read(&b).unwrap(), b"new");
        // a sidecar may be renewed
        assert_eq!(copy_file(&a, &b, true).unwrap(), Copied::New);
        assert_eq!(fs::read(&b).unwrap(), b"longer");
        // the exclusive rename itself refuses a name that appeared meanwhile
        fs::write(d.path().join("t.tmp"), b"abc").unwrap();
        assert_eq!(rename_new(&d.path().join("t.tmp"), &b).unwrap(), Copied::Conflict);
        assert_eq!(fs::read(&b).unwrap(), b"longer");
        assert!(copy_file(&d.path().join("missing"), &b, false).is_err());
        assert!(tmp_files(&d.path().join("share")).is_empty());
    }

    #[test]
    fn a_folder_onto_the_share() {
        let d = tempfile::tempdir().unwrap();
        let (local, share) = (d.path().join("local"), d.path().join("share"));
        mk(&local, "2026/07/Hero13/2026-07-12/GX010001.MP4", b"video");
        mk(&local, "2026/07/Hero13/2026-07-12/GX010001.THM", b"thm");
        mk(&local, "2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3", b"raw");
        mk(&local, "2026/07/EOSR/photos/2026-07-30/CO7A9123.xmp", b"<xmp v2/>");
        // on the NAS already: a file of another Mac, a different one under a local name, a stale xmp
        mk(&share, "2026/07/iPhone-photos/IMG_0955.JPG", b"theirs");
        mk(&share, "2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3", b"other raw");
        mk(&share, "2026/07/EOSR/photos/2026-07-30/CO7A9123.xmp", b"<xmp v1/>");
        let theirs = json!({"contract_version": "1.3", "folder": "2026/07", "files": [
            {"path": "2026/07/iPhone-photos/IMG_0955.JPG", "size": 6, "hash": "sha256:t", "sidecars": []}]});
        manifest::write(&share.join("2026/07/.manifest.json"), &theirs).unwrap();

        let m = manifest::build(&local, "2026/07", "mbp-test", None).unwrap();
        let out = push_folder(&local, &share, &m).unwrap();
        assert_eq!((out.copied, out.there, out.ok), (3, 0, 1)); // video, thm, xmp; the raw conflicts
        assert_eq!(out.conflicts, ["2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3"]);
        assert!(out.missing.is_empty() && out.failed.is_empty());
        assert_eq!(fs::read(share.join("2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3")).unwrap(), b"other raw");
        assert_eq!(fs::read(share.join("2026/07/EOSR/photos/2026-07-30/CO7A9123.xmp")).unwrap(), b"<xmp v2/>");

        let on_nas = manifest::read(&share.join("2026/07/.manifest.json")).unwrap();
        let paths: Vec<&str> = on_nas["files"].as_array().unwrap().iter().map(|r| r["path"].as_str().unwrap()).collect();
        assert_eq!(paths, ["2026/07/Hero13/2026-07-12/GX010001.MP4", "2026/07/iPhone-photos/IMG_0955.JPG"]);
        assert_eq!(on_nas["generated_by"], "mbp-test");
        assert_eq!(manifest::read(&local.join("2026/07/.manifest.json")), Some(on_nas));
        assert_eq!(out.rows, 2);
        assert!(tmp_files(&share).is_empty());

        // a second push copies nothing and finds everything there
        let again = push_folder(&local, &share, &m).unwrap();
        assert_eq!((again.copied, again.there, again.ok), (0, 3, 1));
        // a changed xmp goes again
        mk(&local, "2026/07/EOSR/photos/2026-07-30/CO7A9123.xmp", b"<xmp v3/>");
        assert_eq!(push_folder(&local, &share, &m).unwrap().copied, 1);
    }

    #[test]
    fn a_file_that_cannot_be_read_fails_and_is_missing() {
        let d = tempfile::tempdir().unwrap();
        let (local, share) = (d.path().join("local"), d.path().join("share"));
        mk(&local, "2026/07/x/2026-07-01/a.jpg", b"a");
        let m = manifest::build(&local, "2026/07", "mbp-test", None).unwrap();
        fs::remove_file(local.join("2026/07/x/2026-07-01/a.jpg")).unwrap();
        let out = push_folder(&local, &share, &m).unwrap();
        assert_eq!((out.failed.len(), out.missing.len(), out.rows), (1, 1, 0));
    }
}
