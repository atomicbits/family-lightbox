//! Makes the missing previews in the background (F012/S03): the plan of S02, then for every preview
//! to make, the original from this Mac or from the NAS, the preview into the local folder and onto
//! the `Previews` share. A few at a time at a low priority; it can be paused and starts again where
//! it stopped, because the record is written per preview.

use crate::preview::{self, Kind};
use crate::previews_plan::{self, Record, State, Wanted};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct MakeState {
    pub running: bool,
    /// Paused by hand; stays paused after a restart until resumed.
    pub paused: bool,
    /// Previews to make in this run, and how many are handled so far.
    pub total: u64,
    pub done: u64,
    pub made: u64,
    /// Found on the NAS just before making it (another Mac was first): copied instead.
    pub copied: u64,
    pub failed: u64,
    /// The original couldn't be read now (not local, the NAS not connected): next time.
    pub skipped: u64,
    /// Made locally, but writing it to the NAS failed.
    pub nas_failed: u64,
    pub message: Option<String>,
}

pub trait MakeSink: Send + Sync + 'static {
    fn progress(&self, state: &MakeState);
    fn done(&self, state: &MakeState);
    /// Previews were made since the last call; the app rebuilds the index now and then, so they
    /// show in the grid while the rest is made.
    fn some_made(&self) {}
}

#[derive(Clone, Debug)]
pub struct MakeJob {
    pub index: PathBuf,
    pub record: PathBuf,
    pub previews: PathBuf,
    pub library: PathBuf,
    /// The `Library` share, for originals that aren't on this Mac.
    pub library_nas: Option<PathBuf>,
    /// The `Previews` share, to put new previews on and to look for ones another Mac made.
    pub previews_nas: Option<PathBuf>,
    pub workers: usize,
}

/// Half the cores, at most 4, so browsing stays smooth.
pub fn default_workers() -> usize {
    let cores = std::thread::available_parallelism().map_or(2, |n| n.get());
    (cores / 2).clamp(1, 4)
}

/// How often the grid is given what was made meanwhile.
pub const SHOW_EVERY: Duration = Duration::from_secs(120);

#[derive(Default)]
struct Inner {
    state: MakeState,
    /// What is left, last item first (workers pop from the end).
    queue: Vec<Wanted>,
    focus: Option<String>,
}

#[derive(Clone, Default)]
pub struct MakeRunner {
    inner: Arc<Mutex<Inner>>,
    stop: Arc<AtomicBool>,
}

/// What happened to one preview.
#[derive(Debug, PartialEq)]
enum Outcome {
    Made { nas_error: Option<String> },
    Copied,
    Failed(String),
    Skipped,
}

impl MakeRunner {
    pub fn status(&self) -> MakeState {
        self.inner.lock().map(|i| i.state.clone()).unwrap_or_default()
    }

    pub fn running(&self) -> bool {
        self.status().running
    }

    /// Restore the pause from a previous run (the app's state file).
    pub fn set_paused(&self, paused: bool) {
        if let Ok(mut i) = self.inner.lock() {
            i.state.paused = paused;
        }
    }

    /// Stop after the previews being made now; `paused` keeps it from starting by itself.
    pub fn stop(&self, paused: bool) {
        self.set_paused(paused);
        self.stop.store(true, Ordering::Relaxed);
    }

    /// Put this month first in what is left.
    pub fn focus(&self, month: Option<String>) {
        let Ok(mut i) = self.inner.lock() else { return };
        if i.focus == month {
            return;
        }
        i.focus = month;
        let focus = i.focus.clone();
        previews_plan::order(&mut i.queue, focus.as_deref());
        i.queue.reverse();
    }

    /// Plan and make in the background, unless it runs already or is paused. Returns the thread.
    pub fn start(&self, job: MakeJob, sink: Arc<dyn MakeSink>) -> Option<std::thread::JoinHandle<()>> {
        {
            let mut i = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if i.state.running || i.state.paused {
                return None;
            }
            i.state = MakeState { running: true, ..MakeState::default() };
        }
        self.stop.store(false, Ordering::Relaxed);
        let me = self.clone();
        Some(std::thread::spawn(move || me.run(job, sink)))
    }

