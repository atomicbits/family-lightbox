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
    pub manifests_changed: Vec<String>,
    pub previews_copied: u64,
    pub previews_bytes: u64,
    pub skipped: Vec<String>,
    /// A failure, or the space message.
    pub message: Option<String>,
}

pub trait Sink: Send + Sync + 'static {
    fn progress(&self, state: &SyncState);
    fn done(&self, state: &SyncState);
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
    pub previews: Result<Job, String>,
    pub config_dir: PathBuf,
    pub free: fn(&Path) -> Option<u64>,
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
}

impl SyncRunner {
    pub fn status(&self) -> SyncState {
        self.inner.lock().map(|i| i.state.clone()).unwrap_or_default()
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn start(&self, kind: Kind, confirmed: bool, jobs: Jobs, sink: Arc<dyn Sink>) -> Result<std::thread::JoinHandle<()>, String> {
        {
            let mut inner = self.inner.lock().map_err(|e| e.to_string())?;
            if inner.state.running {
                return Err("er loopt al een synchronisatie".into());
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
        let report = |phase: Phase, p: &sync::Progress| {
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
                s.skipped = out.skipped.clone();
            });
            if out.cancelled {
                return self.finish(sink, Phase::Cancelled, None);
            }
            if let Some(e) = out.error {
                return self.finish(sink, Phase::Failed, Some(e));
            }
        }

        let job = match jobs.previews {
            Ok(j) => j,
            // Without a Previews mount only the manifests run.
            Err(_) if kind == Kind::All => return self.finish(sink, Phase::Done, None),
            Err(e) => return self.finish(sink, Phase::Failed, Some(e)),
        };

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
                    Ok(p) => (job, p),
                    Err(sync::Stop::Cancelled) => return self.finish(sink, Phase::Cancelled, None),
                    Err(sync::Stop::Failed(e)) => return self.finish(sink, Phase::Failed, Some(e)),
                }
            }
        };
        let summary = pull::summarize(&plan, (jobs.free)(&job.local), &config::load_state(&jobs.config_dir));
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
                match pull::run_previews(&job, &plan, confirmed, &jobs.config_dir, free, &mut |p| report(Phase::Copying, p), &self.cancel) {
                    Err(e) => self.finish(sink, Phase::Failed, Some(e)),
                    Ok(o) => {
                        self.update(|s| {
                            s.previews_copied = o.copied;
                            s.previews_bytes = o.bytes;
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
            previews: Ok(Job { remote: pv_r.into(), local: pv_l.into(), filter: pull::is_preview, only: None, max_depth: None }),
            config_dir: cfg.into(),
            free: plenty,
        }
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
