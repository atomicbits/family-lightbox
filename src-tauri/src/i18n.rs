//! The language of the core's messages (F011/S03): the window says which once it knows
//! (`set_language`), and every message a user can see is written in both languages with `tr!`,
//! Dutch first, so a missing one doesn't compile. Dutch until the window says otherwise, as the
//! tests expect.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

static ENGLISH: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// A test's own language, so tests in parallel threads don't see each other's.
    static HERE: Cell<Option<bool>> = const { Cell::new(None) };
}

/// `en` or `nl`; anything else is Dutch.
pub fn set_language(lang: &str) {
    ENGLISH.store(lang == "en", Ordering::Relaxed);
}

pub fn english() -> bool {
    HERE.with(|h| h.get()).unwrap_or_else(|| ENGLISH.load(Ordering::Relaxed))
}

/// Runs `f` with English messages on this thread only (tests).
pub fn in_english<T>(f: impl FnOnce() -> T) -> T {
    HERE.with(|h| h.set(Some(true)));
    let out = f();
    HERE.with(|h| h.set(None));
    out
}

/// `tr!("nog geen index: {}", "no index yet: {}", path)`: the message in the language in use,
/// as `format!` makes it; inline names (`{path}`) work too.
#[macro_export]
macro_rules! tr {
    ($nl:literal, $en:literal $(, $arg:expr)* $(,)?) => {
        if $crate::i18n::english() { format!($en $(, $arg)*) } else { format!($nl $(, $arg)*) }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn both_languages() {
        let path = "/x";
        assert_eq!(tr!("nog geen index: {path}", "no index yet: {path}"), "nog geen index: /x");
        assert_eq!(super::in_english(|| tr!("{} foto's", "{} photos", 3)), "3 photos");
        assert_eq!(tr!("klaar", "done"), "klaar", "back to Dutch after");
    }
}
