//! Tauri-kern: leest de eigen index (F008) en de previews (datacontract sectie 4 en 7),
//! serveert previews via het `preview://`-protocol en opent originelen.

pub mod capture;
pub mod config;
pub mod export;
pub mod fetch;
mod connect;
pub mod db;
pub mod index;
pub mod filter;
pub mod hash;
pub mod import_job;
pub mod import_plan;
pub mod layout;
pub mod lightroom;
pub mod lists;
pub mod lrplugin;
pub mod log;
pub mod manifest;
pub mod marks;
pub mod mounts;
pub mod nas;
pub mod outbox;
#[cfg(target_os = "macos")]
pub mod preview;
#[cfg(target_os = "macos")]
pub mod previews_job;
pub mod previews_plan;
pub mod push;
pub mod rename;
pub mod sources;
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

/// What a move would find with the file in `share`, without using it (F019/S04).
#[tauri::command]
async fn layout_dry_check(share: String, paths: State<'_, Paths>) -> Result<layout::DryCheck, String> {
    let config = paths.config.clone();
    on_blocking(move || Ok(layout::dry_check(&mounts::system_mounts(), &config::load_settings(&config).0.nas_host, &share))).await
}

/// What the welcome finds in the home share (F019/S02).
#[tauri::command]
async fn layout_home(paths: State<'_, Paths>) -> Result<layout::Home, String> {
    let config = paths.config.clone();
    on_blocking(move || {
        let (settings, _) = config::load_settings(&config);
        Ok(layout::home_status(&mounts::system_mounts(), &settings.nas_host, settings.home()))
    })
    .await
}

/// Sets up a library in the home share (F019/S02): in one share, or `layout` pointing at folders
/// that exist; then reads it, so the shares it names are connected next.
#[tauri::command]
async fn layout_set_up(name: String, layout: Option<layout::NasLayout>, paths: State<'_, Paths>) -> Result<Vec<String>, String> {
    let config = paths.config.clone();
    on_blocking(move || {
        let (settings, _) = config::load_settings(&config);
        let (host, home) = (settings.nas_host.clone(), settings.home().to_string());
        let raw = mounts::system_mounts();
        let list = mounts::mounts_of(&raw, &host, &home);
        let m = mounts::NasMounts::chosen(&list).ok_or_else(|| format!("de gedeelde map '{home}' is niet verbonden"))?;
        let mut l = layout.unwrap_or_else(|| layout::NasLayout::in_one_share(&home, ""));
        l.name = name.trim().to_string();
        let made = layout::set_up(Path::new(&m.mount_point), &home, &l)?;
        layout::refresh(&config, &raw, &host, &home)?;
        Ok(made)
    })
    .await
}

/// The layout in use (F019/S03): the NAS's file, its copy, or the Mac's share names.
#[tauri::command]
fn nas_layout(paths: State<Paths>) -> layout::NasLayout {
    config::load_settings(&paths.config).0.effective_layout()
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
        #[cfg(target_os = "macos")]
        let connect_these = |host: &str, list: &[(connect::ShareKey, String)]| connect::connect_all(&connect::NetFsMounter { allow_ui }, host, list);
        #[cfg(not(target_os = "macos"))]
        // connecting is macOS only for now (NetFS); elsewhere the shares are mounted by hand
        let connect_these = |_: &str, _: &[(connect::ShareKey, String)]| -> Vec<connect::Attempt> {
            let _ = allow_ui;
            Vec::new()
        };
        let host = settings.nas_host.clone();
        let home = settings.home().to_string();
        // first the home share, whose file says where the rest is (F019/S01)
        let mut attempts = Vec::new();
        if mounts::NasMounts::chosen(&mounts::mounts_of(&mounts::system_mounts(), &host, &home)).is_none() {
            attempts.extend(connect_these(&host, &[(connect::ShareKey::Lists, home.clone())]));
        }
        let _ = layout::refresh(&config, &mounts::system_mounts(), &host, &home);
        let (settings, _) = config::load_settings(&config);
        let before = mounts::find_shares(&mounts::system_mounts(), &settings);
        let todo = connect::to_connect(&before, &settings);
        attempts.extend(connect_these(&host, &todo));
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

/// What the import can take from on this Mac (F018/S03), with names and layouts from the library.
#[tauri::command]
async fn import_sources(paths: State<'_, Paths>) -> Result<Vec<sources::Found>, String> {
    let paths = paths.inner().clone();
    on_blocking(move || {
        let import = config::load_settings(&paths.config).0.import;
        let db = Db::open(&paths.index()).ok();
        let layouts = db.as_ref().and_then(|d| d.source_paths().ok()).map(|p| sources::layouts(&p)).unwrap_or_default();
        let of_camera = |c: &str| db.as_ref().and_then(|d| d.sources_of_camera(c).ok()).unwrap_or_default();
        Ok(sources::discover(&import, Path::new("/Volumes"), &layouts, &of_camera))
    })
    .await
}

/// The source at `path` found again (S03) and planned (S04), with its settings.
fn plan_source(paths: &Paths, path: &str, progress: &mut dyn FnMut(&import_plan::Progress)) -> Result<(import_plan::SourcePlan, config::SourceSettings), String> {
    let import = config::load_settings(&paths.config).0.import;
    let db = Db::open(&paths.index())?;
    let layouts = sources::layouts(&db.source_paths()?);
    let of_camera = |c: &str| db.sources_of_camera(c).unwrap_or_default();
    let found = sources::discover(&import, Path::new("/Volumes"), &layouts, &of_camera);
    let f = found.iter().find(|f| f.path == path).ok_or_else(|| format!("{path}: geen bron (meer)"))?;
    if let Some(p) = &f.problem {
        return Err(p.clone());
    }
    let folder = Path::new(&f.path).file_name().unwrap_or_default().to_string_lossy().into_owned();
    let set = import.sources.get(&folder).cloned().unwrap_or_default();
    let known = db.path_hashes()?;
    let cache = import_plan::HashCache::open(&paths.config.join("familylightbox-import.sqlite"))?;
    if f.kind == "synced" {
        // copied only, the month from its folders, what was taken recorded (S06)
        let layout_of = |name: &str| layouts.get(name).copied().unwrap_or(sources::NEW);
        let synced = import.synced.iter().find(|x| x.path == f.path);
        let to = import_plan::SyncedTo {
            photos: (f.name.clone(), layout_of(&f.name)),
            videos: (f.videos.clone(), layout_of(&f.videos)),
            skip_live: synced.is_some_and(|x| x.skip_live),
            from_month: match synced.map(|x| x.from.trim()).filter(|x| !x.is_empty()) {
                Some(from) => Some(from.to_string()),
                None => db.first_month(&f.name)?,
            },
        };
        let record = import_plan::TakenRecord::open(&paths.config.join(SYNCED_FILE))?;
        let sizes = db.path_sizes()?;
        return Ok((import_plan::plan_synced(Path::new(&f.path), &to, &set.exclude, &known, &sizes, &cache, &record, progress), set));
    }
    let root = match f.kind {
        "card" => PathBuf::from(&f.path).join("DCIM"),
        _ => PathBuf::from(&f.path),
    };
    let card = f.kind == "card";
    let mut plan = import_plan::plan(&root, &f.name, f.layout, &set.exclude, &known, &cache, progress);
    plan.copy_only = card;
    Ok((plan, set))
}

/// Ejects a card or device (F018/S07): only a volume under `/Volumes` with `DCIM/`.
#[tauri::command]
async fn eject_card(path: String) -> Result<(), String> {
    on_blocking(move || {
        let p = Path::new(&path);
        if !p.starts_with("/Volumes") || !p.join("DCIM").is_dir() {
            return Err(format!("{path}: geen kaart"));
        }
        let out = std::process::Command::new("diskutil").arg("eject").arg(p).output().map_err(|e| e.to_string())?;
        if out.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).trim().to_string()) }
    })
    .await
}

