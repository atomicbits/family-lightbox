//! The outbox (F020): changes to lists and marks kept on this Mac while they can't be written to
//! the `Lists` share, in the share's own format (data contract 5.2, 5.5), and sent there once it
//! is back. Its lines are read as if they stood at the end of this device's own files.

use crate::lists::{self, complete_lines, Op, EVENTS_DIR};
use crate::marks::MARKS_DIR;
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const OUTBOX_DIR: &str = "outbox";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Lists,
    Marks,
}

impl Kind {
    pub const ALL: [Kind; 2] = [Kind::Lists, Kind::Marks];

    fn file(self) -> &'static str {
        match self {
            Kind::Lists => "events.jsonl",
            Kind::Marks => "marks.jsonl",
        }
    }

    /// The folder on the share its lines go to.
    pub fn share_dir(self) -> &'static str {
        match self {
            Kind::Lists => EVENTS_DIR,
            Kind::Marks => MARKS_DIR,
        }
    }
}

pub fn path(config: &Path, kind: Kind) -> PathBuf {
    config.join(OUTBOX_DIR).join(kind.file())
}

/// The outbox's complete lines, each with its `\n`; a half-written last line is left out.
pub fn text(config: &Path, kind: Kind) -> String {
    let raw = fs::read_to_string(path(config, kind)).unwrap_or_default();
    complete_lines(&raw).0.iter().map(|(_, l)| format!("{l}\n")).collect()
}

/// How many changes' lines wait, of both kinds.
pub fn count(config: &Path) -> usize {
    Kind::ALL.iter().map(|k| complete_lines(&text(config, *k)).0.len()).sum()
}