    fn update(&self, f: impl FnOnce(&mut MakeState)) -> MakeState {
        let mut i = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut i.state);
        i.state.clone()
    }

    fn finish(&self, sink: &dyn MakeSink, message: Option<String>) {
        let s = self.update(|s| {
            s.running = false;
            s.message = message;
        });
        sink.done(&s);
    }

    fn run(&self, job: MakeJob, sink: Arc<dyn MakeSink>) {
        let mut record = match Record::open(&job.record) {
            Ok(r) => r,
            Err(e) => return self.finish(sink.as_ref(), Some(e)),
        };
        let focus = self.inner.lock().ok().and_then(|i| i.focus.clone());
        // without the NAS: the sync has just copied what it has; each preview checks it again
        let plan = match previews_plan::plan(&job.index, &mut record, &job.previews, None, focus.as_deref()) {
            Ok(p) => p,
            Err(e) => return self.finish(sink.as_ref(), Some(e)),
        };
        // without the Library share, only what has its original here can be made now
        let reachable = |w: &Wanted| job.library_nas.is_some() || job.library.join(&w.path).is_file();
        let mut todo: Vec<Wanted> = plan.todo.into_iter().filter(|w| w.state == State::Make && reachable(w)).collect();
        todo.reverse();
        let total = todo.len() as u64;
        {
            let mut i = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            i.queue = todo;
            i.state.total = total;
        }
        if total == 0 {
            return self.finish(sink.as_ref(), None);
        }
        sink.progress(&self.status());

        let record = Mutex::new(record);
        let last_emit = Mutex::new(Instant::now());
        let last_show = Mutex::new((Instant::now(), false));
        std::thread::scope(|scope| {
            for _ in 0..job.workers.max(1) {
                scope.spawn(|| {
                    lower_priority();
                    while !self.stop.load(Ordering::Relaxed) {
                        let Some(w) = self.inner.lock().ok().and_then(|mut i| i.queue.pop()) else { break };
                        let outcome = make_one(&job, &w);
                        let saved = match &outcome {
                            Outcome::Made { .. } | Outcome::Copied => record.lock().map_err(|e| e.to_string()).and_then(|mut r| r.made(&w.path, &w.hash)),
                            Outcome::Failed(e) => record.lock().map_err(|e| e.to_string()).and_then(|mut r| r.failed(&w.path, &w.hash, e)),
                            Outcome::Skipped => Ok(()),
                        };
                        let s = self.update(|s| {
                            s.done += 1;
                            match &outcome {
                                Outcome::Made { nas_error } => {
                                    s.made += 1;
                                    if let Some(e) = nas_error {
                                        s.nas_failed += 1;
                                        s.message = Some(e.clone());
                                    }
                                }
                                Outcome::Copied => s.copied += 1,
                                Outcome::Failed(_) => s.failed += 1,
                                Outcome::Skipped => s.skipped += 1,
                            }
                            if let Err(e) = &saved {
                                s.message = Some(e.clone());
                            }
                        });
                        if matches!(outcome, Outcome::Made { .. } | Outcome::Copied) {
                            let mut show = last_show.lock().unwrap_or_else(|e| e.into_inner());
                            show.1 = true;
                            if show.0.elapsed() >= SHOW_EVERY {
                                *show = (Instant::now(), false);
                                sink.some_made();
                            }
                        }
                        let mut t = last_emit.lock().unwrap_or_else(|e| e.into_inner());
                        if t.elapsed() >= Duration::from_millis(200) {
                            *t = Instant::now();
                            sink.progress(&s);
                        }
                    }
                });
            }
        });
        if last_show.lock().map(|s| s.1).unwrap_or(false) {
            sink.some_made();
        }
        if let Ok(mut i) = self.inner.lock() {
            i.queue.clear();
        }
        let message = self.status().message;
        self.finish(sink.as_ref(), message);
    }
}

/// Utility QoS on macOS: the system runs the work when there is room, without slowing the window.
fn lower_priority() {
    #[cfg(target_os = "macos")]
    unsafe {
        libc::pthread_set_qos_class_self_np(libc::qos_class_t::QOS_CLASS_UTILITY, 0);
    }
}

