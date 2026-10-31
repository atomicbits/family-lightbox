//! Connecting shares that aren't mounted, over SMB with NetFS and the macOS sign-in dialog
//! (F006/S03). The app never sees or stores a password.

use crate::config::Settings;
use crate::mounts::NasMounts;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ShareKey {
    Library,
    Previews,
    Lists,
    Exports,
}

/// Characters that must be escaped in one path segment of an `smb://` URL.
const SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ').add(b'"').add(b'#').add(b'%').add(b'/').add(b'<').add(b'>').add(b'?')
    .add(b'`').add(b'{').add(b'}').add(b'[').add(b']').add(b'^').add(b'|').add(b'\\');

pub fn share_url(host: &str, share: &str) -> String {
    format!("smb://{host}/{}", utf8_percent_encode(share, SEGMENT))
}

/// The shares of the locations without any mount, each once, in connect order: the lists' first,
/// because writing lists needs it. Four locations in one share are one mount (F019/S01).
pub fn to_connect(found: &NasMounts, settings: &Settings) -> Vec<(ShareKey, String)> {
    let l = settings.effective_layout();
    let mut out: Vec<(ShareKey, String)> = Vec::new();
    for (key, mounts, loc) in [
        (ShareKey::Lists, &found.lists, &l.lists),
        (ShareKey::Library, &found.library, &l.library),
        (ShareKey::Previews, &found.previews, &l.previews),
        (ShareKey::Exports, &found.exports, &l.exports),
    ] {
        if mounts.is_empty() && !out.iter().any(|(_, s)| *s == loc.share) {
            out.push((key, loc.share.clone()));
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "code", rename_all = "snake_case")]
pub enum Outcome {
    Mounted,
    AlreadyMounted,
    Cancelled,
    AuthFailed,
    NoSuchShare,
    Unreachable,
    Failed(i32),
    /// Not attempted because an earlier share was cancelled or the NAS was unreachable.
    Skipped,
}

/// Return codes of NetFS: Darwin errno values, written out so this module also compiles where
/// `libc` has other numbers. `USER_CANCELED` is Carbon's `userCanceledErr` (dialog cancelled).
mod code {
    pub const ENOENT: i32 = 2;
    pub const EEXIST: i32 = 17;
    pub const EINVAL: i32 = 22;
    pub const ENETUNREACH: i32 = 51;
    pub const ETIMEDOUT: i32 = 60;
    pub const EHOSTDOWN: i32 = 64;
    pub const EHOSTUNREACH: i32 = 65;
    pub const EAUTH: i32 = 80;
    pub const ECANCELED: i32 = 89;
    pub const USER_CANCELED: i32 = -128;
}
use code::*;

pub fn outcome(code: i32) -> Outcome {
    match code {
        0 => Outcome::Mounted,
        EEXIST => Outcome::AlreadyMounted,
        ECANCELED | USER_CANCELED => Outcome::Cancelled,
        EAUTH => Outcome::AuthFailed,
        ENOENT => Outcome::NoSuchShare,
        EHOSTUNREACH | ETIMEDOUT | ENETUNREACH | EHOSTDOWN => Outcome::Unreachable,
        c => Outcome::Failed(c),
    }
}

impl Outcome {
    fn stops(&self) -> bool {
        matches!(self, Outcome::Cancelled | Outcome::Unreachable)
    }

    pub fn message(&self, host: &str, share: &str) -> String {
        match self {
            Outcome::Mounted => "verbonden".into(),
            Outcome::AlreadyMounted => "was al verbonden".into(),
            Outcome::Cancelled => "geannuleerd".into(),
            Outcome::AuthFailed => "aanmelden mislukt".into(),
            Outcome::NoSuchShare => format!("gedeelde map '{share}' bestaat niet op {host}"),
            Outcome::Unreachable => format!("{host} is niet bereikbaar"),
            Outcome::Failed(c) => format!("verbinden mislukt (code {c})"),
            Outcome::Skipped => "niet geprobeerd".into(),
        }
    }
}

pub trait Mounter {
    /// Mounts `url`; returns 0 or an error code.
    fn mount(&self, url: &str) -> i32;
}

#[derive(Debug, Serialize)]
pub struct Attempt {
    pub share: ShareKey,
    pub name: String,
    pub outcome: Outcome,
    pub message: String,
}

/// Connects one share after another. A cancel or an unreachable NAS stops the rest, so the user
/// isn't asked again right away; any other error moves on to the next share.
pub fn connect_all(m: &dyn Mounter, host: &str, list: &[(ShareKey, String)]) -> Vec<Attempt> {
    let mut out = Vec::new();
    let mut stopped = false;
    for (key, name) in list {
        let o = if stopped { Outcome::Skipped } else { outcome(m.mount(&share_url(host, name))) };
        stopped |= o.stops();
        out.push(Attempt { share: *key, name: name.clone(), message: o.message(host, name), outcome: o });
    }
    out
}

#[cfg(target_os = "macos")]
pub struct NetFsMounter {
    /// `false` at app start: connect only when the Keychain has the account, never show a dialog.
    pub allow_ui: bool,
}

/// The value for NetFS's `UIOption` (`kNAUIOptionAllowUI` / `kNAUIOptionNoUI`).
pub fn ui_option(allow_ui: bool) -> &'static str {
    if allow_ui { "AllowUI" } else { "NoUI" }
}