/// Keeps a change's lines: one `write`, then `fsync` (as on the share, 5.2).
pub fn add(config: &Path, kind: Kind, lines: &str) -> Result<(), String> {
    let p = path(config, kind);
    if let Some(d) = p.parent() {
        fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    let mut f = fs::OpenOptions::new().create(true).append(true).open(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    f.write_all(lines.as_bytes()).map_err(|e| format!("wijziging niet bewaard: {e}"))?;
    f.sync_all().map_err(|e| format!("wijziging niet bewaard: {e}"))
}

/// Replaces the outbox file with `content`, through a temporary name.
fn replace(config: &Path, kind: Kind, content: &str) -> Result<(), String> {
    let p = path(config, kind);
    let tmp = p.with_extension("jsonl.tmp");
    lists::write_synced(&tmp, content.as_bytes()).map_err(|e| format!("{}: {e}", tmp.display()))?;
    fs::rename(&tmp, &p).map_err(|e| format!("{}: {e}", p.display()))
}

/// The ids of lists the outbox creates whose id another device created on the share meanwhile,
/// each with the next id free on the share and in the outbox (`2026/vakantie-2`).
fn taken_meanwhile(share: &Path, device: &str, outbox: &str) -> Result<Vec<(String, String)>, String> {
    let (mine, _) = lists::parse_file(outbox, device);
    let created: BTreeSet<String> = mine.iter().filter(|e| matches!(e.op, Op::Create { .. })).map(|e| e.list.clone()).collect();
    if created.is_empty() {
        return Ok(vec![]);
    }
    let theirs = lists::read(share)?.events;
    let mut used: HashSet<String> = theirs.iter().map(|e| e.list.clone()).chain(mine.iter().map(|e| e.list.clone())).collect();
    let mut out = Vec::new();
    for id in created {
        if theirs.iter().any(|e| e.list == id && e.dev != device && matches!(e.op, Op::Create { .. })) {
            let next = (2..).map(|n| format!("{id}-{n}")).find(|c| !used.contains(c)).expect("a free id");
            used.insert(next.clone());
            out.push((id, next));
        }
    }
    Ok(out)
}

/// The outbox's lines with `renames` applied to their `list`; every line stays one line of JSON.
fn renamed(outbox: &str, renames: &[(String, String)]) -> String {
    outbox
        .lines()
        .map(|l| match serde_json::from_str::<serde_json::Value>(l) {
            Ok(mut v) => {
                if let Some(to) = v["list"].as_str().and_then(|id| renames.iter().find(|(from, _)| from == id)).map(|(_, to)| to.clone()) {
                    v["list"] = serde_json::Value::String(to);
                }
                format!("{v}\n")
            }
            Err(_) => format!("{l}\n"),
        })
        .collect()
}

/// Sends what waits to this device's files on the share (F020/S02): a list id taken meanwhile is
/// renamed in the outbox first; lines the share has already aren't written again; the outbox is
/// emptied only once the share's file has every line. Returns how many lines were sent.
pub fn send(config: &Path, share: &Path, device: &str) -> Result<usize, String> {
    let mut sent = 0;
    for kind in Kind::ALL {
        let mut waiting = text(config, kind);
        if waiting.is_empty() {
            continue;
        }
        if kind == Kind::Lists {
            let renames = taken_meanwhile(share, device, &waiting)?;
            if !renames.is_empty() {
                waiting = renamed(&waiting, &renames);
                // saved before sending: a send that stops sends these same lines again
                replace(config, kind, &waiting)?;
            }
        }
        let file = share.join(kind.share_dir()).join(format!("{device}.jsonl"));
        let there = |path: &Path| -> HashSet<String> {
            let t = fs::read_to_string(path).unwrap_or_default();
            complete_lines(&t).0.iter().map(|(_, l)| l.to_string()).collect()
        };
        let before = there(&file);
        let todo: String = waiting.lines().filter(|l| !before.contains(*l)).map(|l| format!("{l}\n")).collect();
        if !todo.is_empty() {
            lists::append_lines(share, kind.share_dir(), device, &todo)?;
        }
        let after = there(&file);
        if let Some(missing) = waiting.lines().find(|l| !after.contains(*l)) {
            return Err(format!("niet op de NAS na het verzenden: {missing}"));
        }
        replace(config, kind, "")?;
        sent += todo.lines().count();
    }
    Ok(sent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lists::DEVICES_FILE;

    fn share(devices: &[&str]) -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        let list: Vec<_> = devices.iter().map(|n| serde_json::json!({"name": n, "owner": "x"})).collect();
        fs::write(d.path().join(DEVICES_FILE), serde_json::json!({"devices": list}).to_string()).unwrap();
        d
    }

    fn create(ts: &str, dev: &str, list: &str, name: &str) -> String {
        format!("{}\n", serde_json::json!({"v": 1, "ts": ts, "dev": dev, "who": "x", "op": "create", "list": list, "name": name}))
    }

    fn add_photo(ts: &str, dev: &str, list: &str, photo: &str) -> String {
        format!("{}\n", serde_json::json!({"v": 1, "ts": ts, "dev": dev, "who": "x", "op": "add", "list": list, "photo": photo}))
    }

    #[test]
    fn kept_counted_and_read_after_the_devices_own_file() {
        let cfg = tempfile::tempdir().unwrap();
        let s = share(&["mbp-peter"]);
        fs::create_dir(s.path().join(EVENTS_DIR)).unwrap();
        fs::write(s.path().join(EVENTS_DIR).join("mbp-peter.jsonl"), create("2026-11-01T10:00:00+01:00", "mbp-peter", "2026/a", "A")).unwrap();
        add(cfg.path(), Kind::Lists, &add_photo("2026-11-04T10:00:00+01:00", "mbp-peter", "2026/a", "sha256:p")).unwrap();
        add(cfg.path(), Kind::Marks, "{\"half\":").unwrap(); // a half-written line doesn't count
        assert_eq!(count(cfg.path()), 1);
        let read = lists::read_with(s.path(), Some(("mbp-peter", &text(cfg.path(), Kind::Lists)))).unwrap();
        let merged = lists::merge(read.events);
        assert!(merged["2026/a"].photos.contains_key("sha256:p"));
    }

    #[test]
    fn sent_once_and_emptied_also_after_a_send_that_stopped() {
        let cfg = tempfile::tempdir().unwrap();
        let s = share(&["mbp-peter"]);
        let one = create("2026-11-04T10:00:00+01:00", "mbp-peter", "2026/a", "A");
        let two = add_photo("2026-11-04T10:01:00+01:00", "mbp-peter", "2026/a", "sha256:p");
        add(cfg.path(), Kind::Lists, &one).unwrap();
        add(cfg.path(), Kind::Lists, &two).unwrap();
        // as if a send wrote the first line and stopped before emptying the outbox
        lists::append_lines(s.path(), EVENTS_DIR, "mbp-peter", &one).unwrap();
        assert_eq!(send(cfg.path(), s.path(), "mbp-peter").unwrap(), 1);
        let on_share = fs::read_to_string(s.path().join(EVENTS_DIR).join("mbp-peter.jsonl")).unwrap();
        assert_eq!(on_share, format!("{one}{two}"));
        assert_eq!(count(cfg.path()), 0);
        assert_eq!(send(cfg.path(), s.path(), "mbp-peter").unwrap(), 0, "nothing waits any more");
    }

    #[test]
    fn a_list_id_taken_meanwhile_gets_the_next_one() {
        let cfg = tempfile::tempdir().unwrap();
        let s = share(&["mbp-peter", "mbp-lynn"]);
        lists::append_lines(s.path(), EVENTS_DIR, "mbp-lynn", &create("2026-11-04T09:00:00+01:00", "mbp-lynn", "2026/vakantie", "Vakantie")).unwrap();
        add(cfg.path(), Kind::Lists, &create("2026-11-04T10:00:00+01:00", "mbp-peter", "2026/vakantie", "Vakantie")).unwrap();
        add(cfg.path(), Kind::Lists, &add_photo("2026-11-04T10:01:00+01:00", "mbp-peter", "2026/vakantie", "sha256:p")).unwrap();
        send(cfg.path(), s.path(), "mbp-peter").unwrap();
        let merged = lists::merge(lists::read(s.path()).unwrap().events);
        assert!(merged["2026/vakantie"].photos.is_empty(), "Lynn's list is left alone");
        assert!(merged["2026/vakantie-2"].photos.contains_key("sha256:p"));
        assert_eq!(merged["2026/vakantie-2"].name, "Vakantie");
    }

    #[test]
    fn nothing_is_sent_for_a_device_the_share_doesnt_list() {
        let cfg = tempfile::tempdir().unwrap();
        let s = share(&["mbp-lynn"]);
        add(cfg.path(), Kind::Lists, &create("2026-11-04T10:00:00+01:00", "mbp-peter", "2026/a", "A")).unwrap();
        assert!(send(cfg.path(), s.path(), "mbp-peter").is_err());
        assert_eq!(count(cfg.path()), 1, "it keeps waiting");
    }
}
