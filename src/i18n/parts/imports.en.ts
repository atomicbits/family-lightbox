// English texts of the imports part (F011/S02); the reference for imports.nl.ts.
import { count } from "../index";

const at = (src: string) => `<year>/<month>/${src}/`;
const what = "A source is the folder in the library that this folder's files go into, by month";

export const imports = {
  /** A source's kind, as a short label. */
  kind: { drop: "subfolder", synced: "sync", card: "card" },

  /** A running import (importjob.ts). */
  job: {
    steps: {
      stage: "copying to the staging folder",
      nas: "to the NAS",
      local: "into the working copy",
      source: "tidying up into _imported",
      taken: "remembering what was taken",
    } as Record<string, string>,
    lineBytes: (name: string, step: string, done: number, total: number, gbDone: string, gbTotal: string) =>
      `importing ${name}: ${step} ${count(done)}/${count(total)} · ${gbDone}/${gbTotal} GB`,
    lineFiles: (name: string, step: string, done: number, total: number) => `importing ${name}: ${step} ${count(done)}/${count(total)}`,
    lineReading: (name: string, done: number, total: number) => `importing ${name}: reading files ${count(done)}/${count(total)}`,
    lineStarting: (name: string) => `importing ${name}…`,
    stopped: (name: string, error: string) => `${name}: stopped (${error})`,
    imported: (n: number) => `${count(n)} imported`,
    failed: (n: number) => `${count(n)} failed`,
    pending: (name: string, step: string) => `${name}, stopped at: ${step}`,
  },

  /** What a plan of one source says (importplan.ts). */
  plan: {
    newFiles: (n: number, gb: string) => `${count(n)} new (${gb} GB)`,
    nothingNew: "nothing new",
    inLibrary: (n: number) => `${count(n)} already in the library`,
    conflicts: (n: number) => `${count(n)} ${n === 1 ? "name conflict" : "name conflicts"}`,
    why: { partner: "from the companion file", name: "from the file name", mtime: "from the modification time" } as Record<string, string>,
    datesNotFromFile: (parts: string) => `date not from the file: ${parts}`,
    actionNew: "new",
    actionDuplicate: (of: string) => `already in the library: ${of}`,
    actionConflict: (existing: string) => `name exists already (${existing}): renamed`,
    progress: (done: number, total: number, gbDone: string, gbTotal: string) => `${count(done)} / ${count(total)} files · ${gbDone} / ${gbTotal} GB`,
    takenBefore: (n: number) => `${count(n)} taken before`,
    older: (n: number, month: string) => `${count(n)} older than ${month}`,
    liveSkipped: (n: number) => `${count(n)} Live Photo ${n === 1 ? "clip" : "clips"} skipped`,
    read: (n: number) => `${count(n)} read`,
  },

  /** The footer's *Import* while a plan is made (importplans.ts). */
  activity: {
    finding: (source: string, more: string) => `plan ${source}: finding files…${more}`,
    percent: (source: string, pct: number, more: string) => `plan ${source}: ${pct}%${more}`,
  },

  /** A source's layout and the synced folder's hint (sources.ts). */
  sources: {
    split: "photos and videos apart",
    withDay: "with day folders",
    withoutDay: "without day folders",
    hintExample: (src: string) => `${what}, e.g. ${at(src)}. Photos and videos may share a source.`,
    hintMissing: (photosGiven: boolean) => `${what}. Also give the source for the ${photosGiven ? "videos" : "photos"}; it may be the same one.`,
    hintSame: (src: string) => `Photos and videos go together into ${at(src)}.`,
    hintApart: (photos: string, videos: string) => `Photos go into ${at(photos)}, videos into ${at(videos)}.`,
  },

  /** The import view (ImportView.tsx). */
  view: {
    title: "Import",
    setUpSources: "Set up sources…",
    intro: "First a plan: which files are new, where they go, and what the library has already (by content, under any name). Nothing changes yet.",
    closeBefore: "You can close this window while a plan or an import runs: the work goes on, its progress shows at the bottom by ",
    closeAfter: ", and a click there brings you back here.",
    syncRunning: (line: string) => `a sync with the NAS is running (${line}); you can import after it`,
    busy: "busy",
    interrupted: (what: string) => `An import was interrupted: ${what}. The originals stay in the source until everything is on the NAS.`,
    resume: "Continue",
    noSources: "No sources yet: add a drop folder in the settings, or insert a card.",
    to: "to",
    and: "and",
    newSource: "new source, without day folders",
    replan: "Plan again",
    makePlan: "Make plan",
    import: "Import",
    tipSyncRunning: "a sync is running: you can import after it",
    tipPlanFirst: "makes the plan first, for you to check",
    tipSynced: "copies to the NAS; the synced folder itself stays as it is",
    tipCard: "copies to the NAS; nothing changes on the card",
    tipDrop: "copies to the NAS, then moves the originals to _imported",
    syncedOnly: "copy only: the folder itself stays as it is",
    cardOnly: "copy only: nothing changes on the card",
    tipEject: "safely remove the card",
    eject: "Eject",
    notEjected: (error: string) => `not ejected: ${error}`,
    planFirst: "first the plan: check it, then click Import again",
    nothingToImport: "nothing new to import",
    planFinding: "making plan: finding files…",
    planReading: (progress: string) => `making plan: reading files · ${progress}`,
    excluded: (n: number) => ` · ${n} excluded`,
    files: "the files",
    inLibrary: "already in the library",
    andMore: (n: number) => `… and ${n} more`,
  },

  /** The settings' *Import* tab (ImportSettings.tsx). */
  settings: {
    pickDrop: "Folder where new photos arrive in a subfolder per source",
    pickSynced: "Folder another app keeps in sync",
    tipKnown: "how this source is laid out in the library",
    tipNew: "new in the library",
    newSource: "new, without day folders",
    source: "source",
    tipDay: "where the next imports go; what is in the library already stays as it is",
    dayLast: "day folders as in the last import",
    ignore: "ignore",
    tipEmpty: "after an import that was checked on the NAS",
    emptyImported: "empty _imported",
    excludePlaceholder: "exclude, e.g. *-mask.png",
    intro: "Where new photos arrive on this laptop. Each subfolder of a drop folder is a source; a synced folder (e.g. from a phone) is never changed. Cards are recognized automatically.",
    dropFolders: "Drop folders: each subfolder is a source",
    add: "Add…",
    tipRemove: "remove (the folder itself stays)",
    noSubfolders: "no subfolders yet",
    syncedFolders: "Synced folders",
    photosSource: "Source for the photos",
    photosPlaceholder: "e.g. iPhone-photos",
    videosSource: "Source for the videos",
    videosPlaceholder: "e.g. iPhone-videos",
    tipFrom: "older months are never read; empty: from the source's first month in the library",
    from: "From",
    fromPlaceholder: "YYYY/MM",
    tipLive: "the video of a Live Photo: a video with a photo's name in the same folder; the photo itself is imported",
    skipLive: "Skip Live Photo clips",
    cards: "Cards",
  },
};
