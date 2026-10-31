//! Which NAS shares are mounted on this Mac, found by server and share name from the system's
//! mount list, never by folder name (F006/S02).

use crate::config::Settings;
use percent_encoding::percent_decode_str;
use serde::Serialize;
use std::path::PathBuf;

/// One entry of the system mount list.
#[derive(Debug, Clone)]
pub struct RawMount {
    pub from: String,
    pub on: PathBuf,
    pub fs_type: String,
}

/// The current mounts. `MNT_NOWAIT` returns cached information, so a NAS that went away
/// can't block the call.
#[cfg(target_os = "macos")]
pub fn system_mounts() -> Vec<RawMount> {
    use std::ffi::CStr;
    let mut buf: *mut libc::statfs = std::ptr::null_mut();
    // SAFETY: getmntinfo fills `buf` with a pointer to `n` statfs records in memory it owns;
    // we only read them before returning and never free or keep the pointer.
    let n = unsafe { libc::getmntinfo(&mut buf, libc::MNT_NOWAIT) };
    if n <= 0 || buf.is_null() {
        return Vec::new();
    }
    let entries = unsafe { std::slice::from_raw_parts(buf, n as usize) };
    let text = |chars: &[libc::c_char]| unsafe { CStr::from_ptr(chars.as_ptr()) }.to_string_lossy().into_owned();
    entries
        .iter()
        .map(|e| RawMount {
            from: text(&e.f_mntfromname),
            on: PathBuf::from(text(&e.f_mntonname)),
            fs_type: text(&e.f_fstypename),
        })
        .collect()
}

#[cfg(not(target_os = "macos"))]
pub fn system_mounts() -> Vec<RawMount> {
    Vec::new()
}

#[derive(Debug, PartialEq)]
pub struct Source {
    pub user: Option<String>,
    pub server: String,
    pub share: String,
}

/// `//[user@]server/share` → its parts; anything else is `None`.
pub fn parse_source(from: &str) -> Option<Source> {
    let rest = from.strip_prefix("//")?;
    let (authority, share) = rest.split_once('/')?;
    if share.is_empty() || share.contains('/') {
        return None;
    }
    let (user, server) = match authority.rsplit_once('@') {
        Some((u, s)) => (Some(percent_decode_str(u).decode_utf8_lossy().into_owned()), s),
        None => (None, authority),
    };
    if server.is_empty() {
        return None;
    }
    Some(Source {
        user,
        server: server.to_string(),
        share: percent_decode_str(share).decode_utf8_lossy().into_owned(),
    })
}

pub fn bare_host(host: &str) -> String {
    let h = host.to_ascii_lowercase();
    for suffix in ["._smb._tcp.local", "._afpovertcp._tcp.local", ".local"] {
        if let Some(stripped) = h.strip_suffix(suffix) {
            return stripped.to_string();
        }
    }
    h
}

