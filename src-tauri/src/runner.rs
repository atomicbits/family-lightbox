//! One sync at a time, in the background (F007/S05): manifests, then counting the previews, then
//! (after confirmation the first time) copying them. Progress goes to a `Sink`; in the app that
//! is a Tauri event, in the tests a recorder.

use crate::config;
use crate::pull::{self, PreviewsSummary};
use crate::sync::{self, Job};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Idle,
    Manifests,
    Lists,
    Counting,
    AwaitingConfirmation,
    Copying,
    Done,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SyncState {
    pub phase: Phase,
    pub running: bool,
    pub folder: String,
    pub files_done: u64,
    pub files_total: u64,
    pub bytes_done: u64,
    pub bytes_total: u64,
    /// The previews count, once there is one.
    pub summary: Option<PreviewsSummary>,
    pub manifests_copied: u64,
    /// Files copied from `Lists` (F003/S01).
    pub lists_copied: u64,
    pub manifests_changed: Vec<String>,
    pub previews_copied: u64,
    pub previews_bytes: u64,
    /// Files skipped after failing twice, and some of them with their error (F007/S06).
    pub failed_count: u64,
    pub failed: Vec<String>,
    pub skipped: Vec<String>,
    /// A failure, or the space message.
    pub message: Option<String>,
}

pub trait Sink: Send + Sync + 'static {
    fn progress(&self, state: &SyncState);
    fn done(&self, state: &SyncState);
    /// A year folder finished copying previews during a long sync (F008/S03): the app rebuilds
    /// the index so they show while the rest comes in.
    fn year_copied(&self) {}
    /// New manifests are in, before the previews are counted: the app builds the index, so a new
    /// Mac shows its months while the long count over the share runs.
    fn manifests_in(&self) {}
}

/// During the copy, notices when it moves on to the next year folder. The year before has then
/// copied all it had (only copies are reported, so a year that shows up copied something). At
/// most one rebuild every 2 minutes; the one after the sync covers what is left (F008/S03).
#[derive(Default)]
pub struct YearWatch {
    year: Option<String>,
    last_ask: Option<Instant>,
}

pub const REBUILD_EVERY: Duration = Duration::from_secs(120);

impl YearWatch {
    pub fn on_copy(&mut self, now: Instant, folder: &str) -> bool {
        let year = folder.split('/').next().unwrap_or_default();
        let moved_on = self.year.as_deref().is_some_and(|y| y != year);
        if self.year.as_deref() != Some(year) {
            self.year = Some(year.to_string());
        }
        let due = self.last_ask.is_none_or(|t| now.duration_since(t) >= REBUILD_EVERY);
        if moved_on && due {
            self.last_ask = Some(now);
            return true;
        }
        false
    }
}

/// At most one progress event every 200 ms, but always one when the phase changes.
pub struct Throttle {
    last: Option<(Instant, Phase)>,
}

impl Throttle {
    pub fn new() -> Self {
        Throttle { last: None }
    }

    pub fn should_emit(&mut self, now: Instant, phase: Phase) -> bool {
        let emit = match self.last {
            None => true,
            Some((t, p)) => p != phase || now.duration_since(t) >= Duration::from_millis(200),
        };
        if emit {
            self.last = Some((now, phase));
        }
        emit
    }
}

#[derive(Debug, PartialEq)]
pub enum AfterCount {
    Nothing,
    NoSpace(String),
    Confirm,
    Copy,
}