/// Copies `from` to `to` through a temporary name of this process, so another Mac writing the
/// same preview at the same moment never sees a half-written file. Keeps the time of `from`.
fn copy_atomic(from: &Path, to: &Path) -> std::io::Result<()> {
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut name = to.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{}.tmp", std::process::id()));
    let tmp = to.with_file_name(name);
    let result = fs::copy(from, &tmp).and_then(|_| {
        if let Ok(t) = fs::metadata(from).and_then(|m| m.modified()) {
            let _ = fs::File::options().write(true).open(&tmp).and_then(|f| f.set_modified(t));
        }
        fs::rename(&tmp, to)
    });
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

fn make_one(job: &MakeJob, w: &Wanted) -> Outcome {
    let (out, thumb) = previews_plan::names(&job.previews, &w.path);
    // another Mac may have made it since the sync
    if let Some(nas) = &job.previews_nas {
        let (n_out, n_thumb) = previews_plan::names(nas, &w.path);
        if n_out.is_file() && n_thumb.is_file() && copy_atomic(&n_thumb, &thumb).and_then(|_| copy_atomic(&n_out, &out)).is_ok() {
            return Outcome::Copied;
        }
    }
    let local = job.library.join(&w.path);
    let src = if local.is_file() {
        local
    } else {
        match job.library_nas.as_ref().map(|n| n.join(&w.path)).filter(|p| p.is_file()) {
            Some(p) => p,
            None => return Outcome::Skipped,
        }
    };
    let kind = if w.kind == "video" { Kind::Video } else { Kind::Photo };
    if let Err(e) = preview::make(&src, kind, w.taken.as_deref(), &out, &thumb) {
        // the NAS went away while reading: not the file's fault, so try again next time
        return if src.is_file() { Outcome::Failed(e) } else { Outcome::Skipped };
    }
    let nas_error = job.previews_nas.as_ref().and_then(|nas| {
        let (n_out, n_thumb) = previews_plan::names(nas, &w.path);
        copy_atomic(&thumb, &n_thumb)
            .and_then(|_| copy_atomic(&out, &n_out))
            .err()
            .map(|e| format!("preview niet op de NAS gezet: {e}"))
    });
    Outcome::Made { nas_error }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{Connection, params};

    const VIDEO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/testsrc-rotated.mp4");

    struct Fixture {
        dir: tempfile::TempDir,
        job: MakeJob,
    }

    impl Fixture {
        fn path(&self, p: &str) -> PathBuf {
            self.dir.path().join(p)
        }
    }

    /// An index with these videos (path, hash), and the folders for this Mac and the NAS.
    fn fixture(items: &[(&str, &str)], nas: bool) -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let index = dir.path().join("index.sqlite");
        let conn = Connection::open(&index).unwrap();
        conn.execute_batch(
            "CREATE TABLE items (path TEXT PRIMARY KEY, folder TEXT, month TEXT, size INTEGER,
                hash TEXT, taken TEXT, taken_from TEXT, kind TEXT, source TEXT, sidecars TEXT,
                local INTEGER, preview INTEGER);",
        )
        .unwrap();
        for (path, hash) in items {
            conn.execute(
                "INSERT INTO items VALUES (?1, 'f', substr(?1, 1, 7), 1, ?2, '2026-07-30T16:58:12+02:00', NULL, 'video', NULL, '[]', 0, 0)",
                params![path, hash],
            )
            .unwrap();
        }
        let job = MakeJob {
            index,
            record: dir.path().join("rec.sqlite"),
            previews: dir.path().join("Previews"),
            library: dir.path().join("Library"),
            library_nas: nas.then(|| dir.path().join("nas/Library")),
            previews_nas: nas.then(|| dir.path().join("nas/Previews")),
            workers: 2,
        };
        Fixture { dir, job }
    }

    fn original(root: &Path, path: &str) {
        let p = root.join(path);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::copy(VIDEO, p).unwrap();
    }

    type OnProgress = Box<dyn Fn(&MakeState) + Send>;

    #[derive(Default)]
    struct Recorder {
        done: Mutex<Option<MakeState>>,
        shown: Mutex<u32>,
        on_progress: Mutex<Option<OnProgress>>,
    }

    impl MakeSink for Recorder {
        fn progress(&self, s: &MakeState) {
            if let Some(f) = self.on_progress.lock().unwrap().as_ref() {
                f(s);
            }
        }
        fn done(&self, s: &MakeState) {
            *self.done.lock().unwrap() = Some(s.clone());
        }
        fn some_made(&self) {
            *self.shown.lock().unwrap() += 1;
        }
    }

    fn run(f: &Fixture, runner: &MakeRunner) -> (MakeState, Arc<Recorder>) {
        let sink = Arc::new(Recorder::default());
        runner.start(f.job.clone(), sink.clone()).expect("started").join().unwrap();
        let state = sink.done.lock().unwrap().clone().expect("done");
        (state, sink)
    }

    fn both(root: &Path, path: &str) -> bool {
        let (o, t) = previews_plan::names(root, path);
        o.is_file() && t.is_file()
    }

    #[test]
    fn from_this_mac_and_from_the_nas_to_both() {
        let f = fixture(&[("2026/07/a/local.mp4", "h1"), ("2019/05/a/nas.mp4", "h2"), ("2019/05/a/nowhere.mp4", "h3")], true);
        original(&f.job.library, "2026/07/a/local.mp4");
        original(f.job.library_nas.as_ref().unwrap(), "2019/05/a/nas.mp4");
        let (s, sink) = run(&f, &MakeRunner::default());
        assert_eq!((s.total, s.done, s.made, s.skipped, s.failed, s.running), (3, 3, 2, 1, 0, false));
        for p in ["2026/07/a/local.mp4", "2019/05/a/nas.mp4"] {
            assert!(both(&f.job.previews, p), "{p} local");
            assert!(both(f.job.previews_nas.as_ref().unwrap(), p), "{p} on the NAS");
        }
        assert!(!both(&f.job.previews, "2019/05/a/nowhere.mp4"));
        assert_eq!(*sink.shown.lock().unwrap(), 1);
        // the record has both; the one without an original is tried again next time
        let again = run(&f, &MakeRunner::default()).0;
        assert_eq!((again.total, again.skipped), (1, 1));
        let left: Vec<_> = walk(&f.path("nas/Previews")).into_iter().filter(|p| p.ends_with(".tmp")).collect();
        assert!(left.is_empty(), "{left:?}");
    }

    fn walk(dir: &Path) -> Vec<String> {
        let mut out = Vec::new();
        for e in fs::read_dir(dir).into_iter().flatten().flatten() {
            if e.path().is_dir() {
                out.extend(walk(&e.path()));
            } else {
                out.push(e.path().to_string_lossy().into_owned());
            }
        }
        out
    }

    #[test]
    fn without_the_nas_only_what_is_local() {
        let f = fixture(&[("2026/07/a/local.mp4", "h1"), ("2019/05/a/nas.mp4", "h2")], false);
        original(&f.job.library, "2026/07/a/local.mp4");
        let (s, _) = run(&f, &MakeRunner::default());
        // the one without its original here isn't even counted
        assert_eq!((s.total, s.made, s.skipped), (1, 1, 0));
        assert!(both(&f.job.previews, "2026/07/a/local.mp4"));
    }

    #[test]
    fn a_preview_another_mac_put_on_the_nas_is_copied_not_made() {
        let f = fixture(&[("2019/05/a/x.mp4", "h")], true);
        original(f.job.library_nas.as_ref().unwrap(), "2019/05/a/x.mp4");
        let nas = f.job.previews_nas.clone().unwrap();
        for p in [previews_plan::names(&nas, "2019/05/a/x.mp4").0, previews_plan::names(&nas, "2019/05/a/x.mp4").1] {
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, b"from the other mac").unwrap();
        }
        let (s, _) = run(&f, &MakeRunner::default());
        assert_eq!((s.made, s.copied), (0, 1));
        assert_eq!(fs::read(previews_plan::names(&f.job.previews, "2019/05/a/x.mp4").0).unwrap(), b"from the other mac");
    }

    #[test]
    fn a_failure_is_recorded_and_not_tried_again() {
        let f = fixture(&[("2026/07/a/broken.mp4", "h")], false);
        let p = f.job.library.join("2026/07/a/broken.mp4");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, b"not a video").unwrap();
        let (s, _) = run(&f, &MakeRunner::default());
        assert_eq!(s.failed, 1);
        assert_eq!(run(&f, &MakeRunner::default()).0.total, 0);
    }

    #[test]
    fn pause_stops_and_resume_goes_on_where_it_stopped() {
        let items: Vec<(String, String)> = (0..40).map(|i| (format!("2026/07/a/v{i:02}.mp4"), format!("h{i}"))).collect();
        let refs: Vec<(&str, &str)> = items.iter().map(|(p, h)| (p.as_str(), h.as_str())).collect();
        let mut f = fixture(&refs, false);
        f.job.workers = 1;
        for (p, _) in &items {
            original(&f.job.library, p);
        }
        let runner = MakeRunner::default();
        let sink = Arc::new(Recorder::default());
        let r = runner.clone();
        // pause once some are made (progress comes every 200 ms)
        *sink.on_progress.lock().unwrap() = Some(Box::new(move |s| if s.made > 0 { r.stop(true) }));
        runner.start(f.job.clone(), sink.clone()).unwrap().join().unwrap();
        let first = sink.done.lock().unwrap().clone().unwrap();
        assert!(first.paused && first.made > 0 && first.made < 40, "{first:?}");
        // paused stays paused
        assert!(runner.start(f.job.clone(), Arc::new(Recorder::default())).is_none());
        runner.set_paused(false);
        let rest = run(&f, &runner).0;
        assert_eq!(first.made + rest.made, 40);
        assert_eq!(rest.total, 40 - first.made);
    }

    #[test]
    fn focus_puts_a_month_first_in_what_is_left() {
        let runner = MakeRunner::default();
        let w = |p: &str, m: &str| Wanted { path: p.into(), month: m.into(), kind: "photo".into(), hash: "h".into(), taken: None, state: State::Make };
        {
            let mut i = runner.inner.lock().unwrap();
            i.queue = vec![w("c", "2012/scans"), w("b", "2019/05"), w("a", "2026/07")];
        }
        runner.focus(Some("2019/05".into()));
        let next: Vec<_> = std::iter::from_fn(|| runner.inner.lock().unwrap().queue.pop()).map(|w| w.path).collect();
        assert_eq!(next, vec!["b", "a", "c"]);
    }
}
