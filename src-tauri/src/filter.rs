//! Filters (F002/S02): which photos match the filter bar's stars, hearts, kind and camera. The
//! stars and hearts come from the frontend as the set of photos that pass them, since it shows
//! a mark change before the NAS has it; the kind and the camera come from the index.

use serde::Deserialize;
use std::collections::HashSet;

/// The photos that pass the stars and hearts, by hash; or, with `exclude`, those that don't
/// (for "without stars", where the rated ones are the short list).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MarksSet {
    pub hashes: HashSet<String>,
    #[serde(default)]
    pub exclude: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Filter {
    /// None: stars and hearts don't filter.
    #[serde(default)]
    pub marks: Option<MarksSet>,
    /// "photo", "video" (not a Live Photo), "live"; empty: every kind.
    #[serde(default)]
    pub kinds: Vec<String>,
    /// Camera names, with "" for unknown; empty: every camera (F002/S04).
    #[serde(default)]
    pub cameras: Vec<String>,
    /// Hidden photos are left out, except in these months (feature 14).
    #[serde(default)]
    pub hidden: HashSet<String>,
    #[serde(default)]
    pub revealed: HashSet<String>,
}

/// What the filter needs to know of one photo or video.
pub struct Candidate<'a> {
    pub hash: &'a str,
    pub kind: &'a str,
    pub live: bool,
    pub month: &'a str,
    pub camera: Option<&'a str>,
}

impl Filter {
    /// Whether any part filters (hidden photos alone don't count: they are left out anyway).
    pub fn is_set(&self) -> bool {
        self.marks.is_some() || !self.kinds.is_empty() || !self.cameras.is_empty()
    }

    pub fn matches(&self, c: &Candidate) -> bool {
        if self.hidden.contains(c.hash) && !self.revealed.contains(c.month) {
            return false;
        }
        if self.marks.as_ref().is_some_and(|m| m.hashes.contains(c.hash) == m.exclude) {
            return false;
        }
        if !self.kinds.is_empty() {
            let kind = match (c.kind, c.live) {
                ("video", true) => "live",
                (k, _) => k,
            };
            if !self.kinds.iter().any(|k| k == kind) {
                return false;
            }
        }
        if !self.cameras.is_empty() && !self.cameras.iter().any(|k| k == c.camera.unwrap_or("")) {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c<'a>(hash: &'a str, kind: &'a str, live: bool, camera: Option<&'a str>) -> Candidate<'a> {
        Candidate { hash, kind, live, month: "2026/07", camera }
    }

    #[test]
    fn an_empty_filter_lets_everything_through_but_hidden_photos() {
        let f = Filter { hidden: ["h".to_string()].into(), ..Default::default() };
        assert!(!f.is_set());
        assert!(f.matches(&c("a", "photo", false, None)));
        assert!(!f.matches(&c("h", "photo", false, None)));
        let shown = Filter { revealed: ["2026/07".to_string()].into(), ..f };
        assert!(shown.matches(&c("h", "photo", false, None)));
    }

    #[test]
    fn marks_include_or_exclude() {
        let f = Filter { marks: Some(MarksSet { hashes: ["a".to_string()].into(), exclude: false }), ..Default::default() };
        assert!(f.matches(&c("a", "photo", false, None)) && !f.matches(&c("b", "photo", false, None)));
        let none = Filter { marks: Some(MarksSet { hashes: ["a".to_string()].into(), exclude: true }), ..Default::default() };
        assert!(!none.matches(&c("a", "photo", false, None)) && none.matches(&c("b", "photo", false, None)));
    }

    #[test]
    fn kinds_tell_live_photos_from_videos_and_or_within() {
        let f = Filter { kinds: vec!["live".into()], ..Default::default() };
        assert!(f.matches(&c("a", "video", true, None)));
        assert!(!f.matches(&c("b", "video", false, None)) && !f.matches(&c("c", "photo", false, None)));
        let both = Filter { kinds: vec!["photo".into(), "video".into()], ..Default::default() };
        assert!(both.matches(&c("c", "photo", false, None)) && both.matches(&c("b", "video", false, None)));
        assert!(!both.matches(&c("a", "video", true, None)));
    }

    #[test]
    fn cameras_with_unknown_and_everything_together() {
        let f = Filter {
            marks: Some(MarksSet { hashes: ["a".to_string(), "b".to_string()].into(), exclude: false }),
            kinds: vec!["photo".into()],
            cameras: vec!["Canon EOS R".into(), "".into()],
            ..Default::default()
        };
        assert!(f.matches(&c("a", "photo", false, Some("Canon EOS R"))));
        assert!(f.matches(&c("b", "photo", false, None)));
        assert!(!f.matches(&c("a", "photo", false, Some("Apple iPhone 13 Pro"))));
        assert!(!f.matches(&c("a", "video", false, Some("Canon EOS R"))));
        assert!(!f.matches(&c("z", "photo", false, Some("Canon EOS R"))));
    }
}