/// What to do once the previews are counted. No space wins over asking: there is nothing to ask.
pub fn decide_after_count(s: &PreviewsSummary, confirmed: bool) -> AfterCount {
    if s.copy_files == 0 {
        AfterCount::Nothing
    } else if let Some(p) = &s.problem {
        AfterCount::NoSpace(p.clone())
    } else if s.needs_confirmation && !confirmed {
        AfterCount::Confirm
    } else {
        AfterCount::Copy
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Manifests, then previews.
    All,
    /// Only the previews (after the confirmation dialog).
    Previews,
}

/// The two jobs for this Mac, or why one can't run (a share not connected).
pub struct Jobs {
    pub manifests: Result<Job, String>,
    /// Without the `Lists` share this pass is skipped.
    pub lists: Result<Job, String>,
    pub previews: Result<Job, String>,
    pub config_dir: PathBuf,
    pub free: fn(&Path) -> Option<u64>,
    /// Which previews the pass counts (F007/S07).
    pub scope: Scope,
    /// Above this the first previews pass on this Mac asks first (`pull::CONFIRM_ABOVE`).
    pub confirm_above: u64,
}

/// Every year of the previews, or at start only where something can be new (F007/S07).
#[derive(Debug, Clone, PartialEq)]
pub enum Scope {
    Full,
    /// The years whose manifests this sync changed and those with photos the index has without a
    /// preview, apart from the previews this Mac failed to make (`record`); full when the index
    /// can't be read.
    WhereNew { index: PathBuf, record: PathBuf },
}

/// How long the sync at start may count only where something can be new.
pub const FULL_EVERY: chrono::Duration = chrono::Duration::days(7);

/// Whether the sync at start should count every year: never a full count yet, or the last more
/// than `FULL_EVERY` ago (or unreadable).
pub fn full_due(last: Option<&str>, now: chrono::DateTime<chrono::FixedOffset>) -> bool {
    match last.and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok()) {
        Some(t) => now - t > FULL_EVERY,
        None => true,
    }
}

/// The years to count: those of the changed manifests (`2026/07`, `2012/scans Peter`) and those
/// without previews, sorted, each once; `None` for every year (the index couldn't be read).
pub fn years_to_count(changed: &[String], missing: Option<Vec<String>>) -> Option<Vec<String>> {
    let mut years: Vec<String> = missing?;
    years.extend(changed.iter().filter_map(|c| c.get(..4)).map(str::to_string));
    years.sort();
    years.dedup();
    Some(years)
}

struct Counted {
    at: Instant,
    job: Job,
    plan: sync::Plan,
}

#[derive(Default)]
struct Inner {
    state: SyncState,
    counted: Option<Counted>,
}

#[derive(Clone, Default)]
pub struct SyncRunner {
    inner: Arc<Mutex<Inner>>,
    cancel: Arc<AtomicBool>,
    /// The sync at launch ran in this process; a reloaded window doesn't start it again.
    launched: Arc<AtomicBool>,
}

/// Keeps the moment of a full count that finished (F007/S07); a failure to write only means the
/// next start counts in full again.
fn record_full_count(config_dir: &Path) {
    let mut state = config::load_state(config_dir);
    state.previews_full_at = Some(chrono::Local::now().fixed_offset().to_rfc3339());
    let _ = config::save_state(config_dir, &state);
}

impl SyncRunner {
    pub fn status(&self) -> SyncState {
        self.inner.lock().map(|i| i.state.clone()).unwrap_or_default()
    }

    /// True the first time only: whether this is the sync at launch.
    pub fn first_launch(&self) -> bool {
        !self.launched.swap(true, Ordering::Relaxed)
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn start(&self, kind: Kind, confirmed: bool, jobs: Jobs, sink: Arc<dyn Sink>) -> Result<std::thread::JoinHandle<()>, String> {
        {
            let mut inner = self.inner.lock().map_err(|e| e.to_string())?;
            if inner.state.running {
                return Err(tr!("er loopt al een synchronisatie", "a sync is running already"));
            }
            inner.state = SyncState {
                running: true,
                phase: if kind == Kind::All { Phase::Manifests } else { Phase::Counting },
                ..SyncState::default()
            };
        }
        self.cancel.store(false, Ordering::Relaxed);
        let me = self.clone();
        Ok(std::thread::spawn(move || me.run(kind, confirmed, jobs, sink.as_ref())))
    }

    fn update(&self, f: impl FnOnce(&mut SyncState)) -> SyncState {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut inner.state);
        inner.state.clone()
    }

    fn finish(&self, sink: &dyn Sink, phase: Phase, message: Option<String>) {
        let s = self.update(|s| {
            s.phase = phase;
            s.running = false;
            s.message = message;
        });
        sink.done(&s);
    }

