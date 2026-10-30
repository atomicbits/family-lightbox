//! Stars and hearts (feature 13, data contract 5.5): like the lists, every device appends events to
//! its own file, `Lists/marks/<device>.jsonl`, and every reader merges them the same way. Stars are
//! shared (the last rating wins); a heart belongs to a person (`who`); hiding is shared, the last
//! `hide` or `unhide` decides (F014/S01); a photo's stack too, by its last `stack` or `unstack`
//! (F015/S01). This module reads, merges and writes them (F013/S01).

use crate::lists::{complete_lines, moment, registered, DEVICES_FILE};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

pub const MARKS_DIR: &str = "marks";

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// 0 removes the rating.
    Rate(u8),
    Heart,
    Unheart,
    Hide,
    Unhide,
    /// Into this stack (F015/S01).
    Stack(String),
    Unstack,
    /// A stack's color (F015/S05); the event has no photo.
    Color { stack: String, color: String },
}

/// The colors a stack can be given (F015/S05), as the app names them.
pub const COLORS: [&str; 8] = ["blauw", "amber", "groen", "roze", "paars", "turkoois", "oranje", "grijs"];

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub ts: String,
    at: i128,
    pub dev: String,
    pub who: String,
    pub photo: String,
    pub op: Op,
    line: usize,
}

#[derive(Deserialize)]
struct Raw {
    v: Option<i64>,
    ts: Option<String>,
    dev: Option<String>,
    who: Option<String>,
    op: Option<String>,
    photo: Option<String>,
    stars: Option<i64>,
    stack: Option<String>,
    color: Option<String>,
}

fn parse(line: &str, device: &str, n: usize) -> Option<Event> {
    let r: Raw = serde_json::from_str(line).ok()?;
    if r.v != Some(1) || r.dev.as_deref() != Some(device) {
        return None;
    }
    let ts = r.ts?;
    let at = moment(&ts)?;
    let op_name = r.op?;
    if op_name == "color" {
        let stack = r.stack.filter(|s| !s.trim().is_empty())?;
        let color = r.color.filter(|c| COLORS.contains(&c.as_str()))?;
        let who = r.who.filter(|w| !w.is_empty())?;
        return Some(Event { ts, at, dev: device.to_string(), who, photo: String::new(), op: Op::Color { stack, color }, line: n });
    }
    let photo = r.photo.filter(|p| !p.trim().is_empty())?;
    let op = match op_name.as_str() {
        "rate" => Op::Rate(u8::try_from(r.stars?).ok().filter(|s| *s <= 5)?),
        "heart" => Op::Heart,
        "unheart" => Op::Unheart,
        "hide" => Op::Hide,
        "unhide" => Op::Unhide,
        "stack" => Op::Stack(r.stack.filter(|s| !s.trim().is_empty())?),
        "unstack" => Op::Unstack,
        _ => return None,
    };
    Some(Event { ts, at, dev: device.to_string(), who: r.who.filter(|w| !w.is_empty())?, photo, op, line: n })
}

/// The events of one device's marks file; a half-written last line is left out.
pub fn parse_file(text: &str, device: &str) -> (Vec<Event>, usize) {
    let (lines, half) = complete_lines(text);
    let mut ignored = half as usize;
    let mut events = Vec::new();
    for (n, line) in lines {
        match parse(line, device, n) {
            Some(e) => events.push(e),
            None => ignored += 1,
        }
    }
    (events, ignored)
}

/// A photo's rating, with who gave it and when.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Rating {
    pub stars: u8,
    pub by: String,
    pub at: String,
}

/// The merged marks, by hash.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Marks {
    pub ratings: HashMap<String, Rating>,
    /// Per photo, the people whose heart is on it, with when they put it on.
    pub hearts: HashMap<String, BTreeMap<String, String>>,
    /// Hidden photos, with who hid them and when (F014/S01).
    pub hidden: HashMap<String, Hidden>,
    /// Each photo in a stack, with its stack's id; stacks of one photo left out (F015/S01).
    pub stacks: HashMap<String, String>,
    /// The color chosen for a stack, by id (F015/S05).
    pub stack_colors: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Hidden {
    pub by: String,
    pub at: String,
}

