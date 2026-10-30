//! Tauri-kern: leest de eigen index (F008) en de previews (datacontract sectie 4 en 7),
//! serveert previews via het `preview://`-protocol en opent originelen.

pub mod capture;
pub mod config;
mod connect;
pub mod db;
pub mod index;
pub mod filter;
pub mod hash;
pub mod lists;
pub mod log;
pub mod manifest;
pub mod marks;
pub mod mounts;
pub mod nas;
#[cfg(target_os = "macos")]
pub mod preview;
#[cfg(target_os = "macos")]
pub mod previews_job;
pub mod previews_plan;
pub mod push;
mod pull;
mod runner;
mod sync;
mod update;
mod window;

use config::{Device, Settings};
use db::{Db, Item, MonthSummary};
use percent_encoding::percent_decode_str;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tauri::http::{header, Response, StatusCode};
use tauri::{Emitter, Manager, State};

/// Where the app keeps its things (data contract 7): the local folders from the settings
/// (F006/S07), else in the proposed folder; `MEDIA_CONFIG`, `MEDIA_PREVIEWS`, `MEDIA_LIBRARY` and
/// `MEDIA_LISTS` win, as for the CLI.
#[derive(Clone)]
pub struct Paths {
    pub config: PathBuf,
    pub previews: PathBuf,
    pub library: PathBuf,
    /// The local copy of `Lists` (data contract 7).
    pub lists: PathBuf,
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_default()
}

fn env_path(name: &str, default: PathBuf) -> PathBuf {
    std::env::var_os(name).map(PathBuf::from).unwrap_or(default)
}

pub fn paths() -> Paths {
    let h = home();
    let config = env_path("MEDIA_CONFIG", h.join(".config").join("media"));
    // an installation from before F006/S06 used the CLI's places: keep them
    config::adopt_local(&config, &h);
    let local = config::load_settings(&config).0.local;
    let proposal = config::proposed_folder(&h);
    let pick = |set: &str, name: &str| if set.is_empty() { proposal.join(name) } else { PathBuf::from(set) };
    Paths {
        previews: env_path("MEDIA_PREVIEWS", pick(&local.previews, "Previews")),
        library: env_path("MEDIA_LIBRARY", pick(&local.library, "Library")),
        lists: env_path("MEDIA_LISTS", pick(&local.lists, "Lists")),
        config,
    }
}

impl Paths {
    /// The app's own index (F008); the CLI's `index.sqlite` is no longer read.
    fn index(&self) -> PathBuf {
        self.config.join(index::INDEX_FILE)
    }

    fn index_job(&self) -> index::IndexJob {
        index::IndexJob { library: self.library.clone(), previews: self.previews.clone(), target: self.index() }
    }
}

#[derive(serde::Serialize)]
pub struct Status {
    pub index: String,
    pub index_exists: bool,
    pub previews: String,
    pub previews_exists: bool,
    pub library: String,
    pub library_exists: bool,
}

#[tauri::command]
fn status(paths: State<Paths>) -> Status {
    Status {
        index: paths.index().display().to_string(),
        index_exists: paths.index().is_file(),
        previews: paths.previews.display().to_string(),
        previews_exists: paths.previews.is_dir(),
        library: paths.library.display().to_string(),
        library_exists: paths.library.is_dir(),
    }
}

#[derive(serde::Serialize)]
pub struct SettingsState {
    pub settings: Settings,
    pub error: Option<String>,
    pub path: String,
}

#[tauri::command]
fn settings(paths: State<Paths>) -> SettingsState {
    let (settings, error) = config::load_settings(&paths.config);
    SettingsState { settings, error, path: paths.config.join(config::SETTINGS_FILE).display().to_string() }
}

#[tauri::command]
fn save_settings(settings: Settings, paths: State<Paths>) -> Result<(), String> {
    config::save_settings(&paths.config, &settings)
}

#[derive(serde::Serialize)]
pub struct DeviceState {
    pub device: Option<Device>,
    pub error: Option<String>,
    pub path: String,
}

#[tauri::command]
fn device(paths: State<Paths>) -> DeviceState {
    let path = paths.config.join(config::DEVICE_FILE).display().to_string();
    match config::load_device(&paths.config) {
        Ok(device) => DeviceState { device, error: None, path },
        Err(e) => DeviceState { device: None, error: Some(e), path },
    }
}

#[tauri::command]
fn create_device(name: String, owner: String, paths: State<Paths>) -> Result<Device, String> {
    config::create_device(&paths.config, &name, &owner)
}

#[tauri::command]
fn nas_mounts(paths: State<Paths>) -> mounts::NasMounts {
    let (settings, _) = config::load_settings(&paths.config);
    mounts::find_shares(&mounts::system_mounts(), &settings)
}

#[derive(serde::Serialize)]
pub struct ConnectResult {
    pub attempts: Vec<connect::Attempt>,
    pub mounts: mounts::NasMounts,
}