/// What imports took from synced folders (F018/S06).
const SYNCED_FILE: &str = "familylightbox-synced.sqlite";

/// Progress events at most five times a second, and the last one always.
fn throttled<'a, T: serde::Serialize>(app: &'a tauri::AppHandle, event: &'a str, key: &'a str) -> impl FnMut(&T, bool) + 'a {
    let mut last = std::time::Instant::now() - std::time::Duration::from_secs(1);
    move |payload: &T, last_one: bool| {
        if last_one || last.elapsed() >= std::time::Duration::from_millis(200) {
            last = std::time::Instant::now();
            let _ = app.emit(event, (key, payload));
        }
    }
}

/// What an import of the source at `path` would do (F018/S04): read and hash its files, compare
/// with the library. Nothing is copied. Progress as `import-plan-progress` events.
#[tauri::command]
async fn import_plan(path: String, app: tauri::AppHandle, paths: State<'_, Paths>) -> Result<import_plan::SourcePlan, String> {
    let paths = paths.inner().clone();
    on_blocking(move || {
        let mut emit = throttled(&app, "import-plan-progress", &path);
        plan_source(&paths, &path, &mut |pr| emit(pr, pr.files_done == pr.files_total)).map(|(p, _)| p)
    })
    .await
}

/// One import at a time (F018/S05); the sync and the previews wait for it.
#[derive(Default)]
pub struct ImportRunner {
    running: std::sync::atomic::AtomicBool,
}