/// Data contract 5.5: sorted by moment, then device, then line; the last `rate` of a photo gives
/// its stars, and the last `heart` or `unheart` of a person decides their heart.
pub fn merge(mut events: Vec<Event>) -> Marks {
    events.sort_by(|a, b| a.at.cmp(&b.at).then_with(|| a.dev.cmp(&b.dev)).then_with(|| a.line.cmp(&b.line)));
    let mut m = Marks::default();
    for e in events {
        match e.op {
            Op::Rate(0) => {
                m.ratings.remove(&e.photo);
            }
            Op::Rate(stars) => {
                m.ratings.insert(e.photo, Rating { stars, by: e.who, at: e.ts });
            }
            Op::Heart => {
                m.hearts.entry(e.photo).or_default().insert(e.who, e.ts);
            }
            Op::Unheart => {
                if let Some(h) = m.hearts.get_mut(&e.photo) {
                    h.remove(&e.who);
                    if h.is_empty() {
                        m.hearts.remove(&e.photo);
                    }
                }
            }
            Op::Hide => {
                m.hidden.insert(e.photo, Hidden { by: e.who, at: e.ts });
            }
            Op::Unhide => {
                m.hidden.remove(&e.photo);
            }
            Op::Stack(id) => {
                m.stacks.insert(e.photo, id);
            }
            Op::Unstack => {
                m.stacks.remove(&e.photo);
            }
            Op::Color { stack, color } => {
                m.stack_colors.insert(stack, color);
            }
        }
    }
    // a stack with one photo left is no stack
    let mut size: HashMap<&str, usize> = HashMap::new();
    for id in m.stacks.values() {
        *size.entry(id.as_str()).or_default() += 1;
    }
    let single: std::collections::HashSet<String> =
        size.into_iter().filter(|(_, n)| *n < 2).map(|(id, _)| id.to_string()).collect();
    m.stacks.retain(|_, id| !single.contains(id));
    m
}

/// A new stack's id: the device and the moment, unique without coordination.
pub fn new_stack_id(device: &str) -> String {
    format!("{device}-{}", chrono::Local::now().format("%Y%m%dT%H%M%S%3f"))
}

#[derive(Debug, Default)]
pub struct Read {
    pub events: Vec<Event>,
    pub ignored: usize,
}

/// The marks of the registered devices. No `marks` folder yet means no marks.
pub fn read(lists: &Path) -> Result<Read, String> {
    let devices = registered(lists)?;
    let mut out = Read::default();
    let Ok(entries) = fs::read_dir(lists.join(MARKS_DIR)) else { return Ok(out) };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(device) = name.strip_suffix(".jsonl") else { continue };
        if !devices.contains(device) {
            continue;
        }
        let bytes = fs::read(entry.path()).map_err(|e| format!("{MARKS_DIR}/{name} niet leesbaar: {e}"))?;
        let (events, ignored) = parse_file(&String::from_utf8_lossy(&bytes), device);
        out.events.extend(events);
        out.ignored += ignored;
    }
    Ok(out)
}

/// A change to stars or hearts, for one or more photos.
#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    Rate { photos: Vec<String>, stars: u8 },
    Heart { photos: Vec<String>, on: bool },
    Hide { photos: Vec<String>, hidden: bool },
    Stack { photos: Vec<String>, stack: String },
    Unstack { photos: Vec<String> },
    Color { stack: String, color: String },
}