#[cfg(target_os = "macos")]
mod netfs {
    use core_foundation::array::CFArrayRef;
    use core_foundation::dictionary::CFMutableDictionaryRef;
    use core_foundation::string::CFStringRef;
    use core_foundation::url::CFURLRef;

    #[link(name = "NetFS", kind = "framework")]
    unsafe extern "C" {
        pub fn NetFSMountURLSync(
            url: CFURLRef,
            mountpath: CFURLRef,
            user: CFStringRef,
            passwd: CFStringRef,
            open_options: CFMutableDictionaryRef,
            mount_options: CFMutableDictionaryRef,
            mountpoints: *mut CFArrayRef,
        ) -> i32;
    }
}

#[cfg(target_os = "macos")]
impl Mounter for NetFsMounter {
    fn mount(&self, url: &str) -> i32 {
        use core_foundation::base::{kCFAllocatorDefault, CFRelease, CFType, TCFType};
        use core_foundation::boolean::CFBoolean;
        use core_foundation::dictionary::CFMutableDictionary;
        use core_foundation::string::CFString;
        use core_foundation::url::{CFURLCreateWithString, CFURL};

        let cf_url = CFString::new(url);
        // SAFETY: CFURLCreateWithString returns a new reference (or null), which CFURL takes over.
        let raw = unsafe { CFURLCreateWithString(kCFAllocatorDefault, cf_url.as_concrete_TypeRef(), std::ptr::null()) };
        if raw.is_null() {
            return EINVAL;
        }
        let url = unsafe { CFURL::wrap_under_create_rule(raw) };

        // Keys and values as in NetFS.h: kNAUIOptionKey, kNetFSSoftMountKey.
        let mut open: CFMutableDictionary<CFString, CFType> = CFMutableDictionary::new();
        open.add(&CFString::new("UIOption"), &CFString::new(ui_option(self.allow_ui)).as_CFType());
        let mut mount: CFMutableDictionary<CFString, CFType> = CFMutableDictionary::new();
        mount.add(&CFString::new("SoftMount"), &CFBoolean::true_value().as_CFType());

        let mut points = std::ptr::null();
        // SAFETY: all references are valid for the call; null mount path, user and password are
        // allowed (macOS picks /Volumes/<share> and asks in its own dialog). The returned array,
        // if any, is ours to release.
        let code = unsafe {
            netfs::NetFSMountURLSync(
                url.as_concrete_TypeRef(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                open.as_concrete_TypeRef(),
                mount.as_concrete_TypeRef(),
                &mut points,
            )
        };
        if !points.is_null() {
            unsafe { CFRelease(points as _) };
        }
        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mounts::Mount;
    use std::cell::RefCell;

    #[test]
    fn ui_option_follows_allow_ui() {
        assert_eq!(ui_option(true), "AllowUI");
        assert_eq!(ui_option(false), "NoUI");
    }

    #[test]
    fn builds_smb_urls() {
        assert_eq!(share_url("raven", "Library"), "smb://raven/Library");
        assert_eq!(share_url("raven.local", "My Share"), "smb://raven.local/My%20Share");
        assert_eq!(share_url("raven", "a/b#c"), "smb://raven/a%2Fb%23c");
    }

    fn mount(fs_type: &str) -> Mount {
        Mount { mount_point: "/Volumes/X".into(), fs_type: fs_type.into(), server: "raven".into(), user: None, chosen: true }
    }

    #[test]
    fn connects_only_what_has_no_mount_lists_first() {
        let s = Settings { nas_host: "raven".into(), ..Settings::default() };
        let keys = |f: &NasMounts| to_connect(f, &s).into_iter().map(|(k, _)| k).collect::<Vec<_>>();
        let none = NasMounts::default();
        assert_eq!(keys(&none), vec![ShareKey::Lists, ShareKey::Library, ShareKey::Previews, ShareKey::Exports]);
        let afp_library = NasMounts { library: vec![mount("afpfs")], ..NasMounts::default() };
        assert_eq!(keys(&afp_library), vec![ShareKey::Lists, ShareKey::Previews, ShareKey::Exports]);
        let all = NasMounts {
            library: vec![mount("afpfs")],
            previews: vec![mount("smbfs")],
            lists: vec![mount("smbfs")],
            exports: vec![mount("smbfs")],
            ..NasMounts::default()
        };
        assert!(keys(&all).is_empty());
        assert_eq!(to_connect(&none, &s)[0].1, "Lists");
    }

    #[test]
    fn maps_return_codes() {
        assert_eq!(outcome(0), Outcome::Mounted);
        assert_eq!(outcome(EEXIST), Outcome::AlreadyMounted);
        assert_eq!(outcome(ECANCELED), Outcome::Cancelled);
        assert_eq!(outcome(-128), Outcome::Cancelled);
        assert_eq!(outcome(EAUTH), Outcome::AuthFailed);
        assert_eq!(outcome(ENOENT), Outcome::NoSuchShare);
        for c in [EHOSTUNREACH, ETIMEDOUT, ENETUNREACH, EHOSTDOWN] {
            assert_eq!(outcome(c), Outcome::Unreachable, "{c}");
        }
        assert_eq!(outcome(5), Outcome::Failed(5));
        assert_eq!(Outcome::NoSuchShare.message("raven", "Lists"), "gedeelde map 'Lists' bestaat niet op raven");
    }

    struct Fake {
        codes: RefCell<Vec<i32>>,
        urls: RefCell<Vec<String>>,
    }

    impl Fake {
        fn new(codes: &[i32]) -> Self {
            Fake { codes: RefCell::new(codes.iter().rev().copied().collect()), urls: RefCell::new(vec![]) }
        }
    }

    impl Mounter for Fake {
        fn mount(&self, url: &str) -> i32 {
            self.urls.borrow_mut().push(url.into());
            self.codes.borrow_mut().pop().unwrap()
        }
    }

    fn all_four() -> Vec<(ShareKey, String)> {
        to_connect(&NasMounts::default(), &Settings::default())
    }

    #[test]
    fn cancel_and_unreachable_stop_the_rest() {
        let f = Fake::new(&[0, -128]);
        let r = connect_all(&f, "raven", &all_four());
        let got: Vec<_> = r.iter().map(|a| a.outcome.clone()).collect();
        assert_eq!(got, vec![Outcome::Mounted, Outcome::Cancelled, Outcome::Skipped, Outcome::Skipped]);
        assert_eq!(*f.urls.borrow(), vec!["smb://raven/Lists", "smb://raven/Library"]);

        let f = Fake::new(&[EHOSTUNREACH]);
        let r = connect_all(&f, "raven", &all_four());
        assert_eq!(r[0].message, "raven is niet bereikbaar");
        assert!(r[1..].iter().all(|a| a.outcome == Outcome::Skipped));
        assert_eq!(f.urls.borrow().len(), 1);
    }

    #[test]
    fn other_errors_move_on_and_already_mounted_counts_as_success() {
        let f = Fake::new(&[5, EEXIST, EAUTH, 0]);
        let r = connect_all(&f, "raven", &all_four());
        let got: Vec<_> = r.iter().map(|a| a.outcome.clone()).collect();
        assert_eq!(got, vec![Outcome::Failed(5), Outcome::AlreadyMounted, Outcome::AuthFailed, Outcome::Mounted]);
        assert_eq!(f.urls.borrow().len(), 4);
    }

    #[test]
    fn one_share_for_four_locations_is_connected_once() {
        let settings = Settings { nas_host: "nas1".into(), layout: Some(crate::layout::NasLayout::in_one_share("Fotos", "x")), ..Settings::default() };
        let todo = to_connect(&NasMounts::default(), &settings);
        assert_eq!(todo.len(), 1);
        assert_eq!(todo[0], (ShareKey::Lists, "Fotos".to_string()));
    }
}
