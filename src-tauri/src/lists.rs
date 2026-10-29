//! The lists (feature 3, data contract 5): every device appends events to its own file in
//! `Lists/events/`, and every reader merges them the same way, so two Macs can change the same
//! list at the same moment without coordination. This module reads and merges (F003/S01).

use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

pub const DEVICES_FILE: &str = "devices.json";
pub const EVENTS_DIR: &str = "events";

/// What an event does (data contract 5.2).
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Create { name: String },
    Add { photo: String, path: Option<String> },
    Remove { photo: String },
    Rename { name: String },
    Archive,
    Unarchive,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub ts: String,
    /// `ts` as nanoseconds since 1970, for sorting across offsets.
    at: i128,
    pub dev: String,
    pub who: String,
    pub list: String,
    pub op: Op,
    /// Line number in the device's file, for equal times.
    line: usize,
}

#[derive(Deserialize)]
struct Raw {
    v: Option<i64>,
    ts: Option<String>,
    dev: Option<String>,
    who: Option<String>,
    op: Option<String>,
    list: Option<String>,
    name: Option<String>,
    photo: Option<String>,
    path: Option<String>,
}

/// A list id is `YYYY/<slug>`.
pub fn valid_id(id: &str) -> bool {
    match id.split_once('/') {
        Some((year, slug)) => {
            year.len() == 4 && year.bytes().all(|b| b.is_ascii_digit()) && !slug.is_empty() && !slug.contains('/')
        }
        None => false,
    }
}

/// One line of `events/<device>.jsonl`, or `None` when it must be ignored.
fn parse(line: &str, device: &str, n: usize) -> Option<Event> {
    let r: Raw = serde_json::from_str(line).ok()?;
    if r.v != Some(1) || r.dev.as_deref() != Some(device) {
        return None;
    }
    let ts = r.ts?;
    let t = DateTime::parse_from_rfc3339(&ts).ok()?;
    let at = t.timestamp() as i128 * 1_000_000_000 + t.timestamp_subsec_nanos() as i128;
    let list = r.list.filter(|l| valid_id(l))?;
    let nonempty = |s: Option<String>| s.filter(|s| !s.trim().is_empty());
    let op = match r.op?.as_str() {
        "create" => Op::Create { name: nonempty(r.name)? },
        "add" => Op::Add { photo: nonempty(r.photo)?, path: r.path },
        "remove" => Op::Remove { photo: nonempty(r.photo)? },
        "rename" => Op::Rename { name: nonempty(r.name)? },
        "archive" => Op::Archive,
        "unarchive" => Op::Unarchive,
        _ => return None,
    };
    Some(Event { ts, at, dev: device.to_string(), who: r.who?, list, op, line: n })
}

/// The events of one device's file. A last line without `\n` is being written and is left out.
pub fn parse_file(text: &str, device: &str) -> (Vec<Event>, usize) {
    let mut events = Vec::new();
    let mut ignored = 0;
    let complete = match text.rfind('\n') {
        Some(i) => &text[..i],
        None => "",
    };
    if complete.len() + 1 < text.len() {
        ignored += 1;
    }
    for (n, line) in complete.split('\n').enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match parse(line, device, n) {
            Some(e) => events.push(e),
            None => ignored += 1,
        }
    }
    (events, ignored)
}

#[derive(Deserialize)]
struct Devices {
    devices: Vec<DeviceEntry>,
}

#[derive(Deserialize)]
struct DeviceEntry {
    name: String,
}

/// The devices that may write: all in `devices.json`, retired ones included (their events count).
pub fn registered(lists: &Path) -> Result<HashSet<String>, String> {
    let path = lists.join(DEVICES_FILE);
    let text = fs::read_to_string(&path).map_err(|e| format!("{} niet leesbaar: {e}", path.display()))?;
    let d: Devices = serde_json::from_str(&text).map_err(|e| format!("{} klopt niet: {e}", path.display()))?;
    Ok(d.devices.into_iter().map(|d| d.name).collect())
}

