// English texts of the settings part (F011/S02); the reference for settings.nl.ts.
import { count } from "../index";

const s = (n: number) => (n === 1 ? "" : "s");

export const settings = {
  /** The panel itself (Settings.tsx). */
  panel: {
    title: "Settings",
    notSaved: "not saved",
    saved: "saved",
    thisDevice: "This Mac",
    ofOwner: (owner: string) => ` · ${owner}'s`,
    noName: "no name yet",
    localCopies: "Local copies",
    local: { library: "originals", previews: "previews", lists: "lists" },
    about: "About",
    messages: "Messages",
    checkTitle: "see whether there's a newer version",
    checking: "Checking…",
    checkUpdates: "Check for Updates",
    nas: "NAS",
    defaultsInUse: "The defaults are in use; the file is only overwritten when you save.",
    nasName: "Name of the NAS",
    homeShare: "Home: the shared folder with ",
    layout: "Layout",
    foundHere: "Found on this Mac",
    saveFirst: "save the settings first",
    connectTitle: "connect the shared folders that are still missing",
    connecting: "Connecting…",
    connect: "Connect",
    syncTitle: "fetch manifests and previews from the NAS",
    syncing: "Syncing…",
    sync: "Sync",
    notConnected: "not connected",
    index: "Index",
    rebuildTitle: "rebuild the index from the manifests in the working copy",
    updatingIndex: "Updating Index…",
    updateIndex: "Update Index",
    previews: "Previews",
    retryTitle: "try making the failed previews again",
    retry: "Try Again",
    failed: (n: number) => `${count(n)} failed:`,
    close: "Close",
    save: "Save",
  },

  tabs: { general: "General", nas: "NAS", import: "Import", library: "Index and Previews", sources: "Sources" },

  /** The four parts of the library on the NAS. */
  shares: { library: "Originals", previews: "Previews", lists: "Lists", exports: "Exports" },

  sources: {
    renameStopped: (to: string, error: string) => `renaming to ${to} stopped: ${error}`,
    renamed: (to: string) => `renamed to ${to}`,
    intro:
      "A source is the folder under the month where files came in. Select one or more to give them a new name or merge them. " +
      "Only the folder's name changes; day folders stay as they are. You see the plan first, and nothing changes yet.",
    interruptedBefore: "A rename to ",
    interruptedAfter: " was interrupted.",
    snapshotToday: "snapshot from today",
    continue: "Continue",
    loading: "loading sources…",
    chooseFirst: "select a source first",
    newName: "new name",
    planning: "Making Plan…",
    makePlan: "Make Plan",
    readingPlan: "reading the plan from the NAS…",
    backupNote: "A backup that tracks files by path sees the moved files as new and copies them again.",
    month: (month: string, files: number, duplicates: number, conflicts: number) =>
      `${month}: ${count(files)} file${s(files)}` +
      (duplicates ? ` · ${count(duplicates)} duplicate${s(duplicates)}, left in place` : "") +
      (conflicts ? ` · ${count(conflicts)} name conflict${s(conflicts)}` : ""),
    snapshotMade: "a snapshot of the NAS was made today",
    run: "Run",
    running: (progress: string) =>
      `${progress} · you can close this window and even restart the app: it carries on, and picks up again by itself when interrupted`,
  },

  lightroom: {
    installed: "installed: restart Lightroom, or choose File → Plug-in Manager → FamilyLightbox → Reload Plug-in there",
    update: "Update Plug-in",
    install: "Install Plug-in",
    // the plug-in's menu item has a Dutch name of its own
    howTo:
      "In Lightroom Classic: Library → Plug-in Extras → Lijsten bijwerken makes a collection for each list, and carries the stars over (a heart is 5 stars).",
    checking: "checking…",
    current: "the FamilyLightbox plug-in is installed",
    different: "a different version of the plug-in is installed: update it",
    missing: "the plug-in isn't installed yet",
  },

  moveCheck: {
    open: "check a move to another shared folder",
    share: "shared folder",
    checking: "Checking…",
    check: "Check",
    wouldFind: "what the app would find there",
    version: (v: string) => `version ${v}`,
    before05: "a version from before 0.5",
    mustUpdate: ": has to update first",
  },

  firstStart: {
    title: "Fetching the library from the NAS",
    hint: "The first time takes a while; the photos appear as soon as the index is ready.",
    connect: "Connecting to the NAS",
    notConnected: "not connected: click Connect at the bottom",
    overview: "Fetching the library's overview",
    folders: (done: number, total: number) => `${count(done)} of ${count(total)} folders`,
    searching: (folder: string) => `looking for folders… ${folder}`,
    lists: "Fetching lists",
    index: "Building the index",
    indexHint: "the months appear as soon as it's ready",
    counting: "Counting previews",
    found: (n: number, folder: string) => `${count(n)} found… ${folder}`,
    fetch: "Fetching previews",
    waiting: "waiting for your confirmation: click the previews line at the bottom",
    copying: (done: number, total: number, gbDone: string, gbTotal: string) => `${count(done)} of ${count(total)} · ${gbDone} of ${gbTotal} GB`,
    failed: "failed",
  },

  confirmPreviews: {
    title: "Fetch Previews",
    where:
      "They go into your Previews folder on this Mac. You can keep browsing meanwhile, and stop at any time with ✕; " +
      "next time it carries on where it stopped.",
    later: "Later",
    yes: "Fetch",
  },

  nas: {
    busy: "working…",
    notSetUp: "not set up",
    unreachable: "NAS unreachable",
    notConnected: "not connected",
    connected: "connected",
    partial: "partly connected",
    layoutNas: "from familylightbox.json on the NAS",
    layoutCopy: "from the copy of familylightbox.json (the NAS isn't connected)",
    layoutSettings: "this Mac's shared folders: there's no familylightbox.json on the NAS",
    layoutUnread: "not read yet",
  },

  access: { readable: "readable", readWrite: "readable and writable" },

  registration: {
    registered: "registered on the NAS just now",
    already: "registered on the NAS",
    refused: (m: string) => `not registered: ${m}`,
    failed: (m: string) => `registration failed: ${m}`,
    notTried: (m: string) => `not registered yet: ${m}`,
  },

  update: {
    upToDate: "You have the latest version",
    percent: (n: number) => ` ${n}%`,
    installing: (percent: string) => `Updating…${percent}`,
    available: (v: string, syncing: boolean) => `Version ${v} available${syncing ? ", after the sync" : ""}`,
    button: "Update",
    afterSync: "after the sync",
    installTitle: (v: string) => `install FamilyLightbox ${v} and restart`,
    isAvailable: (v: string) => `Version ${v} is available`,
  },

  sync: {
    lessThanMinute: "less than a minute",
    aboutMinute: "about a minute",
    aboutMinutes: (m: number) => `about ${count(m)} minutes`,
    confirm: (gb: string, files: number, estimate: string) => `Fetch ${gb} GB of previews (${count(files)} file${s(files)})? That takes ${estimate}.`,
    manifestsOf: (done: number, total: number) => `manifests ${count(done)}/${count(total)}`,
    manifests: (done: number) => `manifests ${count(done)}`,
    lists: "lists…",
    counting: (folder: string) => `counting previews… ${folder}`,
    copying: (done: number, total: number, gbDone: string, gbTotal: string) => `previews ${count(done)}/${count(total)} · ${gbDone}/${gbTotal} GB`,
    ready: (gb: string) => `${gb} GB of previews ready to fetch`,
    readyNoSize: "previews ready to fetch",
    failed: "sync failed",
    cancelled: "canceled",
    skipped: (n: number) => `${count(n)} skipped`,
    manifestsCopied: (n: number) => `${count(n)} manifest${s(n)}`,
    listsCopied: "lists",
    previewsCopied: (n: number) => `${count(n)} preview${s(n)}`,
    updated: (parts: string) => `updated: ${parts}`,
  },

  previews: {
    allHave: "all photos and videos have a preview",
    missing: (n: number) => `${count(n)} ${n === 1 ? "photo or video" : "photos and videos"} without a preview`,
    paused: (progress: string) => `Previews paused${progress}`,
    making: (progress: string) => `Making previews${progress}`,
    resume: "Resume",
    pause: "Pause",
    notOnNas: (n: number) => `${count(n)} ${n === 1 ? "preview isn't" : "previews aren't"} on the NAS yet`,
  },

  indexer: {
    updating: "updating index…",
    error: (e: string) => `index: ${e}`,
    broken: (n: number, folders: string) => `index: ${count(n)} unreadable manifest${s(n)}: ${folders}`,
    outcome: (items: number, manifests: number, secs: string) => `${count(items)} items from ${count(manifests)} manifests · ${secs} s`,
  },

  renamer: {
    progress: (done: number, total: number) => `renaming: month ${done} of ${total}`,
    busy: "renaming…",
  },

  rename: {
    nothing: "nothing to rename",
    files: (files: number, months: number, gb: string) => `${count(files)} file${s(files)} in ${count(months)} month${s(months)} (${gb} GB)`,
    sidecars: (n: number) => `${count(n)} sidecar${s(n)}`,
    previews: (n: number) => `${count(n)} preview${s(n)}`,
    duplicates: (n: number) => `${count(n)} duplicate${s(n)}, left in place`,
    conflicts: (n: number) => `${count(n)} name conflict${s(n)}`,
  },
};
