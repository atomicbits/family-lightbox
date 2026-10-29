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

/// `ts` as nanoseconds since 1970, so times with different offsets sort right; `None` when it
/// isn't ISO 8601 with an offset.
pub fn moment(ts: &str) -> Option<i128> {
    let t = DateTime::parse_from_rfc3339(ts).ok()?;
    Some(t.timestamp() as i128 * 1_000_000_000 + t.timestamp_subsec_nanos() as i128)
}

/// The complete lines of an event file with their line numbers, and whether a last line without
/// `\n` was left out (it is being written; data contract 5.2).
pub fn complete_lines(text: &str) -> (Vec<(usize, &str)>, bool) {
    let complete = match text.rfind('\n') {
        Some(i) => &text[..i],
        None => "",
    };
    let half = complete.len() + 1 < text.len();
    let lines = complete.split('\n').enumerate().filter(|(_, l)| !l.trim().is_empty()).collect();
    (lines, half)
}

/// One line of `events/<device>.jsonl`, or `None` when it must be ignored.
fn parse(line: &str, device: &str, n: usize) -> Option<Event> {
    let r: Raw = serde_json::from_str(line).ok()?;
    if r.v != Some(1) || r.dev.as_deref() != Some(device) {
        return None;
    }
    let ts = r.ts?;
    let at = moment(&ts)?;
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
    let (lines, half) = complete_lines(text);
    let mut ignored = half as usize;
    for (n, line) in lines {
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

/// A fingerprint of the devices and event files, lists and marks: their names, sizes and times.
/// It changes when any device appends an event, so the app knows when to read again (F003/S04,
/// F013/S01).
pub fn stamp(lists: &Path) -> Option<u64> {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    let mut files: Vec<(String, u64, Option<std::time::SystemTime>)> = Vec::new();
    let mut add = |name: String, path: &Path| {
        if let Ok(m) = fs::metadata(path) {
            files.push((name, m.len(), m.modified().ok()));
        }
    };
    add(DEVICES_FILE.into(), &lists.join(DEVICES_FILE));
    let mut any = false;
    for dir in [EVENTS_DIR, crate::marks::MARKS_DIR] {
        let Ok(entries) = fs::read_dir(lists.join(dir)) else { continue };
        any = true;
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.ends_with(".jsonl") {
                add(format!("{dir}/{name}"), &e.path());
            }
        }
    }
    if !any {
        return None;
    }
    files.sort();
    files.hash(&mut h);
    Some(h.finish())
}

/// What the sync copies from `Lists` (F003/S01): the devices, the event files and the summaries.
pub fn is_lists_file(rel: &str) -> bool {
    let parts: Vec<&str> = rel.split('/').collect();
    match parts.as_slice() {
        [f] => *f == DEVICES_FILE,
        [dir, f] if *dir == EVENTS_DIR || *dir == crate::marks::MARKS_DIR => f.ends_with(".jsonl") && f.len() > 6,
        [year, f] => year.len() == 4 && year.bytes().all(|b| b.is_ascii_digit()) && f.ends_with(".json"),
        _ => false,
    }
}

// ── Writing (F003/S02) ─────────────────────────────────────────────────────────────────────────

/// At most this many characters in a list's name.
pub const NAME_MAX: usize = 100;
const SLUG_MAX: usize = 60;

/// A slug from a name: lowercase, accents removed, everything but letters and digits a `-`.
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars().flat_map(char::to_lowercase) {
        let plain = match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' => "a",
            'æ' => "ae",
            'ç' | 'č' | 'ć' => "c",
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ę' | 'ě' => "e",
            'ì' | 'í' | 'î' | 'ï' | 'ī' => "i",
            'ñ' | 'ń' | 'ň' => "n",
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' => "o",
            'œ' => "oe",
            'ß' => "ss",
            'š' | 'ś' => "s",
            'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' => "u",
            'ý' | 'ÿ' => "y",
            'ž' | 'ź' | 'ż' => "z",
            'ł' => "l",
            c if c.is_ascii_alphanumeric() => {
                out.push(c);
                continue;
            }
            _ => "-",
        };
        out.push_str(plain);
    }
    let mut slug = String::new();
    for part in out.split('-').filter(|p| !p.is_empty()) {
        if !slug.is_empty() {
            slug.push('-');
        }
        slug.push_str(part);
    }
    let mut slug: String = slug.chars().take(SLUG_MAX).collect();
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() { "lijst".to_string() } else { slug }
}

/// `<year>/<slug>`, with `-2`, `-3`, … when that id is taken.
pub fn new_id(year: i32, name: &str, taken: &dyn Fn(&str) -> bool) -> String {
    let base = format!("{year:04}/{}", slug(name));
    (1..)
        .map(|n| if n == 1 { base.clone() } else { format!("{base}-{n}") })
        .find(|id| !taken(id))
        .expect("an unused id")
}