/// Why a change can't be written, or `None`.
pub fn refusal(change: &Change) -> Option<String> {
    match change {
        Change::Rate { photos, .. }
        | Change::Heart { photos, .. }
        | Change::Hide { photos, .. }
        | Change::Stack { photos, .. }
        | Change::Unstack { photos }
            if photos.is_empty() =>
        {
            Some("geen foto's gekozen".into())
        }
        Change::Rate { stars, .. } if *stars > 5 => Some(format!("{stars} sterren kan niet: 0 tot 5")),
        Change::Stack { stack, .. } | Change::Color { stack, .. } if stack.trim().is_empty() => Some("een reeks zonder naam".into()),
        Change::Color { color, .. } if !COLORS.contains(&color.as_str()) => Some(format!("onbekende kleur: {color}")),
        _ => None,
    }
}

/// The lines of a change, each ending in `\n` (data contract 5.5).
pub fn lines(change: &Change, ts: &str, dev: &str, who: &str) -> Vec<String> {
    use serde_json::json;
    let head = |op: &str, photo: &str| json!({"v": 1, "ts": ts, "dev": dev, "who": who, "op": op, "photo": photo});
    match change {
        Change::Rate { photos, stars } => photos
            .iter()
            .map(|p| {
                let mut v = head("rate", p);
                v["stars"] = json!(stars);
                v
            })
            .map(|v| v.to_string() + "\n")
            .collect(),
        Change::Heart { photos, on } => {
            photos.iter().map(|p| head(if *on { "heart" } else { "unheart" }, p).to_string() + "\n").collect()
        }
        Change::Hide { photos, hidden } => {
            photos.iter().map(|p| head(if *hidden { "hide" } else { "unhide" }, p).to_string() + "\n").collect()
        }
        Change::Stack { photos, stack } => photos
            .iter()
            .map(|p| {
                let mut v = head("stack", p);
                v["stack"] = json!(stack);
                v.to_string() + "\n"
            })
            .collect(),
        Change::Unstack { photos } => photos.iter().map(|p| head("unstack", p).to_string() + "\n").collect(),
        Change::Color { stack, color } => {
            vec![json!({"v": 1, "ts": ts, "dev": dev, "who": who, "op": "color", "stack": stack, "color": color}).to_string() + "\n"]
        }
    }
}

/// Appends the change to this device's marks file on the share, all lines in one `write`.
pub fn append(lists: &Path, device: &str, owner: &str, change: &Change) -> Result<(), String> {
    if let Some(why) = refusal(change) {
        return Err(why);
    }
    let text = lines(change, &crate::lists::now(), device, owner).concat();
    crate::lists::append_lines(lists, MARKS_DIR, device, &text)
}

/// The marks for the UI: where they were read from, and stars and hearts by hash.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MarksState {
    pub source: crate::lists::Source,
    #[serde(flatten)]
    pub marks: Marks,
    pub ignored: usize,
    pub problem: Option<String>,
}