impl ImportRunner {
    pub fn running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// How an import ended, for the import view and the footer.
#[derive(Clone, serde::Serialize)]
pub struct ImportDone {
    pub id: String,
    pub name: String,
    pub step: import_job::Step,
    pub copied: usize,
    pub duplicates: usize,
    pub failed: usize,
    pub problems: Vec<String>,
    pub error: Option<String>,
}

/// Runs a job on this thread with progress events, then rebuilds the index (the previews follow).
fn run_import(app: &tauri::AppHandle, paths: &Paths, job: &mut import_job::Job) -> ImportDone {
    let (settings, _) = config::load_settings(&paths.config);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    let share = mounts::NasMounts::chosen(&found.library).map(|m| PathBuf::from(&m.mount_point));
    let device = config::load_device(&paths.config).ok().flatten().map(|d| d.name).unwrap_or_else(|| "app".into());
    let taken = paths.config.join(SYNCED_FILE);
    let env = import_job::Env { library: &paths.library, share: share.as_deref(), device: &device, keep_from: import_job::keep_from(job.keep_months), taken: Some(&taken) };
    let duplicates = job.count(import_job::FileState::Duplicate);
    let id = job.id.clone();
    let mut emit = throttled(app, "import-progress", &id);
    let error = import_job::run(job, &env, &mut |p| emit(p, p.done == p.total), None).err();
    let done = job.count(import_job::FileState::Done);
    let out = ImportDone {
        id: job.id.clone(),
        name: job.name.clone(),
        step: job.step,
        copied: done.saturating_sub(duplicates),
        duplicates,
        failed: job.count(import_job::FileState::Failed),
        problems: job.problems.clone(),
        error,
    };
    if let Some(e) = &out.error {
        report(app, log::Level::Warn, "import", e);
    }
    for p in out.problems.iter().take(20) {
        report(app, log::Level::Warn, "import", p);
    }
    out
}

/// Starts importing the source at `path` (F018/S05): plan again, stage, NAS, local, `_imported/`.
#[tauri::command]
fn import_start(path: String, app: tauri::AppHandle, paths: State<Paths>, runner: State<ImportRunner>, sync: State<runner::SyncRunner>) -> Result<(), String> {
    if sync.status().running {
        return Err("er loopt een synchronisatie: importeren kan daarna".into());
    }
    if runner.running.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Err("er loopt al een import".into());
    }
    #[cfg(target_os = "macos")]
    app.state::<previews_job::MakeRunner>().stop(false);
    let paths = paths.inner().clone();
    std::thread::spawn(move || {
        let done = (|| -> Result<ImportDone, String> {
            let mut emit = throttled(&app, "import-plan-progress", &path);
            let (plan, set) = plan_source(&paths, &path, &mut |pr| emit(pr, pr.files_done == pr.files_total))?;
            let keep = config::load_settings(&paths.config).0.import.keep_months.unwrap_or(18);
            let drop_root = Path::new(&path).parent().unwrap_or(Path::new("/"));
            let id = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
            let mut job = import_job::Job::from_plan(&plan, drop_root, &id, set.delete_imported, keep);
            Ok(run_import(&app, &paths, &mut job))
        })();
        finish_import(&app, done);
    });
    Ok(())
}

fn finish_import(app: &tauri::AppHandle, done: Result<ImportDone, String>) {
    app.state::<ImportRunner>().running.store(false, std::sync::atomic::Ordering::SeqCst);
    match done {
        Ok(d) => {
            let _ = app.emit("import-done", &d);
            EventSink(app.clone()).rebuild_index();
        }
        Err(e) => {
            report(app, log::Level::Warn, "import", &e);
            let _ = app.emit("import-failed", &e);
        }
    }
}

/// Imports that didn't finish (F018/S05), to continue.
#[tauri::command]
fn import_pending(paths: State<Paths>) -> Vec<import_job::Job> {
    import_job::pending(&paths.library).into_iter().map(|mut j| { j.files.clear(); j }).collect()
}