/// A change to a list, as the UI asks for it.
#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    Create { list: String, name: String },
    Add { list: String, photos: Vec<(String, Option<String>)> },
    Remove { list: String, photos: Vec<String> },
    Rename { list: String, name: String },
    Archive { list: String, archived: bool },
}

impl Change {
    fn list(&self) -> &str {
        match self {
            Change::Create { list, .. }
            | Change::Add { list, .. }
            | Change::Remove { list, .. }
            | Change::Rename { list, .. }
            | Change::Archive { list, .. } => list,
        }
    }
}

/// The lines this change appends, each ending in `\n` (data contract 5.2).
pub fn lines(change: &Change, ts: &str, dev: &str, who: &str) -> Vec<String> {
    use serde_json::json;
    let head = |op: &str, list: &str| json!({"v": 1, "ts": ts, "dev": dev, "who": who, "op": op, "list": list});
    let with = |mut v: serde_json::Value, key: &str, val: &str| {
        v[key] = json!(val);
        v
    };
    let values: Vec<serde_json::Value> = match change {
        Change::Create { list, name } => vec![with(head("create", list), "name", name)],
        Change::Rename { list, name } => vec![with(head("rename", list), "name", name)],
        Change::Archive { list, archived } => vec![head(if *archived { "archive" } else { "unarchive" }, list)],
        Change::Add { list, photos } => photos
            .iter()
            .map(|(photo, path)| {
                let v = with(head("add", list), "photo", photo);
                match path {
                    Some(p) => with(v, "path", p),
                    None => v,
                }
            })
            .collect(),
        Change::Remove { list, photos } => photos.iter().map(|p| with(head("remove", list), "photo", p)).collect(),
    };
    values.into_iter().map(|v| v.to_string() + "\n").collect()
}

/// Now, as `2026-10-20T19:42:11+02:00`.
pub fn now() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string()
}

/// Why a change can't be written, or `None`.
pub fn refusal(change: &Change, lists: &BTreeMap<String, List>) -> Option<String> {
    let name_problem = |name: &str| {
        let n = name.trim().chars().count();
        if n == 0 {
            Some("de naam is leeg".to_string())
        } else if n > NAME_MAX {
            Some(format!("de naam is langer dan {NAME_MAX} tekens"))
        } else {
            None
        }
    };
    match change {
        Change::Create { list, name } => {
            name_problem(name).or_else(|| (!valid_id(list)).then(|| format!("ongeldige lijst: {list}")))
        }
        _ if !lists.contains_key(change.list()) => Some(format!("onbekende lijst: {}", change.list())),
        Change::Rename { name, .. } => name_problem(name),
        Change::Add { photos, .. } if photos.is_empty() => Some("geen foto's gekozen".into()),
        Change::Remove { photos, .. } if photos.is_empty() => Some("geen foto's gekozen".into()),
        _ => None,
    }
}

/// Appends the change to this device's own event file on the share: all its lines in one `write`,
/// then `fsync` (data contract 5.2). Refused when this device isn't registered.
pub fn append(lists: &Path, device: &str, owner: &str, change: &Change, merged: &BTreeMap<String, List>) -> Result<(), String> {
    if let Some(why) = refusal(change, merged) {
        return Err(why);
    }
    append_lines(lists, EVENTS_DIR, device, &lines(change, &now(), device, owner).concat())
}

/// Appends text to `<dir>/<device>.jsonl` on the share in one `write`, then `fsync`; refused when
/// this device isn't registered.
pub fn append_lines(lists: &Path, dir: &str, device: &str, text: &str) -> Result<(), String> {
    use std::io::Write;
    if !registered(lists)?.contains(device) {
        return Err(format!("dit toestel ({device}) staat niet in {DEVICES_FILE}"));
    }
    let dir = lists.join(dir);
    fs::create_dir_all(&dir).map_err(|e| format!("{} maken mislukt: {e}", dir.display()))?;
    let path = dir.join(format!("{device}.jsonl"));
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("{} openen mislukt: {e}", path.display()))?;
    f.write_all(text.as_bytes()).map_err(|e| format!("lijst bijwerken mislukt: {e}"))?;
    f.sync_all().map_err(|e| format!("lijst bijwerken mislukt: {e}"))
}