    fn run(&self, kind: Kind, confirmed: bool, jobs: Jobs, sink: &dyn Sink) {
        let throttle = Mutex::new(Throttle::new());
        let years = Mutex::new(YearWatch::default());
        let report = |phase: Phase, p: &sync::Progress| {
            if phase == Phase::Copying && years.lock().map(|mut y| y.on_copy(Instant::now(), p.folder)).unwrap_or(false) {
                sink.year_copied();
            }
            let s = self.update(|s| {
                s.phase = phase;
                s.folder = p.folder.to_string();
                s.files_done = p.files_done;
                s.files_total = p.files_total;
                s.bytes_done = p.bytes_done;
                s.bytes_total = p.bytes_total;
            });
            if throttle.lock().map(|mut t| t.should_emit(Instant::now(), phase)).unwrap_or(true) {
                sink.progress(&s);
            }
        };

        if kind == Kind::All {
            let job = match &jobs.manifests {
                Ok(j) => j,
                Err(e) => return self.finish(sink, Phase::Failed, Some(e.clone())),
            };
            sink.progress(&self.status());
            let out = pull::run_manifests(job, &mut |p| report(Phase::Manifests, p), &self.cancel);
            self.update(|s| {
                s.manifests_copied = out.copied;
                s.manifests_changed = out.changed.clone();
                s.failed_count += out.failed_count;
                s.failed.extend(out.failed.iter().cloned());
                s.skipped = out.skipped.clone();
            });
            if out.cancelled {
                return self.finish(sink, Phase::Cancelled, None);
            }
            if let Some(e) = out.error {
                return self.finish(sink, Phase::Failed, Some(e));
            }
            if let Ok(job) = &jobs.lists {
                let s = self.update(|s| s.phase = Phase::Lists);
                sink.progress(&s);
                // the lists are small; a failure here doesn't stop the previews
                match sync::plan(job, &mut |p| report(Phase::Lists, p), &self.cancel) {
                    Ok(plan) => {
                        let r = sync::execute(job, &plan, &mut |p| report(Phase::Lists, p), &self.cancel);
                        self.update(|s| {
                            s.lists_copied = r.copied_missing + r.copied_newer + r.copied_changed;
                            s.failed_count += r.failed_count;
                            s.failed.extend(r.failed.iter().cloned());
                        });
                        if r.cancelled {
                            return self.finish(sink, Phase::Cancelled, None);
                        }
                    }
                    Err(sync::Stop::Cancelled) => return self.finish(sink, Phase::Cancelled, None),
                    Err(sync::Stop::Failed(_)) => {}
                }
            }
            if out.copied > 0 {
                sink.manifests_in();
            }
        }

        let mut job = match jobs.previews {
            Ok(j) => j,
            // Without a Previews mount only the manifests run.
            Err(_) if kind == Kind::All => return self.finish(sink, Phase::Done, None),
            Err(e) => return self.finish(sink, Phase::Failed, Some(e)),
        };
        // At start, only the years where something can be new (F007/S07).
        if let (Kind::All, Scope::WhereNew { index, record }) = (kind, &jobs.scope) {
            let failed: std::collections::HashSet<String> = crate::previews_plan::Record::open(record)
                .and_then(|r| r.failures())
                .map(|f| f.into_iter().map(|x| x.path).collect())
                .unwrap_or_default();
            let missing = crate::db::Db::open(index).and_then(|d| d.missing_preview_years(&failed)).ok();
            let changed = self.status().manifests_changed;
            match years_to_count(&changed, missing) {
                Some(years) if years.is_empty() => return self.finish(sink, Phase::Done, None),
                Some(years) => job.only = Some(years),
                None => {}
            }
        }
        let full = job.only.is_none();

        // A plan counted for the confirmation dialog is reused while it is fresh.
        let reuse = if kind == Kind::Previews && confirmed {
            self.inner.lock().ok().and_then(|mut i| i.counted.take()).filter(|c| pull::plan_is_fresh(c.at, Instant::now()))
        } else {
            None
        };
        let (job, plan) = match reuse {
            Some(c) => (c.job, c.plan),
            None => {
                let s = self.update(|s| s.phase = Phase::Counting);
                sink.progress(&s);
                match sync::plan(&job, &mut |p| report(Phase::Counting, p), &self.cancel) {
                    Ok(p) => {
                        if full {
                            record_full_count(&jobs.config_dir);
                        }
                        (job, p)
                    }
                    Err(sync::Stop::Cancelled) => return self.finish(sink, Phase::Cancelled, None),
                    Err(sync::Stop::Failed(e)) => return self.finish(sink, Phase::Failed, Some(e)),
                }
            }
        };
        let summary = pull::summarize(&plan, (jobs.free)(&job.local), &config::load_state(&jobs.config_dir), jobs.confirm_above);
        self.update(|s| {
            s.summary = Some(summary.clone());
            for y in &summary.skipped {
                if !s.skipped.contains(y) {
                    s.skipped.push(y.clone());
                }
            }
        });
        match decide_after_count(&summary, confirmed) {
            AfterCount::Nothing => self.finish(sink, Phase::Done, None),
            AfterCount::NoSpace(msg) => self.finish(sink, Phase::Failed, Some(msg)),
            AfterCount::Confirm => {
                if let Ok(mut i) = self.inner.lock() {
                    i.counted = Some(Counted { at: Instant::now(), job, plan });
                }
                self.finish(sink, Phase::AwaitingConfirmation, None)
            }
            AfterCount::Copy => {
                let s = self.update(|s| s.phase = Phase::Copying);
                sink.progress(&s);
                let free = (jobs.free)(&job.local);
                match pull::run_previews(&job, &plan, confirmed, &jobs.config_dir, free, jobs.confirm_above, &mut |p| report(Phase::Copying, p), &self.cancel) {
                    Err(e) => self.finish(sink, Phase::Failed, Some(e)),
                    Ok(o) => {
                        self.update(|s| {
                            s.previews_copied = o.copied;
                            s.previews_bytes = o.bytes;
                            s.failed_count += o.failed_count;
                            s.failed.extend(o.failed.iter().cloned());
                        });
                        if o.cancelled {
                            self.finish(sink, Phase::Cancelled, None)
                        } else if let Some(e) = o.error {
                            self.finish(sink, Phase::Failed, Some(e))
                        } else {
                            self.finish(sink, Phase::Done, None)
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[derive(Default)]
    struct Recorder {
        events: Mutex<Vec<(&'static str, Phase)>>,
    }

    impl Sink for Recorder {
        fn progress(&self, s: &SyncState) {
            self.events.lock().unwrap().push(("progress", s.phase));
        }
        fn done(&self, s: &SyncState) {
            self.events.lock().unwrap().push(("done", s.phase));
        }
        fn year_copied(&self) {
            self.events.lock().unwrap().push(("year", Phase::Copying));
        }
        fn manifests_in(&self) {
            self.events.lock().unwrap().push(("manifests_in", Phase::Lists));
        }
    }

    #[test]
    fn a_rebuild_is_asked_when_a_year_is_done_at_most_every_2_minutes() {
        let mut w = YearWatch::default();
        let t0 = Instant::now();
        assert!(!w.on_copy(t0, "2022/01/a"), "the first year is still copying");
        assert!(!w.on_copy(t0, "2022/12/b"));
        assert!(w.on_copy(t0 + Duration::from_secs(1), "2023/01"), "2022 is done");
        assert!(!w.on_copy(t0 + Duration::from_secs(60), "2024/01"), "2023 done, but within 2 minutes");
        assert!(!w.on_copy(t0 + Duration::from_secs(200), "2024/05"), "same year: nothing finished");
        assert!(w.on_copy(t0 + Duration::from_secs(201), "2025/01"), "2024 done, 2 minutes passed");
        assert!(!w.on_copy(t0 + Duration::from_secs(400), "2025/02"));
    }

    fn summary(copy_files: u64, problem: Option<&str>, needs_confirmation: bool) -> PreviewsSummary {
        PreviewsSummary { copy_files, copy_bytes: copy_files, kept: 0, skipped: vec![], free: Some(1), problem: problem.map(String::from), needs_confirmation }
    }

    #[test]
    fn what_happens_after_counting() {
        assert_eq!(decide_after_count(&summary(0, None, true), false), AfterCount::Nothing);
        assert_eq!(decide_after_count(&summary(3, Some("tekort"), true), false), AfterCount::NoSpace("tekort".into()));
        assert_eq!(decide_after_count(&summary(3, None, true), false), AfterCount::Confirm);
        assert_eq!(decide_after_count(&summary(3, None, true), true), AfterCount::Copy);
        assert_eq!(decide_after_count(&summary(3, None, false), false), AfterCount::Copy);
    }

    #[test]
    fn throttle_every_200_ms_and_on_every_phase_change() {
        let mut t = Throttle::new();
        let t0 = Instant::now();
        assert!(t.should_emit(t0, Phase::Manifests));
        assert!(!t.should_emit(t0 + Duration::from_millis(100), Phase::Manifests));
        assert!(t.should_emit(t0 + Duration::from_millis(150), Phase::Counting), "phase change");
        assert!(!t.should_emit(t0 + Duration::from_millis(300), Phase::Counting));
        assert!(t.should_emit(t0 + Duration::from_millis(400), Phase::Counting));
    }

    fn put(root: &Path, rel: &str, body: &str) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }

    fn plenty(_: &Path) -> Option<u64> {
        Some(pull::RESERVE * 10)
    }

    fn jobs(lib_r: &Path, lib_l: &Path, pv_r: &Path, pv_l: &Path, cfg: &Path) -> Jobs {
        Jobs {
            manifests: Ok(Job { remote: lib_r.into(), local: lib_l.into(), filter: pull::is_manifest, only: None, max_depth: Some(2) }),
            lists: Err("niet verbonden".into()),
            previews: Ok(Job { remote: pv_r.into(), local: pv_l.into(), filter: pull::is_preview, only: None, max_depth: None }),
            config_dir: cfg.into(),
            free: plenty,
            scope: Scope::Full,
            // the tests' files are bytes: every first pass asks, as a large one would
            confirm_above: 0,
        }
    }

    #[test]
    fn the_lists_are_copied_after_the_manifests() {
        let d: Vec<_> = (0..7).map(|_| tempfile::tempdir().unwrap()).collect();
        let (lib_r, lib_l, pv_r, pv_l, cfg, li_r, li_l) =
            (d[0].path(), d[1].path(), d[2].path(), d[3].path(), d[4].path(), d[5].path(), d[6].path());
        put(li_r, "devices.json", "{}");
        put(li_r, "events/mbp-lynn.jsonl", "x\n");
        put(li_r, "2026/ski.json", "{}");
        put(li_r, "devices.json.tmp-mbp-lynn", "{}");
        let mut j = jobs(lib_r, lib_l, pv_r, pv_l, cfg);
        j.lists = Ok(Job { remote: li_r.into(), local: li_l.into(), filter: crate::lists::is_lists_file, only: None, max_depth: Some(1) });
        let r = SyncRunner::default();
        r.start(Kind::All, false, j, Arc::new(Recorder::default())).unwrap().join().unwrap();
        assert_eq!(r.status().lists_copied, 3);
        assert!(li_l.join("events/mbp-lynn.jsonl").is_file());
        assert!(!li_l.join("devices.json.tmp-mbp-lynn").exists());
    }

    #[test]
    fn the_whole_sequence_with_confirmation() {
        let d: Vec<_> = (0..5).map(|_| tempfile::tempdir().unwrap()).collect();
        let (lib_r, lib_l, pv_r, pv_l, cfg) = (d[0].path(), d[1].path(), d[2].path(), d[3].path(), d[4].path());
        put(lib_r, "2026/07/.manifest.json", r#"{"files": []}"#);
        put(pv_r, "2026/07/a.JPG.jpg", "a");
        put(pv_r, "2026/07/a.JPG.t.jpg", "t");
        let runner = SyncRunner::default();
        let rec = Arc::new(Recorder::default());

        runner.start(Kind::All, false, jobs(lib_r, lib_l, pv_r, pv_l, cfg), rec.clone()).unwrap().join().unwrap();
        let st = runner.status();
        assert_eq!(st.phase, Phase::AwaitingConfirmation);
        assert!(!st.running);
        assert_eq!(st.manifests_copied, 1);
        assert_eq!(st.summary.as_ref().unwrap().copy_files, 2);
        assert!(!pv_l.join("2026").exists(), "nothing copied before confirmation");
        let phases: Vec<_> = rec.events.lock().unwrap().iter().map(|e| e.1).collect();
        assert_eq!(phases.first(), Some(&Phase::Manifests));
        assert!(phases.contains(&Phase::Counting));
        assert_eq!(rec.events.lock().unwrap().last(), Some(&("done", Phase::AwaitingConfirmation)));
        // the index is asked for before the previews are counted (a new Mac's long wait)
        let order: Vec<_> = rec.events.lock().unwrap().iter().map(|e| e.0).collect();
        let at = |what: &str| order.iter().position(|e| *e == what);
        let counting = rec.events.lock().unwrap().iter().position(|e| e.1 == Phase::Counting);
        assert!(at("manifests_in").is_some() && at("manifests_in") < counting, "{order:?}");

        let rec2 = Arc::new(Recorder::default());
        runner.start(Kind::Previews, true, jobs(lib_r, lib_l, pv_r, pv_l, cfg), rec2.clone()).unwrap().join().unwrap();
        let st = runner.status();
        assert_eq!((st.phase, st.previews_copied), (Phase::Done, 2));
        assert!(pv_l.join("2026/07/a.JPG.t.jpg").exists());
        let ev = rec2.events.lock().unwrap();
        assert!(!ev.iter().any(|e| e.1 == Phase::Counting), "the counted plan was reused");
        assert!(ev.iter().any(|e| e.1 == Phase::Copying));
        assert_eq!(ev.last(), Some(&("done", Phase::Done)));
        assert!(config::load_state(cfg).previews_confirmed_at.is_some());
        assert!(!ev.iter().any(|e| e.0 == "year"), "one year: the rebuild after the sync covers it");
        assert!(!ev.iter().any(|e| e.0 == "manifests_in"), "only the previews this time");
    }

    #[test]
    fn the_years_to_count_at_start() {
        let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        assert_eq!(years_to_count(&s(&["2026/07", "2012/scans Peter"]), Some(vec![])), Some(s(&["2012", "2026"])));
        assert_eq!(years_to_count(&[], Some(s(&["2004", "2019"]))), Some(s(&["2004", "2019"])));
        assert_eq!(years_to_count(&s(&["2019/05"]), Some(s(&["2019", "2004"]))), Some(s(&["2004", "2019"])));
        assert_eq!(years_to_count(&[], Some(vec![])), Some(vec![]), "nothing can be new");
        assert_eq!(years_to_count(&s(&["2026/07"]), None), None, "no index: every year");
    }

    #[test]
    fn a_full_count_once_a_week() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-11-02T21:00:00+01:00").unwrap();
        assert!(full_due(None, now));
        assert!(full_due(Some("nonsense"), now));
        assert!(!full_due(Some("2026-10-30T09:00:00+01:00"), now));
        assert!(full_due(Some("2026-10-26T20:00:00+01:00"), now));
    }

    #[test]
    fn at_start_only_where_something_can_be_new() {
        let d: Vec<_> = (0..5).map(|_| tempfile::tempdir().unwrap()).collect();
        let (lib_r, lib_l, pv_r, pv_l, cfg) = (d[0].path(), d[1].path(), d[2].path(), d[3].path(), d[4].path());
        let manifest = |n: usize| {
            let files: Vec<String> = (0..n)
                .map(|i| format!(r#"{{"path": "2019/05/Cam/b{i}.JPG", "size": 1, "hash": "sha256:b{i}", "kind": "photo", "source": "Cam"}}"#))
                .collect();
            format!(r#"{{"month": "2019/05", "files": [{}]}}"#, files.join(","))
        };
        put(lib_r, "2019/05/.manifest.json", &manifest(1));
        put(pv_r, "2019/05/Cam/b0.JPG.jpg", "b");
        let runner = SyncRunner::default();
        let index = cfg.join("index.sqlite");
        let record = cfg.join(crate::previews_plan::RECORD_FILE);
        let where_new = || Jobs { scope: Scope::WhereNew { index: index.clone(), record: record.clone() }, ..jobs(lib_r, lib_l, pv_r, pv_l, cfg) };
        let counted = |rec: &Recorder| rec.events.lock().unwrap().iter().any(|e| e.1 == Phase::Counting);

        // without an index: every year, and the full count is kept
        let rec = Arc::new(Recorder::default());
        runner.start(Kind::All, false, where_new(), rec.clone()).unwrap().join().unwrap();
        assert!(counted(&rec));
        assert!(config::load_state(cfg).previews_full_at.is_some());

        // every preview there or failed on this Mac, no manifest changed: no count at all
        put(pv_l, "2019/05/Cam/b0.JPG.jpg", "b");
        put(lib_r, "2004/03/.manifest.json", r#"{"month": "2004/03", "files": [{"path": "2004/03/Cam/x.AVI", "size": 1, "hash": "sha256:x", "kind": "video"}]}"#);
        runner.start(Kind::All, false, where_new(), Arc::new(Recorder::default())).unwrap().join().unwrap();
        crate::index::build(lib_l, pv_l, &index).unwrap();
        crate::previews_plan::Record::open(&record).unwrap().failed("2004/03/Cam/x.AVI", "sha256:x", "geen beeld").unwrap();
        put(pv_r, "2019/05/Cam/other.JPG.jpg", "o");
        let rec = Arc::new(Recorder::default());
        runner.start(Kind::All, false, where_new(), rec.clone()).unwrap().join().unwrap();
        assert!(!counted(&rec));
        assert_eq!(runner.status().phase, Phase::Done);
        assert!(!pv_l.join("2019/05/Cam/other.JPG.jpg").exists());

        // a manifest changed on the NAS: its year is counted
        put(lib_r, "2019/05/.manifest.json", &manifest(2));
        put(pv_r, "2020/01/Cam/c.JPG.jpg", "c");
        let rec = Arc::new(Recorder::default());
        runner.start(Kind::All, false, where_new(), rec.clone()).unwrap().join().unwrap();
        assert!(counted(&rec));
        let summary = runner.status().summary.unwrap();
        assert_eq!(summary.copy_files, 1, "2019's new preview, not 2020's: {summary:?}");

        // by hand: every year
        let rec = Arc::new(Recorder::default());
        runner.start(Kind::All, false, jobs(lib_r, lib_l, pv_r, pv_l, cfg), rec.clone()).unwrap().join().unwrap();
        assert_eq!(runner.status().summary.unwrap().copy_files, 2);
    }

    #[test]
    fn a_rebuild_is_asked_while_copying_several_years() {
        let d: Vec<_> = (0..5).map(|_| tempfile::tempdir().unwrap()).collect();
        let (lib_r, lib_l, pv_r, pv_l, cfg) = (d[0].path(), d[1].path(), d[2].path(), d[3].path(), d[4].path());
        for y in ["2022", "2023", "2024"] {
            put(pv_r, &format!("{y}/07/a.JPG.jpg"), "a");
        }
        config::save_state(cfg, &config::AppState { previews_confirmed_at: Some("2026-10-27T08:00:00+01:00".into()), ..Default::default() }).unwrap();
        let rec = Arc::new(Recorder::default());
        SyncRunner::default().start(Kind::Previews, false, jobs(lib_r, lib_l, pv_r, pv_l, cfg), rec.clone()).unwrap().join().unwrap();
        let ev = rec.events.lock().unwrap();
        assert_eq!(ev.last(), Some(&("done", Phase::Done)));
        // 2022 done when 2023 starts; 2023 done within 2 minutes is left to the rebuild after the sync
        assert_eq!(ev.iter().filter(|e| e.0 == "year").count(), 1);
    }

    #[test]
    fn the_sync_at_launch_counts_once_per_process() {
        let r = SyncRunner::default();
        assert!(r.first_launch());
        assert!(!r.first_launch());
        assert!(!r.clone().first_launch(), "a clone shares it");
    }

    #[test]
    fn a_second_start_is_refused_while_one_runs() {
        let runner = SyncRunner::default();
        runner.inner.lock().unwrap().state.running = true;
        let d = tempfile::tempdir().unwrap();
        let err = runner.start(Kind::All, false, jobs(d.path(), d.path(), d.path(), d.path(), d.path()), Arc::new(Recorder::default())).unwrap_err();
        assert_eq!(err, "er loopt al een synchronisatie");
    }

    #[test]
    fn without_a_previews_mount_only_the_manifests_run() {
        let d: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
        put(d[0].path(), "2026/07/.manifest.json", r#"{"files": []}"#);
        let mut j = jobs(d[0].path(), d[1].path(), d[0].path(), d[1].path(), d[2].path());
        j.previews = Err("gedeelde map 'Previews' is niet verbonden".into());
        let runner = SyncRunner::default();
        runner.start(Kind::All, false, j, Arc::new(Recorder::default())).unwrap().join().unwrap();
        let st = runner.status();
        assert_eq!((st.phase, st.manifests_copied, st.message), (Phase::Done, 1, None));
    }
}