/// Same NAS? Case-insensitive, ignoring `.local` and the Bonjour service suffixes.
pub fn same_server(configured: &str, mounted: &str) -> bool {
    bare_host(configured) == bare_host(mounted)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Mount {
    pub mount_point: String,
    pub fs_type: String,
    pub server: String,
    pub user: Option<String>,
    pub chosen: bool,
}

#[derive(Debug, Default, PartialEq, Serialize)]
pub struct NasMounts {
    /// The NAS from the settings; empty while none is set up (F006/S06).
    pub host: String,
    pub library: Vec<Mount>,
    pub previews: Vec<Mount>,
    pub lists: Vec<Mount>,
    pub exports: Vec<Mount>,
}

impl NasMounts {
    /// The mount point the app uses for a share, if it is mounted.
    pub fn chosen(list: &[Mount]) -> Option<&Mount> {
        list.iter().find(|m| m.chosen)
    }
}

pub fn mounts_of(raw: &[RawMount], host: &str, share: &str) -> Vec<Mount> {
    let mut found: Vec<Mount> = raw
        .iter()
        .filter(|r| r.fs_type == "smbfs" || r.fs_type == "afpfs")
        .filter_map(|r| {
            let s = parse_source(&r.from)?;
            (same_server(host, &s.server) && s.share.eq_ignore_ascii_case(share)).then(|| Mount {
                mount_point: r.on.display().to_string(),
                fs_type: r.fs_type.clone(),
                server: s.server,
                user: s.user,
                chosen: false,
            })
        })
        .collect();
    let pick = found.iter().position(|m| m.fs_type == "smbfs").or(if found.is_empty() { None } else { Some(0) });
    if let Some(i) = pick {
        found[i].chosen = true;
    }
    found
}

/// The four locations of the layout (F019/S01), each with the mounts of its share; a mount's
/// point is the location's root: the share's mount plus the location's path.
pub fn find_shares(raw: &[RawMount], settings: &Settings) -> NasMounts {
    let h = &settings.nas_host;
    let layout = settings.effective_layout();
    let at = |l: &crate::layout::Location| {
        let mut list = mounts_of(raw, h, &l.share);
        if !l.path.is_empty() {
            for m in &mut list {
                m.mount_point = std::path::Path::new(&m.mount_point).join(&l.path).display().to_string();
            }
        }
        list
    };
    NasMounts {
        host: h.clone(),
        library: at(&layout.library),
        previews: at(&layout.previews),
        lists: at(&layout.lists),
        exports: at(&layout.exports),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(from: &str, on: &str, fs_type: &str) -> RawMount {
        RawMount { from: from.into(), on: on.into(), fs_type: fs_type.into() }
    }

    #[test]
    fn parses_smb_and_afp_sources() {
        let p = |s| parse_source(s).unwrap();
        assert_eq!(
            p("//peter@raven._smb._tcp.local/Library"),
            Source { user: Some("peter".into()), server: "raven._smb._tcp.local".into(), share: "Library".into() }
        );
        assert_eq!(p("//peter@raven._afpovertcp._tcp.local/Library").server, "raven._afpovertcp._tcp.local");
        assert_eq!(p("//lynn@raven/Lists").user.as_deref(), Some("lynn"));
        assert_eq!(p("//raven/Previews"), Source { user: None, server: "raven".into(), share: "Previews".into() });
        let w = p("//WORKGROUP;lynn@raven.local/Exports");
        assert_eq!((w.user.as_deref(), w.server.as_str(), w.share.as_str()), (Some("WORKGROUP;lynn"), "raven.local", "Exports"));
        assert_eq!(p("//lynn@raven/My%20Share").share, "My Share");
        for bad in ["//raven", "//raven/", "map auto_home", "", "/dev/disk3s5", "//@/x"] {
            assert_eq!(parse_source(bad), None, "{bad}");
        }
    }

    #[test]
    fn matches_the_server_in_all_its_forms() {
        for m in ["raven", "RAVEN", "raven.local", "raven._smb._tcp.local", "raven._afpovertcp._tcp.local"] {
            assert!(same_server("raven", m), "{m}");
            assert!(same_server("raven.local", m), "{m}");
        }
        for m in ["raven2", "ravenx.local", "192.168.1.10"] {
            assert!(!same_server("raven", m), "{m}");
        }
        assert!(same_server("192.168.1.10", "192.168.1.10"));
    }

    #[test]
    fn finds_shares_keeps_only_smb_and_afp_and_prefers_smb() {
        let list = vec![
            raw("//peter@raven._afpovertcp._tcp.local/Library", "/Volumes/Library", "afpfs"),
            raw("//peter@raven._smb._tcp.local/library", "/Volumes/Library-1", "smbfs"),
            raw("//peter@raven/Lists", "/Volumes/Lists", "nfs"),
            raw("/dev/disk3s5", "/System/Volumes/Data", "apfs"),
            raw("//peter@raven/TimeMachinePeter", "/Volumes/.timemachine/x/TimeMachinePeter", "smbfs"),
            raw("//peter@raven._afpovertcp._tcp.local/Datasets", "/Volumes/Datasets", "afpfs"),
            raw("//lynn@raven2/Previews", "/Volumes/Previews", "smbfs"),
        ];
        let found = find_shares(&list, &Settings { nas_host: "raven".into(), ..Settings::default() });
        assert_eq!(found.library.len(), 2);
        let chosen = NasMounts::chosen(&found.library).unwrap();
        assert_eq!((chosen.mount_point.as_str(), chosen.fs_type.as_str()), ("/Volumes/Library-1", "smbfs"));
        assert_eq!(found.library.iter().filter(|m| m.chosen).count(), 1);
        assert!(found.lists.is_empty(), "nfs is not a share");
        assert!(found.previews.is_empty(), "other server");
        assert!(found.exports.is_empty());
    }

    #[test]
    fn a_single_afp_mount_is_chosen() {
        let list = vec![raw("//peter@raven._afpovertcp._tcp.local/Library", "/Volumes/Library", "afpfs")];
        let found = find_shares(&list, &Settings { nas_host: "raven".into(), ..Settings::default() });
        assert_eq!(NasMounts::chosen(&found.library).unwrap().mount_point, "/Volumes/Library");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn the_real_mount_list_contains_the_root() {
        let all = system_mounts();
        assert!(all.iter().any(|m| m.on == std::path::Path::new("/")), "{all:?}");
    }

    #[test]
    fn the_roots_of_a_layout_in_one_share() {
        let list = vec![raw("//anna@nas1/Fotos", "/Volumes/Fotos", "smbfs"), raw("//anna@nas1/photo", "/Volumes/photo", "smbfs")];
        let mut layout = crate::layout::NasLayout::in_one_share("Fotos", "x");
        layout.exports = crate::layout::Location { share: "photo".into(), path: "FamilyLightbox/Exports".into() };
        let settings = Settings { nas_host: "nas1".into(), layout: Some(layout), ..Settings::default() };
        let found = find_shares(&list, &settings);
        let root = |l: &[Mount]| NasMounts::chosen(l).map(|m| m.mount_point.clone());
        assert_eq!(root(&found.library).as_deref(), Some("/Volumes/Fotos/Library"));
        assert_eq!(root(&found.lists).as_deref(), Some("/Volumes/Fotos/Lists"));
        assert_eq!(root(&found.exports).as_deref(), Some("/Volumes/photo/FamilyLightbox/Exports"));
        // without a layout file the Mac's share names apply, each at its root
        let old = find_shares(&[raw("//anna@nas1/Lists", "/Volumes/Lists", "smbfs")], &Settings { nas_host: "nas1".into(), ..Settings::default() });
        assert_eq!(root(&old.lists).as_deref(), Some("/Volumes/Lists"));
    }
}
