//! The app's log and messages (F016/S01): every warning and error goes to a log file on disk, one
//! line each, and to the window as a `message` event for the list behind the ⚠ in the footer.

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub const LOG_FILE: &str = "familylightbox.log";
/// At this size the log is renamed to `.log.1` (the previous one dropped) and started anew.
pub const MAX_BYTES: u64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Info,
    Warn,
    Error,
}

impl Level {
    fn tag(self) -> &'static str {
        match self {
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }
}

/// One message: when, how serious, from which part of the app, and what.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Message {
    pub at: String,
    pub level: Level,
    pub source: String,
    pub text: String,
}

impl Message {
    pub fn new(level: Level, source: &str, text: &str) -> Message {
        Message { at: crate::lists::now(), level, source: source.to_string(), text: text.to_string() }
    }

    /// `2026-10-30T21:15:02+01:00 WARN sync text`, on one line.
    pub fn line(&self) -> String {
        let text = self.text.replace(['\r', '\n'], " | ");
        format!("{} {} {} {}\n", self.at, self.level.tag(), self.source, text)
    }
}

/// Appends the message to the log in `dir`, rotating it first when it is too large. A log that
/// can't be written is no reason to fail anything else.
pub fn write(dir: &Path, m: &Message, max: u64) {
    let path = dir.join(LOG_FILE);
    if fs::metadata(&path).is_ok_and(|meta| meta.len() >= max) {
        let _ = fs::rename(&path, dir.join(format!("{LOG_FILE}.1")));
    }
    let _ = fs::create_dir_all(dir);
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(m.line().as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_per_message_on_one_line() {
        let m = Message { at: "2026-10-30T21:15:02+01:00".into(), level: Level::Warn, source: "sync".into(), text: "a.jpg: timed out\nagain".into() };
        assert_eq!(m.line(), "2026-10-30T21:15:02+01:00 WARN sync a.jpg: timed out | again\n");
    }

    #[test]
    fn appends_and_rotates_at_the_limit() {
        let dir = tempfile::tempdir().unwrap();
        let m = Message::new(Level::Error, "index", "manifest onleesbaar");
        write(dir.path(), &m, 200);
        write(dir.path(), &m, 200);
        let text = fs::read_to_string(dir.path().join(LOG_FILE)).unwrap();
        assert_eq!(text.lines().count(), 2);
        assert!(text.lines().all(|l| l.contains(" ERROR index manifest onleesbaar")));
        // past the limit: the old log becomes .1, a new one starts
        for _ in 0..5 {
            write(dir.path(), &m, 200);
        }
        assert!(dir.path().join(format!("{LOG_FILE}.1")).is_file());
        assert!(fs::metadata(dir.path().join(LOG_FILE)).unwrap().len() < 200);
    }
}