/// Connects the shares that aren't mounted. Blocks while the macOS sign-in dialog is open, so it
/// runs on a blocking thread.
#[tauri::command]
async fn connect_nas(allow_ui: bool, paths: State<'_, Paths>) -> Result<ConnectResult, String> {
    let config = paths.config.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (settings, _) = config::load_settings(&config);
        if settings.nas_host.trim().is_empty() {
            return Err("nog geen NAS ingesteld: geef de naam van je NAS in de instellingen".to_string());
        }
        let before = mounts::find_shares(&mounts::system_mounts(), &settings);
        let todo = connect::to_connect(&before, &settings);
        #[cfg(target_os = "macos")]
        let attempts = connect::connect_all(&connect::NetFsMounter { allow_ui }, &settings.nas_host, &todo);
        #[cfg(not(target_os = "macos"))]
        let _ = allow_ui;
        #[cfg(not(target_os = "macos"))]
        let attempts = if todo.is_empty() {
            Vec::new()
        } else {
            return Err("verbinden met de NAS kan nog alleen op macOS".to_string());
        };
        Ok(ConnectResult { attempts, mounts: mounts::find_shares(&mounts::system_mounts(), &settings) })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// What the welcome proposes (F006/S06): the owner from the NAS user `Lists` is mounted with, a
/// device name from the Mac's name, and the devices the NAS knows already.
#[derive(serde::Serialize)]
pub struct Whoami {
    pub user: Option<String>,
    pub owner: String,
    pub device: String,
    pub computer: String,
    pub known: Vec<nas::Known>,
}

#[tauri::command]
fn nas_whoami(paths: State<Paths>) -> Whoami {
    let (settings, _) = config::load_settings(&paths.config);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    let lists = mounts::NasMounts::chosen(&found.lists);
    let user = lists.and_then(|m| m.user.clone());
    let known = lists.map(|m| nas::known_devices(Path::new(&m.mount_point))).unwrap_or_default();
    let owner = user.as_deref().map(nas::owner_from_user).unwrap_or_default();
    let computer = nas::computer_name();
    let device = nas::propose_device(&computer, &owner, &known);
    Whoami { user, owner, device, computer, known }
}

/// The local folders (F006/S07): where they are now, whether they were chosen or adopted, the
/// folder proposed on a new Mac, and the free space there.
#[derive(serde::Serialize)]
pub struct LocalFolders {
    pub library: String,
    pub previews: String,
    pub lists: String,
    pub chosen: bool,
    pub proposal: String,
}

#[tauri::command]
fn local_folders(paths: State<Paths>) -> LocalFolders {
    let chosen = config::load_settings(&paths.config).0.local.is_set();
    LocalFolders {
        library: paths.library.display().to_string(),
        previews: paths.previews.display().to_string(),
        lists: paths.lists.display().to_string(),
        chosen,
        proposal: config::proposed_folder(&home()).display().to_string(),
    }
}

/// The free space on the disk of `path`, or of the nearest folder above it that exists.
#[tauri::command]
fn free_space(path: String) -> Option<u64> {
    let mut p = PathBuf::from(path);
    while !p.exists() {
        p = p.parent()?.to_path_buf();
    }
    disk_free(&p)
}

#[cfg(target_os = "macos")]
fn disk_free(p: &Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    let c = std::ffi::CString::new(p.as_os_str().as_bytes()).ok()?;
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: a valid NUL-terminated path and a struct statvfs to fill
    (unsafe { libc::statvfs(c.as_ptr(), &mut st) } == 0).then(|| st.f_bavail as u64 * st.f_frsize as u64)
}

#[cfg(not(target_os = "macos"))]
fn disk_free(_: &Path) -> Option<u64> {
    None
}

#[tauri::command]
fn choose_local(folder: String, paths: State<Paths>) -> Result<config::Local, String> {
    config::choose_local(&paths.config, Path::new(&folder))
}

/// Starts the app again, so a new local folder is used everywhere (F006/S07).
#[tauri::command]
fn relaunch(app: tauri::AppHandle) {
    app.restart();
}

#[derive(serde::Serialize)]
pub struct NasCheck {
    pub library: nas::Access,
    pub previews: nas::Access,
    pub lists: nas::Access,
    pub exports: nas::Access,
    pub registration: nas::Registration,
}

/// Access per connected share, then registering this device once `Lists` is writable. On a
/// blocking thread, because a slow NAS can take a while to answer.
#[tauri::command]
async fn nas_check(paths: State<'_, Paths>) -> Result<NasCheck, String> {
    let config = paths.config.clone();
    tauri::async_runtime::spawn_blocking(move || check_nas(&config)).await.map_err(|e| e.to_string())
}

pub fn check_nas(config: &Path) -> NasCheck {
    let (settings, _) = config::load_settings(config);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    let device = config::load_device(config).ok().flatten();
    let check = |list: &[mounts::Mount], write: bool| match mounts::NasMounts::chosen(list) {
        None => nas::Access::default(),
        Some(m) => nas::check_share(
            Path::new(&m.mount_point),
            write.then(|| device.as_ref().map_or("nodevice", |d| d.name.as_str())),
        ),
    };
    let lists = check(&found.lists, true);
    let registration = match (&device, lists.writable) {
        (None, _) => nas::Registration::NotTried("dit toestel heeft nog geen naam".into()),
        (_, None) => nas::Registration::NotTried("Lijsten is niet verbonden".into()),
        (_, Some(false)) => nas::Registration::NotTried("Lijsten is niet schrijfbaar".into()),
        (Some(d), Some(true)) => {
            let dir = mounts::NasMounts::chosen(&found.lists).map(|m| m.mount_point.clone()).unwrap_or_default();
            nas::register(Path::new(&dir), &nas::entry_for(d), &|| {})
        }
    };
    NasCheck {
        library: check(&found.library, false),
        previews: check(&found.previews, false),
        exports: check(&found.exports, false),
        lists,
        registration,
    }
}

/// A warning or error to the log file and to the window's list of messages (F016/S01).
fn report(app: &tauri::AppHandle, level: log::Level, source: &str, text: &str) {
    let m = log::Message::new(level, source, text);
    log::write(&app.state::<Paths>().config, &m, log::MAX_BYTES);
    let _ = app.emit("message", &m);
}

/// The UI's own warnings and errors (a write that failed after its retry, an update), for the log.
#[tauri::command]
fn log_message(level: log::Level, source: String, text: String, app: tauri::AppHandle) {
    report(&app, level, &source, &text);
}

/// The filter kept on this Mac (F002/S03).
#[tauri::command]
fn get_filter(paths: State<Paths>) -> Option<serde_json::Value> {
    config::load_state(&paths.config).filter
}

#[tauri::command]
fn set_filter(filter: Option<serde_json::Value>, paths: State<Paths>) -> Result<(), String> {
    let mut state = config::load_state(&paths.config);
    state.filter = filter;
    config::save_state(&paths.config, &state)
}

/// Where the log is, for the overlay.
#[tauri::command]
fn log_path(paths: State<Paths>) -> String {
    paths.config.join(log::LOG_FILE).display().to_string()
}

/// Sends the runner's progress to the window as Tauri events (F007/S05).
struct EventSink(tauri::AppHandle);

impl runner::Sink for EventSink {
    fn progress(&self, state: &runner::SyncState) {
        let _ = self.0.emit("sync-progress", state);
    }
    fn done(&self, state: &runner::SyncState) {
        let _ = self.0.emit("sync-done", state);
        for f in &state.failed {
            report(&self.0, log::Level::Warn, "synchronisatie", &format!("overgeslagen: {f}"));
        }
        if state.failed_count as usize > state.failed.len() {
            let more = state.failed_count as usize - state.failed.len();
            report(&self.0, log::Level::Warn, "synchronisatie", &format!("en nog {more} bestanden overgeslagen"));
        }
        if let (runner::Phase::Failed, Some(m)) = (state.phase, &state.message) {
            report(&self.0, log::Level::Error, "synchronisatie", m);
        }
        // What came in only shows in the grid once the index has it (F008/S02).
        if state.manifests_copied > 0 || state.previews_copied > 0 {
            self.rebuild_index();
        } else {
            start_making(&self.0);
        }
    }
    fn year_copied(&self) {
        // the copy goes on meanwhile; the build runs on its own thread (F008/S03)
        self.rebuild_index();
    }
    fn manifests_in(&self) {
        // the months show before the previews are counted, which takes minutes on a new Mac
        self.rebuild_index();
    }
}

impl EventSink {
    fn rebuild_index(&self) {
        let job = self.0.state::<Paths>().index_job();
        self.0.state::<index::IndexRunner>().request(job, Arc::new(EventSink(self.0.clone())));
    }
}

impl index::IndexSink for EventSink {
    fn done(&self, state: &index::IndexState) {
        let _ = self.0.emit("index-done", state);
        if let Some(e) = &state.error {
            report(&self.0, log::Level::Error, "index", e);
        }
        for b in state.outcome.iter().flat_map(|o| &o.broken) {
            report(&self.0, log::Level::Warn, "index", &format!("manifest onleesbaar: {b}"));
        }
        // the index is current now: make what is missing (F012/S03)
        start_making(&self.0);
    }
}

/// The job that makes missing previews, for the current mounts (F012/S03).
#[cfg(target_os = "macos")]
fn make_job(paths: &Paths) -> previews_job::MakeJob {
    let (settings, _) = config::load_settings(&paths.config);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    let chosen = |list: &[mounts::Mount]| mounts::NasMounts::chosen(list).map(|m| PathBuf::from(&m.mount_point));
    previews_job::MakeJob {
        index: paths.index(),
        record: paths.config.join(previews_plan::RECORD_FILE),
        previews: paths.previews.clone(),
        library: paths.library.clone(),
        library_nas: chosen(&found.library),
        previews_nas: chosen(&found.previews),
        workers: previews_job::default_workers(),
    }
}

/// Start making previews, unless a sync or an index build runs, an update installs, there is no
/// index yet, or it is paused or running already.
fn start_making(app: &tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    {
        // without an index (a new Mac before its first sync) there is nothing to make yet
        let busy = app.state::<runner::SyncRunner>().status().running
            || app.state::<index::IndexRunner>().status().running
            || app.state::<update::UpdateState>().installing()
            || !app.state::<Paths>().index().exists();
        if !busy {
            let job = make_job(&app.state::<Paths>());
            app.state::<previews_job::MakeRunner>().start(job, Arc::new(EventSink(app.clone())));
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = app;
}

#[cfg(target_os = "macos")]
impl previews_job::MakeSink for EventSink {
    fn progress(&self, state: &previews_job::MakeState) {
        let _ = self.0.emit("previews-progress", state);
    }
    fn done(&self, state: &previews_job::MakeState) {
        let _ = self.0.emit("previews-done", state);
        if state.failed > 0 {
            report(&self.0, log::Level::Warn, "previews", &format!("{} previews konden niet gemaakt worden (zie Instellingen → Previews)", state.failed));
        }
        if let Some(m) = &state.message {
            report(&self.0, log::Level::Warn, "previews", m);
        }
        // the counts in the month list come from a build
        if state.made + state.copied > 0 {
            self.rebuild_index();
        }
    }
    fn made(&self, done: &[previews_job::Done]) {
        // the flag in the index, then the grid (F012/S04); a missing index is built at the end anyway
        let paths: Vec<&str> = done.iter().map(|d| d.path.as_str()).collect();
        if index::mark_previews(&self.0.state::<Paths>().index(), &paths).is_ok() {
            let _ = self.0.emit("previews-made", done);
        }
    }
}

#[derive(serde::Serialize)]
pub struct PreviewsOverview {
    /// Photos and videos without a preview on this Mac.
    pub missing: i64,
    pub failed: Vec<previews_plan::Failure>,
}

/// For the settings: how many previews are missing and what failed (F012/S04).
#[tauri::command]
fn previews_overview(paths: State<Paths>) -> Result<PreviewsOverview, String> {
    let missing = Db::open(&paths.index())?.missing_previews()?;
    let failed = previews_plan::Record::open(&paths.config.join(previews_plan::RECORD_FILE))?.failures()?;
    Ok(PreviewsOverview { missing, failed })
}

/// Try the failures again: forget them and start making.
#[tauri::command]
fn previews_retry(app: tauri::AppHandle, paths: State<Paths>) -> Result<usize, String> {
    let n = previews_plan::Record::open(&paths.config.join(previews_plan::RECORD_FILE))?.clear_failures()?;
    start_making(&app);
    Ok(n)
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn previews_status(runner: State<previews_job::MakeRunner>) -> previews_job::MakeState {
    runner.status()
}

/// Pause making previews; it stays paused after a restart until resumed.
#[cfg(target_os = "macos")]
#[tauri::command]
fn previews_pause(paths: State<Paths>, runner: State<previews_job::MakeRunner>) -> Result<(), String> {
    runner.stop(true);
    let mut state = config::load_state(&paths.config);
    state.previews_paused = true;
    config::save_state(&paths.config, &state)
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn previews_resume(app: tauri::AppHandle, paths: State<Paths>, runner: State<previews_job::MakeRunner>) -> Result<(), String> {
    let mut state = config::load_state(&paths.config);
    state.previews_paused = false;
    config::save_state(&paths.config, &state)?;
    runner.set_paused(false);
    start_making(&app);
    Ok(())
}

/// The month at the top of the grid goes first.
#[cfg(target_os = "macos")]
#[tauri::command]
fn previews_focus(month: Option<String>, runner: State<previews_job::MakeRunner>) {
    runner.focus(month);
}

/// Rebuild the index in the background; with `only_if_stale`, only when a manifest changed since
/// the last build. Returns whether a build was asked for. The end arrives as `index-done`.
#[tauri::command]
fn index_rebuild(only_if_stale: bool, app: tauri::AppHandle, paths: State<Paths>, runner: State<index::IndexRunner>) -> bool {
    let job = paths.index_job();
    if only_if_stale && !runner.stale(&job) {
        return false;
    }
    runner.request(job, Arc::new(EventSink(app)));
    true
}

#[tauri::command]
fn index_status(runner: State<index::IndexRunner>) -> index::IndexState {
    runner.status()
}

/// The two sync jobs for the current settings and mounts.
fn sync_jobs(paths: &Paths) -> runner::Jobs {
    let (settings, _) = config::load_settings(&paths.config);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    runner::Jobs {
        manifests: pull::manifests_job(&settings, &found, &paths.library),
        lists: pull::lists_job(&settings, &found, &paths.lists),
        previews: pull::previews_job(&settings, &found, &paths.previews),
        config_dir: paths.config.clone(),
        free: pull::free_bytes,
    }
}

/// Start a sync in the background: `kind` is "all" (manifests, then previews) or "previews"
/// (after the confirmation dialog). Progress arrives as `sync-progress`, the end as `sync-done`.
/// `at_launch` is the window's own sync at start: only once per process, so reloading the window
/// (⌘R, or the dev server) doesn't sync again and interrupt the previews being made.
#[tauri::command]
fn sync_start(
    kind: String,
    confirmed: bool,
    at_launch: Option<bool>,
    app: tauri::AppHandle,
    paths: State<Paths>,
    runner: State<runner::SyncRunner>,
    update: State<update::UpdateState>,
) -> Result<(), String> {
    if update.installing() {
        return Err("de app wordt bijgewerkt en herstart zo".into());
    }
    if at_launch == Some(true) && !runner.first_launch() {
        return Ok(());
    }
    // no previews are made during a sync; they start again after it (F012/S03)
    #[cfg(target_os = "macos")]
    app.state::<previews_job::MakeRunner>().stop(false);
    let kind = match kind.as_str() {
        "all" => runner::Kind::All,
        "previews" => runner::Kind::Previews,
        other => return Err(format!("onbekende synchronisatie: {other}")),
    };
    runner.start(kind, confirmed, sync_jobs(&paths), Arc::new(EventSink(app))).map(|_| ())
}

#[tauri::command]
fn sync_cancel(runner: State<runner::SyncRunner>) {
    runner.cancel();
}

#[tauri::command]
fn sync_status(runner: State<runner::SyncRunner>) -> runner::SyncState {
    runner.status()
}

/// The lists from the `Lists` share, or from the local copy when it isn't connected (F003/S01).
fn load_lists(paths: &Paths) -> (lists::ListsState, std::collections::BTreeMap<String, lists::List>) {
    let (settings, _) = config::load_settings(&paths.config);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    let nas = mounts::NasMounts::chosen(&found.lists).map(|m| PathBuf::from(&m.mount_point));
    lists::load(nas.as_deref(), &paths.lists)
}

/// A fingerprint of the event files where the lists are read from, to notice the other Mac's
/// changes (F003/S04); as text, since a u64 doesn't fit a JavaScript number. It names the source
/// too: when the `Lists` share connects after the app started, the files are the same as the
/// local copy's, but the lists must be read again from the share to become changeable.
#[tauri::command]
async fn lists_stamp(paths: State<'_, Paths>) -> Result<Option<String>, String> {
    let paths = paths.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (settings, _) = config::load_settings(&paths.config);
        let found = mounts::find_shares(&mounts::system_mounts(), &settings);
        let nas = mounts::NasMounts::chosen(&found.lists).map(|m| PathBuf::from(&m.mount_point));
        let source = if nas.is_some() { "nas" } else { "local" };
        let dir = nas.unwrap_or(paths.lists.clone());
        Some(format!("{source}:{}", lists::stamp(&dir).map_or("-".into(), |s| s.to_string())))
    })
    .await
    .map_err(|e| e.to_string())
}

/// The `Lists` share when it is connected.
fn lists_share(paths: &Paths) -> Option<PathBuf> {
    let (settings, _) = config::load_settings(&paths.config);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    mounts::NasMounts::chosen(&found.lists).map(|m| PathBuf::from(&m.mount_point))
}

/// Stars and hearts by hash, from the share or the local copy (F013/S01).
#[tauri::command]
async fn marks_state(paths: State<'_, Paths>) -> Result<marks::MarksState, String> {
    let paths = paths.inner().clone();
    tauri::async_runtime::spawn_blocking(move || marks::load(lists_share(&paths).as_deref(), &paths.lists))
        .await
        .map_err(|e| e.to_string())
}

/// Writes a change to stars or hearts on the share and answers with the marks as they are now.
fn change_marks(paths: &Paths, change: marks::Change) -> Result<marks::MarksState, String> {
    let nas = lists_share(paths).ok_or("Lijsten is niet verbonden: sterren en hartjes wijzigen kan alleen met de NAS")?;
    let device = config::load_device(&paths.config)?.ok_or("dit toestel heeft nog geen naam")?;
    marks::append(&nas, &device.name, &device.owner, &change)?;
    Ok(marks::load(Some(&nas), &paths.lists))
}

#[tauri::command]
async fn marks_rate(photos: Vec<String>, stars: u8, paths: State<'_, Paths>) -> Result<marks::MarksState, String> {
    let paths = paths.inner().clone();
    on_blocking(move || change_marks(&paths, marks::Change::Rate { photos, stars })).await
}

#[tauri::command]
async fn marks_hide(photos: Vec<String>, hidden: bool, paths: State<'_, Paths>) -> Result<marks::MarksState, String> {
    let paths = paths.inner().clone();
    on_blocking(move || change_marks(&paths, marks::Change::Hide { photos, hidden })).await
}

/// Puts photos in a stack (F015/S01): the given one, or a new one made by this device.
#[tauri::command]
async fn marks_stack(photos: Vec<String>, stack: Option<String>, paths: State<'_, Paths>) -> Result<marks::MarksState, String> {
    let paths = paths.inner().clone();
    on_blocking(move || {
        let stack = match stack {
            Some(s) => s,
            None => {
                let device = config::load_device(&paths.config)?.ok_or("dit toestel heeft nog geen naam")?;
                marks::new_stack_id(&device.name)
            }
        };
        change_marks(&paths, marks::Change::Stack { photos, stack })
    })
    .await
}

#[tauri::command]
async fn marks_color(stack: String, color: String, paths: State<'_, Paths>) -> Result<marks::MarksState, String> {
    let paths = paths.inner().clone();
    on_blocking(move || change_marks(&paths, marks::Change::Color { stack, color })).await
}

#[tauri::command]
async fn marks_unstack(photos: Vec<String>, paths: State<'_, Paths>) -> Result<marks::MarksState, String> {
    let paths = paths.inner().clone();
    on_blocking(move || change_marks(&paths, marks::Change::Unstack { photos })).await
}

#[tauri::command]
async fn marks_heart(photos: Vec<String>, on: bool, paths: State<'_, Paths>) -> Result<marks::MarksState, String> {
    let paths = paths.inner().clone();
    on_blocking(move || change_marks(&paths, marks::Change::Heart { photos, on })).await
}

/// All lists, without their photos. On a blocking thread: the share may be slow to answer.
#[tauri::command]
async fn lists_state(paths: State<'_, Paths>) -> Result<lists::ListsState, String> {
    let paths = paths.inner().clone();
    tauri::async_runtime::spawn_blocking(move || load_lists(&paths).0).await.map_err(|e| e.to_string())
}

/// The photos of one list, by hash.
#[tauri::command]
async fn list_photos(id: String, paths: State<'_, Paths>) -> Result<Vec<lists::Member>, String> {
    let paths = paths.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (state, mut all) = load_lists(&paths);
        if let Some(p) = state.problem {
            return Err(p);
        }
        all.remove(&id).map(|l| l.photos.into_values().collect()).ok_or_else(|| format!("onbekende lijst: {id}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Lists whose summary is to be written (F003/S02).
#[derive(Default)]
pub struct Summaries(std::sync::Mutex<std::collections::HashSet<String>>);

/// How long after a change the summaries are written, so a run of adds writes them once.
const SUMMARY_DELAY: std::time::Duration = std::time::Duration::from_secs(3);

fn write_summaries(app: &tauri::AppHandle) {
    let ids: Vec<String> = app.state::<Summaries>().0.lock().map(|mut p| p.drain().collect()).unwrap_or_default();
    if ids.is_empty() {
        return;
    }
    let paths = app.state::<Paths>().inner().clone();
    let (settings, _) = config::load_settings(&paths.config);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    let (Some(nas), Ok(Some(device))) = (mounts::NasMounts::chosen(&found.lists), config::load_device(&paths.config)) else {
        return;
    };
    let nas = PathBuf::from(&nas.mount_point);
    let Ok(read) = lists::read(&nas) else { return };
    let merged = lists::merge(read.events);
    let index = rusqlite::Connection::open_with_flags(paths.index(), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).ok();
    let path_of = |hash: &str| -> Option<String> {
        index.as_ref()?.query_row(
            "SELECT path FROM items WHERE hash = ?1 ORDER BY folder DESC LIMIT 1", [hash], |r| r.get(0),
        ).ok()
    };
    for list in ids.iter().filter_map(|id| merged.get(id)) {
        if let Err(e) = lists::write_summary(&nas, &device.name, list, &path_of) {
            eprintln!("{e}");
        }
    }
}

#[derive(serde::Serialize)]
pub struct ListChanged {
    /// The list that changed (for a new list, its new id).
    pub id: String,
    pub state: lists::ListsState,
}

/// Writes one change on the `Lists` share and answers with the lists as they are now. Without the
/// share, or without a device, it refuses: changes are only written on the NAS (data contract 5.2).
fn change_list(app: &tauri::AppHandle, make: impl FnOnce(&std::collections::BTreeMap<String, lists::List>) -> lists::Change) -> Result<ListChanged, String> {
    let paths = app.state::<Paths>().inner().clone();
    let (settings, _) = config::load_settings(&paths.config);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    let nas = mounts::NasMounts::chosen(&found.lists)
        .map(|m| PathBuf::from(&m.mount_point))
        .ok_or("Lijsten is niet verbonden: lijsten wijzigen kan alleen met de NAS")?;
    let device = config::load_device(&paths.config)?.ok_or("dit toestel heeft nog geen naam")?;
    let merged = lists::merge(lists::read(&nas)?.events);
    let change = make(&merged);
    lists::append(&nas, &device.name, &device.owner, &change, &merged)?;
    let id = match &change {
        lists::Change::Create { list, .. }
        | lists::Change::Add { list, .. }
        | lists::Change::Remove { list, .. }
        | lists::Change::Rename { list, .. }
        | lists::Change::Archive { list, .. } => list.clone(),
    };
    if let Ok(mut p) = app.state::<Summaries>().0.lock() {
        p.insert(id.clone());
    }
    let later = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(SUMMARY_DELAY);
        write_summaries(&later);
    });
    Ok(ListChanged { id, state: lists::load(Some(&nas), &paths.lists).0 })
}

#[derive(serde::Serialize)]
pub struct ListItem {
    /// The month the grid groups it under.
    pub month: String,
    #[serde(flatten)]
    pub item: Item,
}

#[derive(serde::Serialize)]
pub struct ListItems {
    pub items: Vec<ListItem>,
    /// Photos in the list whose hash isn't in this Mac's index (not synced yet).
    pub missing: usize,
}

/// The photos of a list as grid items, from the index (F003/S04).
#[tauri::command]
async fn list_items(id: String, paths: State<'_, Paths>) -> Result<ListItems, String> {
    let paths = paths.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (state, mut all) = load_lists(&paths);
        if let Some(p) = state.problem {
            return Err(p);
        }
        let list = all.remove(&id).ok_or_else(|| format!("onbekende lijst: {id}"))?;
        let hashes: Vec<String> = list.photos.into_keys().collect();
        let rows = Db::open(&paths.index())?.items_by_hash(&hashes)?;
        let found: std::collections::HashSet<&str> = rows.iter().map(|(_, i)| i.hash.as_str()).collect();
        let missing = hashes.iter().filter(|h| !found.contains(h.as_str())).count();
        Ok(ListItems { items: rows.into_iter().map(|(month, item)| ListItem { month, item }).collect(), missing })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Runs a list change on a blocking thread: the share may be slow.
async fn on_blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn list_create(name: String, year: i32, app: tauri::AppHandle) -> Result<ListChanged, String> {
    on_blocking(move || {
        change_list(&app, |merged| {
            let list = lists::new_id(year, &name, &|id| merged.contains_key(id));
            lists::Change::Create { list, name: name.trim().to_string() }
        })
    })
    .await
}

#[derive(serde::Deserialize)]
pub struct PhotoRef {
    pub photo: String,
    pub path: Option<String>,
}

#[tauri::command]
async fn list_add(id: String, photos: Vec<PhotoRef>, app: tauri::AppHandle) -> Result<ListChanged, String> {
    on_blocking(move || {
        change_list(&app, |_| lists::Change::Add { list: id, photos: photos.into_iter().map(|p| (p.photo, p.path)).collect() })
    })
    .await
}

#[tauri::command]
async fn list_remove(id: String, photos: Vec<String>, app: tauri::AppHandle) -> Result<ListChanged, String> {
    on_blocking(move || change_list(&app, |_| lists::Change::Remove { list: id, photos })).await
}

#[tauri::command]
async fn list_rename(id: String, name: String, app: tauri::AppHandle) -> Result<ListChanged, String> {
    on_blocking(move || change_list(&app, |_| lists::Change::Rename { list: id, name: name.trim().to_string() })).await
}

#[tauri::command]
async fn list_archive(id: String, archived: bool, app: tauri::AppHandle) -> Result<ListChanged, String> {
    on_blocking(move || change_list(&app, |_| lists::Change::Archive { list: id, archived })).await
}

/// The photos with these hashes as grid items with their month: the members of the stacks, which
/// may lie in other months than the one their stack stands in (F015/S02).
#[tauri::command]
async fn items_by_hash(hashes: Vec<String>, paths: State<'_, Paths>) -> Result<Vec<ListItem>, String> {
    let paths = paths.inner().clone();
    on_blocking(move || {
        let rows = Db::open(&paths.index())?.items_by_hash(&hashes)?;
        Ok(rows.into_iter().map(|(month, item)| ListItem { month, item }).collect())
    })
    .await
}

/// How many of these photos (by hash) each month has: the hidden ones (F014/S02).
#[tauri::command]
async fn count_per_month(hashes: Vec<String>, paths: State<'_, Paths>) -> Result<std::collections::HashMap<String, i64>, String> {
    let paths = paths.inner().clone();
    on_blocking(move || Db::open(&paths.index())?.count_per_month(&hashes.into_iter().collect())).await
}

/// The months with the number of matches, for a filter (F002/S02).
#[tauri::command]
async fn months_filtered(filter: filter::Filter, paths: State<'_, Paths>) -> Result<Vec<(String, i64)>, String> {
    let paths = paths.inner().clone();
    on_blocking(move || Db::open(&paths.index())?.filtered_months(&filter)).await
}

/// A month's items that match a filter (F002/S02).
#[tauri::command]
async fn items_filtered(month: String, filter: filter::Filter, paths: State<'_, Paths>) -> Result<Vec<Item>, String> {
    let paths = paths.inner().clone();
    on_blocking(move || Db::open(&paths.index())?.filtered_items(&month, &filter)).await
}

/// The cameras with how many photos and videos each and when last used, "" for unknown (F002/S04).
#[tauri::command]
async fn cameras(paths: State<'_, Paths>) -> Result<Vec<(String, i64, Option<String>)>, String> {
    let paths = paths.inner().clone();
    on_blocking(move || Db::open(&paths.index())?.camera_counts()).await
}

#[tauri::command]
fn months(paths: State<Paths>) -> Result<Vec<MonthSummary>, String> {
    Db::open(&paths.index())?.months()
}

#[tauri::command]
fn items(month: String, paths: State<Paths>) -> Result<Vec<Item>, String> {
    Db::open(&paths.index())?.items(&month)
}

/// Een relatief contractpad (`2026/07/EOSR/.../X.CR3`) mag nooit uit de boom wijzen.
fn safe_join(root: &Path, rel: &str) -> Option<PathBuf> {
    let rel = Path::new(rel);
    if rel.is_absolute() || rel.components().any(|c| !matches!(c, Component::Normal(_))) {
        return None;
    }
    Some(root.join(rel))
}

fn original(paths: &Paths, rel: &str) -> Result<PathBuf, String> {
    let p = safe_join(&paths.library, rel).ok_or("ongeldig pad")?;
    if !p.is_file() {
        return Err("het origineel staat niet op deze laptop (alleen op de NAS)".into());
    }
    Ok(p)
}

/// Toon het origineel in de Finder / Verkenner.
#[tauri::command]
fn reveal(path: String, paths: State<Paths>) -> Result<(), String> {
    let p = original(&paths, &path)?;
    tauri_plugin_opener::reveal_item_in_dir(p).map_err(|e| e.to_string())
}

/// Open het origineel met het standaardprogramma (Voorvertoning, Lightroom, …).
#[tauri::command]
fn open_original(path: String, paths: State<Paths>) -> Result<(), String> {
    let p = original(&paths, &path)?;
    tauri_plugin_opener::open_path(p, None::<&str>).map_err(|e| e.to_string())
}

/// `preview://localhost/<pad>` → bestand onder de Previews-map. Het pad komt percent-gecodeerd binnen.
fn preview_response(previews: &Path, uri_path: &str) -> Response<Vec<u8>> {
    let decoded = percent_decode_str(uri_path.trim_start_matches('/')).decode_utf8_lossy();
    let file = safe_join(previews, &decoded).filter(|p| p.extension().is_some_and(|e| e == "jpg"));
    match file.and_then(|p| std::fs::read(p).ok()) {
        Some(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/jpeg")
            .header(header::CACHE_CONTROL, "max-age=86400")
            .body(bytes)
            .unwrap(),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(b"geen preview".to_vec())
            .unwrap(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let paths = paths();
    let previews_root = paths.previews.clone();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(window::flags())
                .skip_initial_state("main")
                .build(),
        )
        .on_window_event(window::on_event)
        .setup(|app| {
            if let Some(main) = app.get_webview_window("main") {
                window::restore(&main);
            }
            // an installation from before F006/S06 had the NAS built in: keep it
            config::adopt_host(&app.state::<Paths>().config, &mounts::system_mounts());
            #[cfg(target_os = "macos")]
            {
                let paused = config::load_state(&app.state::<Paths>().config).previews_paused;
                app.state::<previews_job::MakeRunner>().set_paused(paused);
            }
            Ok(())
        })
        .manage(paths)
        .manage(runner::SyncRunner::default())
        .manage(index::IndexRunner::default())
        .manage(update::UpdateState::default())
        .manage(Summaries::default())
        .manage(previews_job::MakeRunner::default())
        .register_uri_scheme_protocol("preview", move |_ctx, request| {
            preview_response(&previews_root, request.uri().path())
        })
        .invoke_handler(tauri::generate_handler![
            status, months, items, reveal, open_original, settings, save_settings, device, create_device,
            nas_mounts, connect_nas, nas_check, sync_start, sync_cancel, sync_status, index_rebuild, index_status,
            update::update_check, update::update_install,
            log_message, log_path, get_filter, set_filter,
            nas_whoami, local_folders, free_space, choose_local, relaunch, lists_state, lists_stamp, list_photos, marks_state, marks_rate, marks_heart, marks_hide, marks_stack, marks_unstack, marks_color, items_by_hash, months_filtered, items_filtered, count_per_month, cameras, list_create, list_add, list_remove, list_rename, list_archive, list_items,
            previews_status, previews_pause, previews_resume, previews_focus, previews_overview, previews_retry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_join_rejects_escapes() {
        let root = Path::new("/r");
        assert_eq!(safe_join(root, "2026/07/a.jpg"), Some(PathBuf::from("/r/2026/07/a.jpg")));
        assert_eq!(safe_join(root, "../etc/passwd"), None);
        assert_eq!(safe_join(root, "/etc/passwd"), None);
        assert_eq!(safe_join(root, "2026/../../x"), None);
    }

    #[test]
    fn preview_protocol_serves_jpg_and_refuses_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("2026/07")).unwrap();
        std::fs::write(dir.path().join("2026/07/IMG (1).JPG.t.jpg"), b"jpegdata").unwrap();
        std::fs::write(dir.path().join("geheim.txt"), b"nee").unwrap();
        let ok = preview_response(dir.path(), "/2026%2F07%2FIMG%20(1).JPG.t.jpg");
        assert_eq!(ok.status(), StatusCode::OK);
        assert_eq!(ok.body(), b"jpegdata");
        assert_eq!(ok.headers()[header::CONTENT_TYPE], "image/jpeg");
        assert_eq!(preview_response(dir.path(), "/2026/07/IMG%20(1).JPG.t.jpg").status(), StatusCode::OK);
        assert_eq!(preview_response(dir.path(), "/geheim.txt").status(), StatusCode::NOT_FOUND);
        assert_eq!(preview_response(dir.path(), "/..%2F..%2Fetc%2Fpasswd").status(), StatusCode::NOT_FOUND);
        assert_eq!(preview_response(dir.path(), "/2026/07/ontbreekt.jpg").status(), StatusCode::NOT_FOUND);
    }
}