#[derive(Debug, Default)]
pub struct Read {
    pub events: Vec<Event>,
    /// Lines left out: half written, not valid, or not this device's.
    pub ignored: usize,
}

/// Every event of the registered devices; files of other devices are not read.
pub fn read(lists: &Path) -> Result<Read, String> {
    let devices = registered(lists)?;
    let mut out = Read::default();
    let dir = lists.join(EVENTS_DIR);
    let Ok(entries) = fs::read_dir(&dir) else { return Ok(out) }; // no events yet
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(device) = name.strip_suffix(".jsonl") else { continue };
        if !devices.contains(device) {
            continue;
        }
        let bytes = fs::read(entry.path()).map_err(|e| format!("{name} niet leesbaar: {e}"))?;
        let (events, ignored) = parse_file(&String::from_utf8_lossy(&bytes), device);
        out.events.extend(events);
        out.ignored += ignored;
    }
    Ok(out)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Member {
    pub photo: String,
    /// The path from the `add` event: a hint, may be outdated.
    pub path: Option<String>,
    pub added_by: String,
    pub added_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct List {
    pub id: String,
    pub name: String,
    pub year: i32,
    pub archived: bool,
    pub created_by: String,
    pub created_at: String,
    pub updated_by: String,
    pub updated_at: String,
    /// By hash.
    #[serde(skip)]
    pub photos: BTreeMap<String, Member>,
}

impl List {
    fn new(e: &Event, name: String) -> List {
        List {
            id: e.list.clone(),
            name,
            year: e.list[..4].parse().unwrap_or(0),
            archived: false,
            created_by: e.who.clone(),
            created_at: e.ts.clone(),
            updated_by: e.dev.clone(),
            updated_at: e.ts.clone(),
            photos: BTreeMap::new(),
        }
    }
}

/// The slug of an id, as the name of a list that has no `create`.
fn slug_of(id: &str) -> String {
    id.split_once('/').map_or(id, |(_, s)| s).to_string()
}

/// Data contract 5.3: sorted by time, then device, then line; the same result on every Mac.
pub fn merge(mut events: Vec<Event>) -> BTreeMap<String, List> {
    events.sort_by(|a, b| a.at.cmp(&b.at).then_with(|| a.dev.cmp(&b.dev)).then_with(|| a.line.cmp(&b.line)));
    let mut lists: BTreeMap<String, List> = BTreeMap::new();
    for e in &events {
        let named = match &e.op {
            Op::Create { name } | Op::Rename { name } => name.clone(),
            _ => slug_of(&e.list),
        };
        let list = lists.entry(e.list.clone()).or_insert_with(|| List::new(e, named.clone()));
        list.updated_by = e.dev.clone();
        list.updated_at = e.ts.clone();
        match &e.op {
            Op::Create { .. } | Op::Rename { .. } => list.name = named,
            Op::Add { photo, path } => {
                list.photos.entry(photo.clone()).or_insert_with(|| Member {
                    photo: photo.clone(),
                    path: path.clone(),
                    added_by: e.who.clone(),
                    added_at: e.ts.clone(),
                });
            }
            Op::Remove { photo } => {
                list.photos.remove(photo);
            }
            Op::Archive => list.archived = true,
            Op::Unarchive => list.archived = false,
        }
    }
    lists
}

/// Where the lists were read from.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// The `Lists` share: changes are possible.
    Nas,
    /// The local copy: look, don't change.
    Local,
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ListInfo {
    #[serde(flatten)]
    pub list: List,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ListsState {
    pub source: Source,
    /// Newest year first, then by name.
    pub lists: Vec<ListInfo>,
    pub ignored: usize,
    pub problem: Option<String>,
}

/// The lists from the share when it is connected, else from the local copy.
pub fn load(nas: Option<&Path>, local: &Path) -> (ListsState, BTreeMap<String, List>) {
    let (source, dir) = match nas {
        Some(n) => (Source::Nas, n),
        None if local.join(DEVICES_FILE).is_file() => (Source::Local, local),
        None => {
            let state = ListsState { source: Source::None, lists: vec![], ignored: 0, problem: None };
            return (state, BTreeMap::new());
        }
    };
    match read(dir) {
        Err(e) => (ListsState { source, lists: vec![], ignored: 0, problem: Some(e) }, BTreeMap::new()),
        Ok(r) => {
            let merged = merge(r.events);
            let mut lists: Vec<ListInfo> =
                merged.values().map(|l| ListInfo { list: l.clone(), count: l.photos.len() }).collect();
            lists.sort_by(|a, b| {
                b.list.year.cmp(&a.list.year).then_with(|| a.list.name.to_lowercase().cmp(&b.list.name.to_lowercase()))
            });
            (ListsState { source, lists, ignored: r.ignored, problem: None }, merged)
        }
    }
}

/// What the sync copies from `Lists` (F003/S01): the devices, the event files and the summaries.
pub fn is_lists_file(rel: &str) -> bool {
    let parts: Vec<&str> = rel.split('/').collect();
    match parts.as_slice() {
        [f] => *f == DEVICES_FILE,
        [dir, f] if *dir == EVENTS_DIR => f.ends_with(".jsonl") && f.len() > 6,
        [year, f] => year.len() == 4 && year.bytes().all(|b| b.is_ascii_digit()) && f.ends_with(".json"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The example from data contract 5.2.
    const LYNN: &str = concat!(
        r#"{"v":1,"ts":"2026-10-20T19:42:11+02:00","dev":"mbp-lynn","who":"lynn","op":"create","list":"2026/herfstwandeling","name":"Herfstwandeling"}"#,
        "\n",
        r#"{"v":1,"ts":"2026-10-20T19:42:30+02:00","dev":"mbp-lynn","who":"lynn","op":"add","list":"2026/herfstwandeling","photo":"sha256:9f3a","path":"2026/10/EOSR/photos/2026-10-18/CO7A9410.CR3"}"#,
        "\n",
        r#"{"v":1,"ts":"2026-10-20T19:43:02+02:00","dev":"mbp-lynn","who":"lynn","op":"add","list":"2026/herfstwandeling","photo":"sha256:c71e","path":"2026/10/EOSR/photos/2026-10-18/CO7A9412.CR3"}"#,
        "\n",
    );
    const PETER: &str = concat!(
        r#"{"v":1,"ts":"2026-10-20T21:10:40+02:00","dev":"mbp-peter","who":"peter","op":"remove","list":"2026/herfstwandeling","photo":"sha256:9f3a"}"#,
        "\n",
        r#"{"v":1,"ts":"2026-11-02T09:00:00+01:00","dev":"mbp-peter","who":"peter","op":"rename","list":"2026/herfstwandeling","name":"Herfstwandeling Ardennen"}"#,
        "\n",
        r#"{"v":1,"ts":"2027-01-05T20:00:00+01:00","dev":"mbp-peter","who":"peter","op":"archive","list":"2026/herfstwandeling"}"#,
        "\n",
    );

    fn events(files: &[(&str, &str)]) -> Vec<Event> {
        files.iter().flat_map(|(dev, text)| parse_file(text, dev).0).collect()
    }

    fn line(ts: &str, dev: &str, op: &str, extra: &str) -> String {
        format!(r#"{{"v":1,"ts":"{ts}","dev":"{dev}","who":"x","op":"{op}","list":"2026/l"{extra}}}"#) + "\n"
    }

    #[test]
    fn the_contract_example() {
        let lists = merge(events(&[("mbp-lynn", LYNN), ("mbp-peter", PETER)]));
        let l = &lists["2026/herfstwandeling"];
        assert_eq!((l.name.as_str(), l.year, l.archived), ("Herfstwandeling Ardennen", 2026, true));
        assert_eq!((l.created_by.as_str(), l.created_at.as_str()), ("lynn", "2026-10-20T19:42:11+02:00"));
        assert_eq!((l.updated_by.as_str(), l.updated_at.as_str()), ("mbp-peter", "2027-01-05T20:00:00+01:00"));
        assert_eq!(l.photos.keys().collect::<Vec<_>>(), vec!["sha256:c71e"]);
        let m = &l.photos["sha256:c71e"];
        assert_eq!((m.added_by.as_str(), m.added_at.as_str()), ("lynn", "2026-10-20T19:43:02+02:00"));
        assert_eq!(m.path.as_deref(), Some("2026/10/EOSR/photos/2026-10-18/CO7A9412.CR3"));
    }

    #[test]
    fn an_add_without_create_makes_the_list_with_its_slug_and_duplicates_change_nothing() {
        let text = [
            line("2026-10-20T10:00:00+02:00", "a", "add", r#","photo":"h1""#),
            line("2026-10-20T10:01:00+02:00", "a", "add", r#","photo":"h1","path":"later""#),
            line("2026-10-20T10:02:00+02:00", "a", "remove", r#","photo":"h2""#),
            line("2026-10-20T10:03:00+02:00", "a", "unarchive", ""),
        ]
        .concat();
        let lists = merge(events(&[("a", &text)]));
        let l = &lists["2026/l"];
        assert_eq!((l.name.as_str(), l.archived, l.photos.len()), ("l", false, 1));
        assert_eq!(l.photos["h1"].added_at, "2026-10-20T10:00:00+02:00");
        assert_eq!(l.photos["h1"].path, None);
    }

    #[test]
    fn order_is_by_moment_then_device_then_line() {
        // 10:00+02:00 is 08:00Z, before 09:00+00:00 although its text sorts after
        let a = line("2026-10-20T10:00:00+02:00", "a", "add", r#","photo":"h""#);
        let b = line("2026-10-20T09:00:00+00:00", "b", "remove", r#","photo":"h""#);
        assert!(merge(events(&[("a", &a), ("b", &b)]))["2026/l"].photos.is_empty());
        // equal moment: device "a" before "b", so b's add wins over a's remove
        let a = line("2026-10-20T10:00:00+02:00", "a", "remove", r#","photo":"h""#);
        let b = line("2026-10-20T08:00:00Z", "b", "add", r#","photo":"h""#);
        assert_eq!(merge(events(&[("b", &b), ("a", &a)]))["2026/l"].photos.len(), 1);
        // equal moment and device: the later line wins
        let same = [
            line("2026-10-20T10:00:00+02:00", "a", "add", r#","photo":"h""#),
            line("2026-10-20T10:00:00+02:00", "a", "remove", r#","photo":"h""#),
        ]
        .concat();
        assert!(merge(events(&[("a", &same)]))["2026/l"].photos.is_empty());
    }

    #[test]
    fn bad_lines_are_ignored_and_counted() {
        let good = line("2026-10-20T10:00:00+02:00", "a", "add", r#","photo":"h""#);
        let text = [
            good.as_str(),
            "{not json\n",
            &line("2026-10-20T10:00:00+02:00", "b", "add", r#","photo":"x""#), // another device's dev
            &line("2026-10-20 10:00", "a", "add", r#","photo":"x""#),          // no offset
            &line("2026-10-20T10:00:00+02:00", "a", "delete", ""),              // unknown op
            &line("2026-10-20T10:00:00+02:00", "a", "add", ""),                 // no photo
            &line("2026-10-20T10:00:00+02:00", "a", "create", r#","name":"  ""#), // empty name
            r#"{"v":2,"ts":"2026-10-20T10:00:00+02:00","dev":"a","who":"x","op":"archive","list":"2026/l"}"#,
            "\n",
            r#"{"v":1,"ts":"2026-10-20T10:00:00+02:00","dev":"a","who":"x","op":"archive","list":"l"}"#,
            "\n",
            r#"{"v":1,"ts":"2026-10-20T11:00:00+02:00","dev":"a","who":"x","op":"rem"#, // half written
        ]
        .concat();
        let (ev, ignored) = parse_file(&text, "a");
        assert_eq!((ev.len(), ignored), (1, 9));
        assert_eq!(parse_file("", "a"), (vec![], 0));
    }

    #[test]
    fn only_registered_devices_are_read() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(EVENTS_DIR)).unwrap();
        fs::write(
            dir.path().join(DEVICES_FILE),
            r#"{"contract_version":"1.0","devices":[{"name":"mbp-lynn","owner":"lynn"},{"name":"mbp-peter","owner":"peter","retired":"2027-01-01T00:00:00+01:00"}]}"#,
        )
        .unwrap();
        fs::write(dir.path().join("events/mbp-lynn.jsonl"), LYNN).unwrap();
        fs::write(dir.path().join("events/mbp-peter.jsonl"), PETER).unwrap();
        fs::write(dir.path().join("events/stranger.jsonl"), line("2026-10-20T10:00:00+02:00", "stranger", "add", r#","photo":"h""#)).unwrap();
        fs::write(dir.path().join("events/notes.txt"), "x").unwrap();
        let r = read(dir.path()).unwrap();
        assert_eq!((r.events.len(), r.ignored), (6, 0));

        let (state, _) = load(None, dir.path());
        assert_eq!((state.source, state.lists.len()), (Source::Local, 1));
        assert_eq!((state.lists[0].count, state.lists[0].list.archived), (1, true));
        let (state, _) = load(Some(dir.path()), Path::new("/nowhere"));
        assert_eq!(state.source, Source::Nas);
    }

    #[test]
    fn a_broken_devices_file_gives_a_problem_not_lists() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(DEVICES_FILE), "{").unwrap();
        let (state, _) = load(None, dir.path());
        assert!(state.problem.unwrap().contains("klopt niet"));
        let (state, _) = load(None, &dir.path().join("nothing"));
        assert_eq!(state.source, Source::None);
    }

    #[test]
    fn lists_sort_newest_year_first_then_by_name() {
        let text = [
            r#"{"v":1,"ts":"2026-10-20T10:00:00+02:00","dev":"a","who":"x","op":"create","list":"2025/zee","name":"Zee"}"#, "\n",
            r#"{"v":1,"ts":"2026-10-20T10:00:00+02:00","dev":"a","who":"x","op":"create","list":"2026/ski","name":"ski"}"#, "\n",
            r#"{"v":1,"ts":"2026-10-20T10:00:00+02:00","dev":"a","who":"x","op":"create","list":"2026/b","name":"Bday"}"#, "\n",
        ]
        .concat();
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(EVENTS_DIR)).unwrap();
        fs::write(dir.path().join(DEVICES_FILE), r#"{"devices":[{"name":"a"}]}"#).unwrap();
        fs::write(dir.path().join("events/a.jsonl"), text).unwrap();
        let ids: Vec<_> = load(None, dir.path()).0.lists.into_iter().map(|l| l.list.id).collect();
        assert_eq!(ids, vec!["2026/b", "2026/ski", "2025/zee"]);
    }

    #[test]
    fn the_sync_copies_devices_events_and_summaries_only() {
        for ok in ["devices.json", "events/mbp-lynn.jsonl", "2026/herfstwandeling.json"] {
            assert!(is_lists_file(ok), "{ok}");
        }
        for no in ["devices.json.tmp-mbp-lynn", "events/.jsonl", "events/x.txt", "notes.json", "2026/sub/x.json", "abcd/x.json"] {
            assert!(!is_lists_file(no), "{no}");
        }
    }
}