#[tauri::command]
fn import_resume(id: String, app: tauri::AppHandle, paths: State<Paths>, runner: State<ImportRunner>, sync: State<runner::SyncRunner>) -> Result<(), String> {
    if sync.status().running {
        return Err("er loopt een synchronisatie: importeren kan daarna".into());
    }
    let mut job = import_job::pending(&paths.library).into_iter().find(|j| j.id == id).ok_or("deze import is al afgewerkt")?;
    if runner.running.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Err("er loopt al een import".into());
    }
    #[cfg(target_os = "macos")]
    app.state::<previews_job::MakeRunner>().stop(false);
    let paths = paths.inner().clone();
    std::thread::spawn(move || {
        let done = run_import(&app, &paths, &mut job);
        finish_import(&app, Ok(done));
    });
    Ok(())
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
    /// Where the layout in use came from (F019/S01), or why the NAS's file couldn't be read.
    pub layout_source: Option<layout::Source>,
    pub layout_problem: Option<String>,
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
    // the layout first: it says which shares and folders the rest are
    let mut refreshed = layout::refresh(config, &mounts::system_mounts(), &settings.nas_host, settings.home());
    // a moved library: the file says where it lives now, and that share has it (F019/S04)
    if let Some(next) = layout::next_home(config, &mounts::system_mounts(), &settings.nas_host, settings.home())
        && config::set_home_share(config, &next).is_ok() {
            refreshed = layout::refresh(config, &mounts::system_mounts(), &settings.nas_host, &next);
        }
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
    let (layout_source, layout_problem) = match refreshed {
        Ok(s) => (Some(s), None),
        Err(e) => (None, Some(e)),
    };
    NasCheck {
        layout_source,
        layout_problem,
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
        // a rename of sources done on another Mac: the same moves here (F022/S04)
        let renamed = apply_renames(&self.0);
        // What came in only shows in the grid once the index has it (F008/S02).
        if renamed > 0 || state.manifests_copied > 0 || state.previews_copied > 0 {
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

/// Applies the renames in the local copy of `Lists` this Mac hasn't applied yet (F022/S04);
/// returns how many files moved. A failure is reported and tried again after the next sync.
fn apply_renames(app: &tauri::AppHandle) -> usize {
    let paths = app.state::<Paths>().inner().clone();
    let mut state = config::load_state(&paths.config);
    let mut moved = 0;
    let todo: Vec<rename::Record> = rename::records(&paths.lists).into_iter().filter(|r| !state.renames_applied.contains(&r.id)).collect();
    for rec in todo {
        match rename::apply(&rec, &paths.library, &paths.previews) {
            Ok(n) => {
                moved += n.max(1);
                state.renames_applied.push(rec.id.clone());
                let _ = config::save_state(&paths.config, &state);
            }
            Err(e) => report(app, log::Level::Warn, "bronnen", &format!("hernoeming {} niet toegepast: {e}", rec.id)),
        }
    }
    moved
}

/// What a rename would do (F022/S01), read from the NAS; nothing is written.
#[tauri::command]
async fn rename_plan(from: Vec<String>, to: String, paths: State<'_, Paths>) -> Result<rename::RenamePlan, String> {
    let paths = paths.inner().clone();
    on_blocking(move || {
        let (library, previews, _) = rename_shares(&paths)?;
        rename::plan(&library, &previews, &from, to.trim())
    })
    .await
}

/// The `Library`, `Previews` and `Lists` folders on the NAS, all three needed for a rename.
fn rename_shares(paths: &Paths) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let (settings, _) = config::load_settings(&paths.config);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    let at = |list: &[mounts::Mount], what: &str| {
        mounts::NasMounts::chosen(list).map(|m| PathBuf::from(&m.mount_point)).ok_or(format!("{what} is niet verbonden: hernoemen kan alleen met de NAS"))
    };
    Ok((at(&found.library, "de bibliotheek")?, at(&found.previews, "Previews")?, at(&found.lists, "Lijsten")?))
}

/// How a rename ended, for the window.
#[derive(Clone, serde::Serialize)]
pub struct RenameDone {
    pub to: String,
    pub error: Option<String>,
}

/// Executes a rename on the NAS (F022/S03), then applies it here (S04); only after the window
/// confirmed a snapshot of today. Progress as `rename-progress` (months done, of), the end as
/// `rename-done`. One at a time, and not during an import or a sync.
#[tauri::command]
fn rename_start(from: Vec<String>, to: String, snapshot_today: bool, app: tauri::AppHandle, runner: State<ImportRunner>, sync: State<runner::SyncRunner>) -> Result<(), String> {
    // a rename that was interrupted continues without asking again: its snapshot was confirmed
    let mut sorted = from.clone();
    sorted.sort();
    let continuing = rename::Job::pending(&app.state::<Paths>().config).is_some_and(|j| j.record.from == sorted && j.record.to == to.trim());
    if !snapshot_today && !continuing {
        return Err("maak eerst een snapshot van de NAS".into());
    }
    if sync.status().running {
        return Err("er loopt een synchronisatie: hernoemen kan daarna".into());
    }
    if runner.running.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Err("er loopt een import of een hernoeming".into());
    }
    #[cfg(target_os = "macos")]
    app.state::<previews_job::MakeRunner>().stop(false);
    std::thread::spawn(move || {
        let paths = app.state::<Paths>().inner().clone();
        let to = to.trim().to_string();
        let result = (|| -> Result<(), String> {
            let (library, previews, lists) = rename_shares(&paths)?;
            let device = config::load_device(&paths.config)?.ok_or("dit toestel heeft nog geen naam")?;
            let mut job = match rename::Job::pending(&paths.config) {
                Some(j) if j.record.to == to && j.record.from == { let mut f = from.clone(); f.sort(); f } => j,
                Some(j) => return Err(format!("eerst de onderbroken hernoeming naar {} afwerken", j.record.to)),
                None => rename::Job::new(&rename::plan(&library, &previews, &from, &to)?, &device.name, &device.owner),
            };
            let emit = app.clone();
            rename::execute(&mut job, &library, &previews, &lists, &paths.config, &mut |n, t| { let _ = emit.emit("rename-progress", (n, t)); })?;
            // here at once: the files, the months' manifests from the NAS, the record applied
            rename::apply(&job.record, &paths.library, &paths.previews)?;
            for month in &job.months {
                let (nas, here) = (library.join(month).join(manifest::MANIFEST_NAME), paths.library.join(month).join(manifest::MANIFEST_NAME));
                if here.exists() && let Some(m) = manifest::read(&nas) {
                    manifest::write(&here, &m)?;
                }
            }
            let mut state = config::load_state(&paths.config);
            state.renames_applied.push(job.record.id.clone());
            config::save_state(&paths.config, &state)
        })();
        app.state::<ImportRunner>().running.store(false, std::sync::atomic::Ordering::SeqCst);
        if let Err(e) = &result {
            report(&app, log::Level::Error, "bronnen", &format!("hernoemen naar {to}: {e}"));
        }
        let _ = app.emit("rename-done", RenameDone { to, error: result.err() });
        EventSink(app.clone()).rebuild_index();
    });
    Ok(())
}

/// One export at a time (F024/S02).
#[derive(Default)]
pub struct ExportRunner(std::sync::atomic::AtomicBool);

#[tauri::command]
fn export_presets() -> Vec<export::Preset> {
    export::PRESETS.to_vec()
}

/// A list's photos and videos from the index, once each (a hash in several places gives its
/// first), and the list's name and slug.
fn list_media(paths: &Paths, id: &str) -> Result<(String, Vec<db::Item>), String> {
    let (state, mut all) = load_lists(paths);
    if let Some(p) = state.problem {
        return Err(p);
    }
    let list = all.remove(id).ok_or_else(|| format!("onbekende lijst: {id}"))?;
    let hashes: Vec<String> = list.photos.into_keys().collect();
    let mut seen = std::collections::HashSet::new();
    let items = Db::open(&paths.index())?.items_by_hash(&hashes)?.into_iter().map(|(_, i)| i).filter(|i| seen.insert(i.hash.clone())).collect();
    Ok((list.name, items))
}

/// What an export of a list at a preset makes (F024/S03): its files, and the folder by default.
#[derive(serde::Serialize)]
pub struct ExportPlan {
    pub photos: usize,
    pub videos: usize,
    pub files: usize,
    /// `Exports/…` on the NAS, or none when it isn't connected.
    pub dest: Option<String>,
}