/// The marks from the share when it is connected, else from the local copy.
pub fn load(nas: Option<&Path>, local: &Path) -> MarksState {
    use crate::lists::Source;
    let (source, dir) = match nas {
        Some(n) => (Source::Nas, n),
        None if local.join(DEVICES_FILE).is_file() => (Source::Local, local),
        None => return MarksState { source: Source::None, marks: Marks::default(), ignored: 0, problem: None },
    };
    match read(dir) {
        Ok(r) => MarksState { source, marks: merge(r.events), ignored: r.ignored, problem: None },
        Err(e) => MarksState { source, marks: Marks::default(), ignored: 0, problem: Some(e) },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(ts: &str, dev: &str, who: &str, op: &str, photo: &str, extra: &str) -> String {
        format!(r#"{{"v":1,"ts":"{ts}","dev":"{dev}","who":"{who}","op":"{op}","photo":"{photo}"{extra}}}"#) + "\n"
    }

    fn merged(files: &[(&str, String)]) -> Marks {
        merge(files.iter().flat_map(|(dev, text)| parse_file(text, dev).0).collect())
    }

    #[test]
    fn the_last_rating_wins_across_devices_and_offsets() {
        let lynn = line("2026-10-30T10:00:00+01:00", "mbp-lynn", "lynn", "rate", "h", r#","stars":4"#); // 09:00Z
        let peter = line("2026-10-30T09:30:00+00:00", "mbp-peter", "peter", "rate", "h", r#","stars":2"#);
        let m = merged(&[("mbp-lynn", lynn.clone()), ("mbp-peter", peter)]);
        assert_eq!(m.ratings["h"], Rating { stars: 2, by: "peter".into(), at: "2026-10-30T09:30:00+00:00".into() });
        let removed = line("2026-10-30T11:00:00+01:00", "mbp-lynn", "lynn", "rate", "h", r#","stars":0"#);
        let m = merged(&[("mbp-lynn", lynn + &removed)]);
        assert!(m.ratings.is_empty());
    }

    #[test]
    fn hearts_are_per_person() {
        let lynn = [
            line("2026-10-30T10:00:00+01:00", "mbp-lynn", "lynn", "heart", "h", ""),
            line("2026-10-30T10:01:00+01:00", "mbp-lynn", "lynn", "heart", "g", ""),
            line("2026-10-30T10:02:00+01:00", "mbp-lynn", "lynn", "unheart", "g", ""),
        ]
        .concat();
        let peter = line("2026-10-30T10:05:00+01:00", "mbp-peter", "peter", "heart", "h", "");
        let m = merged(&[("mbp-lynn", lynn), ("mbp-peter", peter)]);
        assert_eq!(m.hearts["h"].keys().collect::<Vec<_>>(), vec!["lynn", "peter"]);
        assert!(!m.hearts.contains_key("g"));
        // an unheart of someone else's heart changes nothing
        let other = line("2026-10-30T10:06:00+01:00", "mbp-peter", "peter", "unheart", "x", "");
        assert!(merged(&[("mbp-peter", other)]).hearts.is_empty());
    }

    #[test]
    fn equal_times_go_by_device_then_line() {
        let a = line("2026-10-30T10:00:00+01:00", "a", "x", "rate", "h", r#","stars":5"#);
        let b = line("2026-10-30T09:00:00Z", "b", "y", "rate", "h", r#","stars":1"#);
        assert_eq!(merged(&[("b", b), ("a", a)]).ratings["h"].stars, 1);
        let same = [
            line("2026-10-30T10:00:00+01:00", "a", "x", "heart", "h", ""),
            line("2026-10-30T10:00:00+01:00", "a", "x", "unheart", "h", ""),
        ]
        .concat();
        assert!(merged(&[("a", same)]).hearts.is_empty());
    }

    #[test]
    fn bad_lines_are_ignored_and_counted() {
        let text = [
            line("2026-10-30T10:00:00+01:00", "a", "x", "rate", "h", r#","stars":3"#),
            line("2026-10-30T10:00:00+01:00", "a", "x", "rate", "h", r#","stars":6"#),
            line("2026-10-30T10:00:00+01:00", "a", "x", "rate", "h", ""),
            line("2026-10-30T10:00:00+01:00", "a", "x", "like", "h", ""),
            line("2026-10-30T10:00:00+01:00", "a", "x", "heart", "", ""),
            line("2026-10-30T10:00:00+01:00", "b", "x", "heart", "h", ""),
            line("yesterday", "a", "x", "heart", "h", ""),
            r#"{"v":1,"ts":"2026-10-30T10:00:00+01:00","dev":"a","who":"x","op":"hea"#.to_string(),
        ]
        .concat();
        let (ev, ignored) = parse_file(&text, "a");
        assert_eq!((ev.len(), ignored), (1, 7));
    }

    fn registered_dir(devices: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let list: Vec<_> = devices.iter().map(|d| serde_json::json!({"name": d, "owner": "x"})).collect();
        fs::write(dir.path().join(DEVICES_FILE), serde_json::json!({"devices": list}).to_string()).unwrap();
        dir
    }

    #[test]
    fn writing_and_reading_back() {
        let dir = registered_dir(&["mbp-lynn"]);
        let rate = Change::Rate { photos: vec!["h1".into(), "h2".into()], stars: 4 };
        append(dir.path(), "mbp-lynn", "lynn", &rate).unwrap();
        append(dir.path(), "mbp-lynn", "lynn", &Change::Heart { photos: vec!["h1".into()], on: true }).unwrap();
        let text = fs::read_to_string(dir.path().join("marks/mbp-lynn.jsonl")).unwrap();
        assert_eq!(text.lines().count(), 3);
        assert!(text.lines().next().unwrap().ends_with(r#""op":"rate","photo":"h1","stars":4}"#));
        let s = load(Some(dir.path()), Path::new("/nowhere"));
        assert_eq!((s.marks.ratings.len(), s.marks.ratings["h2"].stars, s.marks.ratings["h2"].by.as_str()), (2, 4, "lynn"));
        assert!(s.marks.hearts["h1"].contains_key("lynn"));
        // a stranger's file is not read
        fs::write(dir.path().join("marks/stranger.jsonl"), line("2026-10-30T10:00:00+01:00", "stranger", "s", "heart", "h2", "")).unwrap();
        assert!(!load(None, dir.path()).marks.hearts.contains_key("h2"));
    }

    #[test]
    fn refusals() {
        let dir = registered_dir(&["mbp-lynn"]);
        let none = Change::Rate { photos: vec![], stars: 3 };
        assert_eq!(append(dir.path(), "mbp-lynn", "lynn", &none).unwrap_err(), "geen foto's gekozen");
        let six = Change::Rate { photos: vec!["h".into()], stars: 6 };
        assert!(append(dir.path(), "mbp-lynn", "lynn", &six).unwrap_err().contains("0 tot 5"));
        let heart = Change::Heart { photos: vec!["h".into()], on: true };
        assert!(append(dir.path(), "other", "x", &heart).unwrap_err().contains("staat niet in devices.json"));
        assert!(!dir.path().join(MARKS_DIR).exists());
    }

    #[test]
    fn the_watch_sees_a_new_mark() {
        let dir = registered_dir(&["mbp-lynn"]);
        fs::create_dir(dir.path().join("events")).unwrap();
        let before = crate::lists::stamp(dir.path()).unwrap();
        append(dir.path(), "mbp-lynn", "lynn", &Change::Heart { photos: vec!["h".into()], on: true }).unwrap();
        assert_ne!(crate::lists::stamp(dir.path()).unwrap(), before);
        assert!(crate::lists::is_lists_file("marks/mbp-lynn.jsonl"));
    }

    #[test]
    fn hiding_is_shared_and_the_last_one_decides() {
        let lynn = line("2026-10-30T10:00:00+01:00", "mbp-lynn", "lynn", "hide", "h", "")
            + &line("2026-10-30T10:00:00+01:00", "mbp-lynn", "lynn", "hide", "g", "");
        let peter = line("2026-10-30T10:05:00+01:00", "mbp-peter", "peter", "unhide", "g", "");
        let m = merged(&[("mbp-lynn", lynn.clone()), ("mbp-peter", peter)]);
        assert_eq!(m.hidden["h"], Hidden { by: "lynn".into(), at: "2026-10-30T10:00:00+01:00".into() });
        assert!(!m.hidden.contains_key("g"));
        // an earlier unhide doesn't undo a later hide
        let early = line("2026-10-30T09:00:00+01:00", "mbp-peter", "peter", "unhide", "h", "");
        assert!(merged(&[("mbp-lynn", lynn), ("mbp-peter", early)]).hidden.contains_key("h"));
    }

    #[test]
    fn writing_hide_and_unhide() {
        let dir = registered_dir(&["mbp-lynn"]);
        append(dir.path(), "mbp-lynn", "lynn", &Change::Hide { photos: vec!["a".into(), "b".into()], hidden: true }).unwrap();
        append(dir.path(), "mbp-lynn", "lynn", &Change::Hide { photos: vec!["b".into()], hidden: false }).unwrap();
        let s = load(Some(dir.path()), Path::new("/nowhere"));
        assert_eq!(s.marks.hidden.keys().collect::<Vec<_>>(), vec!["a"]);
        assert!(append(dir.path(), "mbp-lynn", "lynn", &Change::Hide { photos: vec![], hidden: true }).is_err());
    }

    #[test]
    fn stacks_per_photo_the_last_event_wins_and_one_photo_is_no_stack() {
        let x = r#","stack":"mbp-peter-1""#;
        let peter = [
            line("2026-10-30T10:00:00+01:00", "mbp-peter", "peter", "stack", "a", x),
            line("2026-10-30T10:00:00+01:00", "mbp-peter", "peter", "stack", "b", x),
            line("2026-10-30T10:00:00+01:00", "mbp-peter", "peter", "stack", "c", x),
            line("2026-10-30T10:00:00+01:00", "mbp-peter", "peter", "stack", "d", r#","stack":"mbp-peter-2""#),
        ]
        .concat();
        let lynn = line("2026-10-30T11:00:00+01:00", "mbp-lynn", "lynn", "unstack", "c", "");
        let m = merged(&[("mbp-peter", peter), ("mbp-lynn", lynn)]);
        let mut got: Vec<_> = m.stacks.iter().map(|(p, s)| (p.as_str(), s.as_str())).collect();
        got.sort();
        // c taken out later by lynn; d alone in its stack, so no stack
        assert_eq!(got, vec![("a", "mbp-peter-1"), ("b", "mbp-peter-1")]);
        let bad = line("2026-10-30T10:00:00+01:00", "a", "x", "stack", "h", "");
        assert_eq!(parse_file(&bad, "a").1, 1, "a stack without an id");
    }

    #[test]
    fn writing_stacks() {
        let dir = registered_dir(&["mbp-lynn"]);
        let id = new_stack_id("mbp-lynn");
        assert!(id.starts_with("mbp-lynn-20"), "{id}");
        append(dir.path(), "mbp-lynn", "lynn", &Change::Stack { photos: vec!["a".into(), "b".into(), "c".into()], stack: id.clone() }).unwrap();
        append(dir.path(), "mbp-lynn", "lynn", &Change::Unstack { photos: vec!["c".into()] }).unwrap();
        let s = load(Some(dir.path()), Path::new("/nowhere"));
        assert_eq!((s.marks.stacks.len(), s.marks.stacks["a"].as_str()), (2, id.as_str()));
        let empty = Change::Stack { photos: vec!["a".into()], stack: " ".into() };
        assert!(append(dir.path(), "mbp-lynn", "lynn", &empty).is_err());
    }

    #[test]
    fn a_stack_color_the_last_one_wins() {
        let dir = registered_dir(&["mbp-lynn"]);
        append(dir.path(), "mbp-lynn", "lynn", &Change::Color { stack: "s".into(), color: "groen".into() }).unwrap();
        append(dir.path(), "mbp-lynn", "lynn", &Change::Color { stack: "s".into(), color: "roze".into() }).unwrap();
        let s = load(Some(dir.path()), Path::new("/nowhere"));
        assert_eq!(s.marks.stack_colors["s"], "roze");
        assert!(append(dir.path(), "mbp-lynn", "lynn", &Change::Color { stack: "s".into(), color: "rood".into() }).unwrap_err().contains("onbekende kleur"));
        let unknown = r#"{"v":1,"ts":"2026-10-30T10:00:00+01:00","dev":"a","who":"x","op":"color","stack":"s","color":"rood"}"#.to_string() + "\n";
        assert_eq!(parse_file(&unknown, "a").1, 1);
    }
}