/// The summary of one list (data contract 5.4); `path_of` gives the current path of a hash from
/// the index, where it is known.
pub fn summary(list: &List, path_of: &dyn Fn(&str) -> Option<String>) -> serde_json::Value {
    let mut photos: Vec<(Option<String>, &Member)> =
        list.photos.values().map(|m| (path_of(&m.photo).or_else(|| m.path.clone()), m)).collect();
    photos.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.photo.cmp(&b.1.photo)));
    serde_json::json!({
        "contract_version": "1.0",
        "list": list.id,
        "name": list.name,
        "year": list.year,
        "archived": list.archived,
        "created_by": list.created_by,
        "created_at": list.created_at,
        "updated_at": list.updated_at,
        "updated_by": list.updated_by,
        "count": photos.len(),
        "photos": photos.iter().map(|(path, m)| serde_json::json!({
            "photo": m.photo, "path": path, "added_by": m.added_by, "added_at": m.added_at,
        })).collect::<Vec<_>>(),
    })
}

/// Writes `Lists/<year>/<slug>.json` through a temporary name of this device and a rename.
pub fn write_summary(lists: &Path, device: &str, list: &List, path_of: &dyn Fn(&str) -> Option<String>) -> Result<(), String> {
    let (year, slug) = list.id.split_once('/').ok_or_else(|| format!("ongeldige lijst: {}", list.id))?;
    let dir = lists.join(year);
    fs::create_dir_all(&dir).map_err(|e| format!("{} maken mislukt: {e}", dir.display()))?;
    let target = dir.join(format!("{slug}.json"));
    let tmp = dir.join(format!("{slug}.json.tmp-{device}"));
    let body = serde_json::to_string_pretty(&summary(list, path_of)).map_err(|e| e.to_string())? + "\n";
    fs::write(&tmp, body)
        .and_then(|_| fs::rename(&tmp, &target))
        .map_err(|e| {
            let _ = fs::remove_file(&tmp);
            format!("samenvatting {} niet geschreven: {e}", target.display())
        })
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
        for ok in ["devices.json", "events/mbp-lynn.jsonl", "marks/mbp-lynn.jsonl", "2026/herfstwandeling.json"] {
            assert!(is_lists_file(ok), "{ok}");
        }
        for no in ["devices.json.tmp-mbp-lynn", "events/.jsonl", "events/x.txt", "notes.json", "2026/sub/x.json", "abcd/x.json"] {
            assert!(!is_lists_file(no), "{no}");
        }
    }

    fn registered_dir(devices: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let list: Vec<_> = devices.iter().map(|d| serde_json::json!({"name": d, "owner": "x"})).collect();
        fs::write(dir.path().join(DEVICES_FILE), serde_json::json!({"devices": list}).to_string()).unwrap();
        dir
    }

    fn merged_of(dir: &Path) -> BTreeMap<String, List> {
        merge(read(dir).unwrap().events)
    }

    #[test]
    fn slugs_and_unique_ids() {
        assert_eq!(slug("Herfstwandeling Ardennen"), "herfstwandeling-ardennen");
        assert_eq!(slug("  Zomer — Côte d'Azur!  "), "zomer-cote-d-azur");
        assert_eq!(slug("Kerstfeest 2026 (bij oma)"), "kerstfeest-2026-bij-oma");
        assert_eq!(slug("Æble & Straße"), "aeble-strasse");
        assert_eq!(slug("!!!"), "lijst");
        assert_eq!(slug(&"a b ".repeat(40)).len(), 59);
        let taken = |id: &str| id == "2026/ski" || id == "2026/ski-2";
        assert_eq!(new_id(2026, "Ski", &taken), "2026/ski-3");
        assert_eq!(new_id(2025, "Ski", &taken), "2025/ski");
    }

    #[test]
    fn a_change_is_one_line_per_event_in_the_contracts_shape() {
        let ts = "2026-10-20T19:42:30+02:00";
        let add = Change::Add { list: "2026/l".into(), photos: vec![("sha256:a".into(), Some("2026/10/a.CR3".into())), ("sha256:b".into(), None)] };
        assert_eq!(
            lines(&add, ts, "mbp-lynn", "lynn"),
            vec![
                format!(r#"{{"v":1,"ts":"{ts}","dev":"mbp-lynn","who":"lynn","op":"add","list":"2026/l","photo":"sha256:a","path":"2026/10/a.CR3"}}"#) + "\n",
                format!(r#"{{"v":1,"ts":"{ts}","dev":"mbp-lynn","who":"lynn","op":"add","list":"2026/l","photo":"sha256:b"}}"#) + "\n",
            ]
        );
        let archive = Change::Archive { list: "2026/l".into(), archived: false };
        assert!(lines(&archive, ts, "d", "w")[0].contains(r#""op":"unarchive","list":"2026/l"}"#));
        // what is written reads back as the same events
        let text = lines(&add, ts, "mbp-lynn", "lynn").concat();
        assert_eq!(parse_file(&text, "mbp-lynn").0.len(), 2);
        let n = now();
        assert!(DateTime::parse_from_rfc3339(&n).is_ok(), "{n}");
    }

    #[test]
    fn appending_writes_and_reads_back() {
        let dir = registered_dir(&["mbp-lynn"]);
        let create = Change::Create { list: "2026/ski".into(), name: "Ski".into() };
        append(dir.path(), "mbp-lynn", "lynn", &create, &BTreeMap::new()).unwrap();
        let merged = merged_of(dir.path());
        let add = Change::Add { list: "2026/ski".into(), photos: vec![("h1".into(), None), ("h2".into(), None)] };
        append(dir.path(), "mbp-lynn", "lynn", &add, &merged).unwrap();
        let text = fs::read_to_string(dir.path().join("events/mbp-lynn.jsonl")).unwrap();
        assert_eq!(text.lines().count(), 3);
        assert!(text.ends_with('\n'));
        let l = &merged_of(dir.path())["2026/ski"];
        assert_eq!((l.name.as_str(), l.photos.len(), l.created_by.as_str()), ("Ski", 2, "lynn"));
    }

    #[test]
    fn refusals() {
        let dir = registered_dir(&["mbp-lynn"]);
        let none = BTreeMap::new();
        let create = |name: &str| Change::Create { list: "2026/x".into(), name: name.into() };
        assert_eq!(append(dir.path(), "mbp-lynn", "lynn", &create("  "), &none).unwrap_err(), "de naam is leeg");
        assert!(append(dir.path(), "mbp-lynn", "lynn", &create(&"x".repeat(101)), &none).unwrap_err().contains("langer dan 100"));
        assert!(append(dir.path(), "mbp-other", "x", &create("Ski"), &none).unwrap_err().contains("staat niet in devices.json"));
        let add = Change::Add { list: "2026/nope".into(), photos: vec![("h".into(), None)] };
        assert_eq!(append(dir.path(), "mbp-lynn", "lynn", &add, &none).unwrap_err(), "onbekende lijst: 2026/nope");
        append(dir.path(), "mbp-lynn", "lynn", &create("Ski"), &none).unwrap();
        let merged = merged_of(dir.path());
        let empty = Change::Add { list: "2026/x".into(), photos: vec![] };
        assert_eq!(append(dir.path(), "mbp-lynn", "lynn", &empty, &merged).unwrap_err(), "geen foto's gekozen");
        assert!(!dir.path().join("events/mbp-other.jsonl").exists());
    }

    #[test]
    fn the_summary_follows_the_contract_and_prefers_the_index_path() {
        let dir = registered_dir(&["mbp-lynn", "mbp-peter"]);
        fs::create_dir(dir.path().join(EVENTS_DIR)).unwrap();
        fs::write(dir.path().join("events/mbp-lynn.jsonl"), LYNN).unwrap();
        fs::write(dir.path().join("events/mbp-peter.jsonl"), PETER).unwrap();
        let merged = merged_of(dir.path());
        let l = &merged["2026/herfstwandeling"];
        let moved = |h: &str| (h == "sha256:c71e").then(|| "2026/10/moved/CO7A9412.CR3".to_string());
        write_summary(dir.path(), "mbp-peter", l, &moved).unwrap();
        let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(dir.path().join("2026/herfstwandeling.json")).unwrap()).unwrap();
        assert_eq!(v["name"], "Herfstwandeling Ardennen");
        assert_eq!((v["count"].as_u64(), v["archived"].as_bool(), v["year"].as_i64()), (Some(1), Some(true), Some(2026)));
        assert_eq!(v["updated_by"], "mbp-peter");
        assert_eq!(v["photos"][0]["path"], "2026/10/moved/CO7A9412.CR3");
        assert_eq!(v["photos"][0]["added_by"], "lynn");
        assert!(!dir.path().join("2026/herfstwandeling.json.tmp-mbp-peter").exists());
        assert!(is_lists_file("2026/herfstwandeling.json"));
    }

    #[test]
    fn the_stamp_changes_when_a_device_appends() {
        let dir = registered_dir(&["mbp-lynn"]);
        assert_eq!(stamp(dir.path()), None, "no events folder yet");
        let create = Change::Create { list: "2026/ski".into(), name: "Ski".into() };
        append(dir.path(), "mbp-lynn", "lynn", &create, &BTreeMap::new()).unwrap();
        let before = stamp(dir.path()).unwrap();
        assert_eq!(stamp(dir.path()), Some(before));
        let add = Change::Add { list: "2026/ski".into(), photos: vec![("h".into(), None)] };
        append(dir.path(), "mbp-lynn", "lynn", &add, &merged_of(dir.path())).unwrap();
        assert_ne!(stamp(dir.path()), Some(before));
    }
}