#[tauri::command]
async fn export_plan(list: String, preset: String, custom: Option<export::Custom>, state: State<'_, Paths>) -> Result<ExportPlan, String> {
    let p = state.inner().clone();
    on_blocking(move || {
        let own = custom.map(|c| c.preset()).transpose()?;
        let pr = own.as_ref().or_else(|| export::preset(&preset)).ok_or("onbekende voorinstelling")?;
        let (_, items) = list_media(&p, &list)?;
        let parts = export::parts(&items, pr, &|path| sidecars_of(&p, path));
        Ok(ExportPlan {
            photos: items.iter().filter(|i| i.kind == "photo").count(),
            videos: items.iter().filter(|i| i.kind == "video" && i.live.is_none()).count(),
            files: parts.len(),
            dest: exports_folder(&p).map(|e| e.join(export::folder_name(list.rsplit('/').next().unwrap_or(&list), pr, &chrono::Local::now())).display().to_string()),
        })
    })
    .await
}

/// The `Exports` folder on the NAS, from the layout.
fn exports_folder(paths: &Paths) -> Option<PathBuf> {
    let (settings, _) = config::load_settings(&paths.config);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    mounts::NasMounts::chosen(&found.exports).map(|m| PathBuf::from(&m.mount_point))
}

/// An original's sidecars, from its month's manifest in the working copy.
fn sidecars_of(paths: &Paths, path: &str) -> Vec<String> {
    let mut it = path.splitn(3, '/');
    let (Some(y), Some(m)) = (it.next(), it.next()) else { return vec![] };
    let Some(man) = manifest::read(&paths.library.join(format!("{y}/{m}")).join(manifest::MANIFEST_NAME)) else { return vec![] };
    man["files"].as_array().into_iter().flatten()
        .find(|r| r["path"].as_str() == Some(path))
        .map(|r| r["sidecars"].as_array().into_iter().flatten().filter_map(|x| x.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

/// How an export ended, for the window.
#[derive(Clone, serde::Serialize)]
pub struct ExportDone {
    pub files: usize,
    pub dest: String,
    pub problems: Vec<String>,
    pub error: Option<String>,
}

/// Exports a list at a preset into `dest` (by default `Exports/…` on the NAS), or continues the
/// export that was interrupted when `list` is empty (F024/S02). Originals not here are read from
/// the NAS. Progress as `export-progress`, the end as `export-done`.
#[tauri::command]
fn export_start(list: String, preset: String, custom: Option<export::Custom>, dest: Option<String>, app: tauri::AppHandle, runner: State<ExportRunner>) -> Result<(), String> {
    // an own setting is checked before anything starts
    if let Some(c) = &custom {
        c.preset()?;
    }
    if runner.0.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Err("er loopt al een export".into());
    }
    std::thread::spawn(move || {
        let p = app.state::<Paths>().inner().clone();
        let mut dest_shown = String::new();
        let result = (|| -> Result<(usize, Vec<String>), String> {
            let mut job = match export::Job::pending(&p.config) {
                Some(j) => j,
                None if list.is_empty() => return Ok((0, vec![])),
                None => {
                    let own = custom.map(|c| c.preset()).transpose()?;
                    let pr = own.as_ref().or_else(|| export::preset(&preset)).ok_or("onbekende voorinstelling")?;
                    let (name, items) = list_media(&p, &list)?;
                    let folder = match dest.filter(|d| !d.trim().is_empty()) {
                        Some(d) => PathBuf::from(d),
                        None => exports_folder(&p).ok_or("Exports op de NAS is niet verbonden: kies een map")?.join(export::folder_name(list.rsplit('/').next().unwrap_or(&list), pr, &chrono::Local::now())),
                    };
                    let by = config::load_device(&p.config).ok().flatten().map(|d| d.owner).unwrap_or_default();
                    let parts = export::parts(&items, pr, &|path| sidecars_of(&p, path));
                    export::Job { id: chrono::Local::now().format("%Y%m%d-%H%M%S").to_string(), list: list.clone(), list_name: name, preset: pr.id.into(), dest: folder.display().to_string(), by, custom, parts, problems: vec![] }
                }
            };
            dest_shown = job.dest.clone();
            let pr = match &job.custom {
                Some(c) => c.preset()?,
                None => *export::preset(&job.preset).ok_or("onbekende voorinstelling")?,
            };
            let (settings, _) = config::load_settings(&p.config);
            let found = mounts::find_shares(&mounts::system_mounts(), &settings);
            let nas = mounts::NasMounts::chosen(&found.library).map(|m| PathBuf::from(&m.mount_point));
            let render = move |src: &Path, part: &export::Part, out: &Path| preview::render(src, pr.long, pr.quality, pr.dpi, part.taken.as_deref(), part.camera.as_deref(), out);
            let emit = app.clone();
            export::run(&mut job, &p.library, nas.as_deref(), &p.config, &render, &mut |pr| { let _ = emit.emit("export-progress", pr); })?;
            Ok((job.parts.len(), job.problems))
        })();
        app.state::<ExportRunner>().0.store(false, std::sync::atomic::Ordering::SeqCst);
        let done = match result {
            Ok((files, problems)) => ExportDone { files, dest: dest_shown, problems, error: None },
            Err(e) => ExportDone { files: 0, dest: dest_shown, problems: vec![], error: Some(e) },
        };
        for pr in done.problems.iter().chain(&done.error) {
            report(&app, log::Level::Warn, "export", pr);
        }
        let _ = app.emit("export-done", done);
    });
    Ok(())
}

#[tauri::command]
fn export_pending(paths: State<Paths>) -> bool {
    export::Job::pending(&paths.config).is_some()
}

/// Whether the Lightroom plug-in is installed and the app's (F023/S03).
#[tauri::command]
fn lightroom_plugin_status() -> lrplugin::Status {
    lrplugin::status(&lrplugin::modules(&home()))
}

/// Installs the app's Lightroom plug-in, or replaces an older one (F023/S03).
#[tauri::command]
fn lightroom_plugin_install() -> Result<lrplugin::Status, String> {
    let m = lrplugin::modules(&home());
    lrplugin::install(&m)?;
    Ok(lrplugin::status(&m))
}

/// One fetch of originals at a time (F009/S02).
#[derive(Default)]
pub struct FetchRunner(std::sync::atomic::AtomicBool);

/// What fetching these originals takes (F009/S01), from the manifests in the working copy.
#[tauri::command]
async fn fetch_plan(paths: Vec<String>, state: State<'_, Paths>) -> Result<fetch::FetchPlan, String> {
    let p = state.inner().clone();
    on_blocking(move || Ok(fetch::plan(&p.library, &paths, pull::free_bytes(&p.library)))).await
}

/// How a fetch ended, for the window.
#[derive(Clone, serde::Serialize)]
pub struct FetchDone {
    pub fetched: usize,
    pub problems: Vec<String>,
    pub error: Option<String>,
}

/// Fetches the originals of `paths` (F009/S02), or continues the fetch that was interrupted when
/// `paths` is empty. Progress as `fetch-progress`, the end as `fetch-done`; the index rebuilt.
#[tauri::command]
fn fetch_start(paths: Vec<String>, app: tauri::AppHandle, runner: State<FetchRunner>) -> Result<(), String> {
    if runner.0.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Err("er worden al originelen opgehaald".into());
    }
    std::thread::spawn(move || {
        let p = app.state::<Paths>().inner().clone();
        let result = (|| -> Result<(usize, Vec<String>), String> {
            let (settings, _) = config::load_settings(&p.config);
            let found = mounts::find_shares(&mounts::system_mounts(), &settings);
            let nas = mounts::NasMounts::chosen(&found.library).map(|m| PathBuf::from(&m.mount_point)).ok_or("de bibliotheek op de NAS is niet verbonden")?;
            let mut job = match fetch::Job::pending(&p.config) {
                Some(j) => j,
                None if paths.is_empty() => return Ok((0, vec![])),
                None => {
                    let plan = fetch::plan(&p.library, &paths, pull::free_bytes(&p.library));
                    if plan.fits == Some(false) {
                        return Err("niet genoeg plaats op deze Mac".into());
                    }
                    fetch::Job::new(&plan)
                }
            };
            let emit = app.clone();
            fetch::run(&mut job, &nas, &p.library, &p.config, &mut |pr| { let _ = emit.emit("fetch-progress", pr); })?;
            Ok((job.files.iter().filter(|f| !f.sidecar).count(), job.problems))
        })();
        app.state::<FetchRunner>().0.store(false, std::sync::atomic::Ordering::SeqCst);
        let done = match result {
            Ok((fetched, problems)) => FetchDone { fetched, problems, error: None },
            Err(e) => FetchDone { fetched: 0, problems: vec![], error: Some(e) },
        };
        for pr in done.problems.iter().chain(&done.error) {
            report(&app, log::Level::Warn, "originelen", pr);
        }
        let _ = app.emit("fetch-done", done);
        EventSink(app.clone()).rebuild_index();
    });
    Ok(())
}

/// Whether a fetch was interrupted (F009/S02), to continue once the NAS is there.
#[tauri::command]
fn fetch_pending(paths: State<Paths>) -> bool {
    fetch::Job::pending(&paths.config).is_some()
}

/// A rename this Mac started and didn't finish (F022/S03): its sources and target, to continue.
#[tauri::command]
fn rename_pending(paths: State<Paths>) -> Option<(Vec<String>, String)> {
    rename::Job::pending(&paths.config).map(|j| (j.record.from, j.record.to))
}

impl EventSink {
    fn rebuild_index(&self) {
        let job = self.0.state::<Paths>().index_job();
        // a new Mac has no working copy until the first sync made it: nothing to build, no error
        if !job.library.is_dir() {
            return;
        }
        self.0.state::<index::IndexRunner>().request(job, Arc::new(EventSink(self.0.clone())));
    }
}

/// The file the Lightroom plug-in reads (F023/S01), after the lists or the index changed; a
/// failure only means the plug-in reads the previous one.
fn write_lightroom(paths: &Paths) {
    let (_, merged) = load_lists(paths);
    let path_of: std::collections::HashMap<String, String> = Db::open(&paths.index())
        .and_then(|db| db.path_hashes())
        .map(|m| m.into_iter().map(|(path, hash)| (hash, path)).collect())
        .unwrap_or_default();
    let mut file = lightroom::build(&merged, &paths.library, &path_of, &lists::now());
    file.ratings = lightroom::ratings(&load_marks(paths).marks, &paths.library, &path_of);
    let _ = lightroom::write(&paths.config, &file);
}

impl index::IndexSink for EventSink {
    fn done(&self, state: &index::IndexState) {
        let _ = self.0.emit("index-done", state);
        write_lightroom(&self.0.state::<Paths>());
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
            || app.state::<ImportRunner>().running()
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
    let db = Db::open(&paths.index())?;
    let missing = db.missing_previews()?;
    // a failure of a file that is no photo or video any more (below `.xvpics`), or has its preview
    // by now, isn't shown
    let still = db.without_preview()?;
    let failed = previews_plan::Record::open(&paths.config.join(previews_plan::RECORD_FILE))?
        .failures()?
        .into_iter()
        .filter(|f| still.contains(&f.path))
        .collect();
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
    if !job.library.is_dir() || (only_if_stale && !runner.stale(&job)) {
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
fn sync_jobs(paths: &Paths, scope: runner::Scope) -> runner::Jobs {
    let (settings, _) = config::load_settings(&paths.config);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    runner::Jobs {
        manifests: pull::manifests_job(&settings, &found, &paths.library),
        lists: pull::lists_job(&settings, &found, &paths.lists),
        previews: pull::previews_job(&settings, &found, &paths.previews),
        config_dir: paths.config.clone(),
        free: pull::free_bytes,
        scope,
        confirm_above: pull::CONFIRM_ABOVE,
    }
}

/// Start a sync in the background: `kind` is "all" (manifests, then previews) or "previews"
/// (after the confirmation dialog). Progress arrives as `sync-progress`, the end as `sync-done`.
/// `at_launch` is the window's own sync at start: only once per process, so reloading the window
/// (⌘R, or the dev server) doesn't sync again and interrupt the previews being made; `Some(false)`
/// is the window's quiet sync after a reconnect (F006/S08).
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
    if app.state::<ImportRunner>().running() {
        return Err("er loopt een import: synchroniseren kan daarna".into());
    }
    // no previews are made during a sync; they start again after it (F012/S03)
    #[cfg(target_os = "macos")]
    app.state::<previews_job::MakeRunner>().stop(false);
    let kind = match kind.as_str() {
        "all" => runner::Kind::All,
        "previews" => runner::Kind::Previews,
        other => return Err(format!("onbekende synchronisatie: {other}")),
    };
    // the sync at start counts the previews only where something can be new, and every year once a
    // week; Synchroniseren in the settings always every year (F007/S07)
    let last_full = config::load_state(&paths.config).previews_full_at;
    // `at_launch: Some(false)` is a quiet later sync, after a reconnect (F006/S08): the same scope
    let scope = if at_launch.is_some() && !runner::full_due(last_full.as_deref(), chrono::Local::now().fixed_offset()) {
        runner::Scope::WhereNew { index: paths.index(), record: paths.config.join(previews_plan::RECORD_FILE) }
    } else {
        runner::Scope::Full
    };
    runner.start(kind, confirmed, sync_jobs(&paths, scope), Arc::new(EventSink(app))).map(|_| ())
}

#[tauri::command]
fn sync_cancel(runner: State<runner::SyncRunner>) {
    runner.cancel();
}

#[tauri::command]
fn sync_status(runner: State<runner::SyncRunner>) -> runner::SyncState {
    runner.status()
}

/// How many changes wait in the outbox for the NAS (F020/S03).
#[tauri::command]
fn outbox_count(paths: State<Paths>) -> usize {
    outbox::count(&paths.config)
}

/// Sends what waits in the outbox, when the `Lists` share is there (F020/S02); answers how many
/// changes still wait. The summaries of the lists it touched are written afterwards.
#[tauri::command]
async fn outbox_send(app: tauri::AppHandle) -> Result<usize, String> {
    on_blocking(move || {
        let paths = app.state::<Paths>().inner().clone();
        let Some(share) = lists_share(&paths) else { return Ok(outbox::count(&paths.config)) };
        if outbox::count(&paths.config) == 0 {
            return Ok(0);
        }
        let device = config::load_device(&paths.config)?.ok_or("dit toestel heeft nog geen naam")?;
        let touched: Vec<String> = outbox::text(&paths.config, outbox::Kind::Lists)
            .lines()
            .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok()?["list"].as_str().map(str::to_string))
            .collect();
        outbox::send(&paths.config, &share, &device.name)?;
        if let Ok(mut p) = app.state::<Summaries>().0.lock() {
            p.extend(touched);
        }
        write_summaries(&app);
        write_lightroom(&paths);
        Ok(outbox::count(&paths.config))
    })
    .await
}

/// The lists from the `Lists` share, or from the local copy when it isn't connected (F003/S01).
fn load_lists(paths: &Paths) -> (lists::ListsState, std::collections::BTreeMap<String, lists::List>) {
    let (settings, _) = config::load_settings(&paths.config);
    let found = mounts::find_shares(&mounts::system_mounts(), &settings);
    let nas = mounts::NasMounts::chosen(&found.lists).map(|m| PathBuf::from(&m.mount_point));
    let (device, waiting) = outbox_of(paths, outbox::Kind::Lists);
    lists::load(nas.as_deref(), &paths.lists, device.as_deref().map(|d| (d, waiting.as_str())))
}

/// This device's name and what its outbox keeps of one kind (F020/S01).
fn outbox_of(paths: &Paths, kind: outbox::Kind) -> (Option<String>, String) {
    let device = config::load_device(&paths.config).ok().flatten().map(|d| d.name);
    (device, outbox::text(&paths.config, kind))
}

/// Writes a change's lines (F020/S01, S02): to the share when it is there, after what waits in
/// the outbox, so the order stays; to the outbox when the share isn't there or refuses them, so
/// nothing is lost. A Mac the local copy doesn't list as registered can't keep changes.
fn write_change(paths: &Paths, kind: outbox::Kind, device: &str, text: &str) -> Result<(), String> {
    let share_error = match lists_share(paths) {
        Some(share) => match outbox::send(&paths.config, &share, device).and_then(|_| lists::append_lines(&share, kind.share_dir(), device, text)) {
            Ok(()) => return Ok(()),
            Err(e) => Some(e),
        },
        None => None,
    };
    if !lists::registered(&paths.lists).map(|d| d.contains(device)).unwrap_or(false) {
        return Err(share_error.unwrap_or_else(|| "Lijsten is niet verbonden, en dit toestel werd nog nooit met de NAS geregistreerd".into()));
    }
    outbox::add(&paths.config, kind, text)
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
    tauri::async_runtime::spawn_blocking(move || load_marks(&paths))
        .await
        .map_err(|e| e.to_string())
}

/// Writes a change to stars or hearts on the share and answers with the marks as they are now.
fn change_marks(paths: &Paths, change: marks::Change) -> Result<marks::MarksState, String> {
    let device = config::load_device(&paths.config)?.ok_or("dit toestel heeft nog geen naam")?;
    if let Some(why) = marks::refusal(&change) {
        return Err(why);
    }
    let text = marks::lines(&change, &lists::now(), &device.name, &device.owner).concat();
    write_change(paths, outbox::Kind::Marks, &device.name, &text)?;
    write_lightroom(paths);
    Ok(load_marks(paths))
}

/// The marks from the share or the local copy, with what waits in the outbox (F020/S01).
fn load_marks(paths: &Paths) -> marks::MarksState {
    let (device, waiting) = outbox_of(paths, outbox::Kind::Marks);
    marks::load(lists_share(paths).as_deref(), &paths.lists, device.as_deref().map(|d| (d, waiting.as_str())))
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
    let device = config::load_device(&paths.config)?.ok_or("dit toestel heeft nog geen naam")?;
    // the lists as this Mac knows them, with what waits in the outbox (F020/S01)
    let (_, merged) = load_lists(&paths);
    let change = make(&merged);
    if let Some(why) = lists::refusal(&change, &merged) {
        return Err(why);
    }
    let text = lists::lines(&change, &lists::now(), &device.name, &device.owner).concat();
    write_change(&paths, outbox::Kind::Lists, &device.name, &text)?;
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
    write_lightroom(&paths);
    Ok(ListChanged { id, state: load_lists(&paths).0 })
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

/// The sources for the filter bar's list (F002/S05).
#[tauri::command]
async fn sources(paths: State<'_, Paths>) -> Result<Vec<(String, i64, Option<String>)>, String> {
    let paths = paths.inner().clone();
    on_blocking(move || Db::open(&paths.index())?.source_counts()).await
}

/// The originals of these months or years that aren't here (F009/S04).
#[tauri::command]
async fn not_local(keys: Vec<String>, paths: State<'_, Paths>) -> Result<Vec<String>, String> {
    let index = paths.index();
    on_blocking(move || Db::open(&index)?.not_local(&keys)).await
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
        .manage(ImportRunner::default())
        .manage(FetchRunner::default())
        .manage(ExportRunner::default())
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
            nas_whoami, not_local, nas_layout, layout_dry_check, layout_home, layout_set_up, import_sources, import_plan, import_start, import_pending, import_resume, local_folders, free_space, choose_local, relaunch, lists_state, lists_stamp, list_photos, marks_state, marks_rate, marks_heart, marks_hide, marks_stack, marks_unstack, marks_color, items_by_hash, months_filtered, items_filtered, count_per_month, cameras, sources, outbox_count, outbox_send, rename_plan, rename_start, rename_pending, fetch_plan, fetch_start, fetch_pending, lightroom_plugin_status, lightroom_plugin_install, export_presets, export_plan, export_start, export_pending, eject_card, list_create, list_add, list_remove, list_rename, list_archive, list_items,
            previews_status, previews_pause, previews_resume, previews_focus, previews_overview, previews_retry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_the_nas_a_change_waits_in_the_outbox_and_shows() {
        let d = tempfile::tempdir().unwrap();
        let paths = Paths { config: d.path().join("config"), previews: d.path().join("p"), library: d.path().join("l"), lists: d.path().join("lists") };
        // no NAS in these settings; a Mac never registered can't keep changes
        config::create_device(&paths.config, "mbp-test", "peter").unwrap();
        let change = marks::Change::Rate { photos: vec!["sha256:a".into()], stars: 4 };
        assert!(change_marks(&paths, change.clone()).unwrap_err().contains("nog nooit"));
        assert_eq!(outbox::count(&paths.config), 0);
        // registered once: the local copy lists it
        std::fs::create_dir_all(&paths.lists).unwrap();
        std::fs::write(paths.lists.join(lists::DEVICES_FILE), r#"{"devices": [{"name": "mbp-test", "owner": "peter"}]}"#).unwrap();
        let state = change_marks(&paths, change).unwrap();
        assert_eq!(state.marks.ratings["sha256:a"].stars, 4);
        assert_eq!(outbox::count(&paths.config), 1);
        // and after a restart, read again from the local copy with the outbox
        assert_eq!(load_marks(&paths).marks.ratings["sha256:a"].stars, 4);
    }

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
