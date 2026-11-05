// English texts of the browse part (F011/S02); the reference for browse.nl.ts.
import { count } from "../index";

const photos = (n: number) => `${count(n)} ${n === 1 ? "photo" : "photos"}`;
const stars = (n: number) => `${n} ${n === 1 ? "star" : "stars"}`;
const and = (xs: string[]) => (xs.length < 2 ? xs.join("") : `${xs.slice(0, -1).join(", ")} and ${xs[xs.length - 1]}`);

/** The stacks' colors by their name in the marks events (`PALETTE` in lib/stacks.ts). */
const colors: Record<string, string> = {
  blauw: "blue",
  amber: "amber",
  groen: "green",
  roze: "pink",
  paars: "purple",
  turkoois: "turquoise",
  oranje: "orange",
  grijs: "gray",
};

export const browse = {
  colors,
  video: "video",
  loading: "loading…",
  close: "close",

  undo: {
    stack: (kind: "make" | "add" | "out" | "dissolve", n: number): string => {
      switch (kind) {
        case "make": return `stack of ${photos(n)} created`;
        case "add": return `${photos(n)} added to the stack`;
        case "out": return `${photos(n)} taken out of the stack`;
        case "dissolve": return `stack of ${photos(n)} dissolved`;
      }
    },
    hide: (n: number, hidden: boolean) => (hidden ? `${photos(n)} hidden` : `${photos(n)} no longer hidden`),
    skipped: (n: number, by: string[]) => `${photos(n)} changed ${by.length ? `by ${by.join(" and ")} ` : ""}meanwhile, not restored`,
    list: (n: number, name: string, add: boolean) => (add ? `${photos(n)} added to ${name}` : `${photos(n)} removed from ${name}`),
    color: (color: string) => `stack colored ${colors[color] ?? color}`,
    colorSkipped: "the color was changed meanwhile, not restored",
    undone: (text: string, skipped: string) => `undone: ${text}${skipped ? ` · ${skipped}` : ""}`,
    button: "Undo",
    title: (text: string) => `Undo: ${text} (⌘Z)`,
    nothing: "nothing to undo",
  },

  shortcuts: {
    title: "Keyboard Shortcuts",
    close: (mod: string) => `Esc or ${mod}? to close`,
    groups: (mod: string) => [
      {
        title: "Browse",
        items: [
          { keys: "← → ↑ ↓", what: "select the next photo" },
          { keys: "Space or Return", what: "open the photo (or double-click)" },
          { keys: "Esc", what: "close the photo, clear the selection" },
        ],
      },
      {
        title: "Select",
        items: [
          { keys: "Shift-click", what: "select a range, also across months" },
          { keys: "Shift + arrow", what: "extend the range" },
          { keys: `${mod}-click`, what: "add a photo to the selection or remove it" },
        ],
      },
      {
        title: "Stars and heart",
        items: [
          { keys: "1 – 5", what: "give stars" },
          { keys: "0", what: "remove the stars" },
          { keys: "H", what: "your heart on or off" },
        ],
      },
      {
        title: "Lists and hiding",
        items: [
          { keys: "drag", what: "photos to a list in the sidebar" },
          { keys: "right-click", what: "add to a list, hide" },
          { keys: "⌫", what: "in a list: remove the photos from it" },
          { keys: "right-click a month or year", what: "show hidden photos for now" },
        ],
      },
      {
        title: "General",
        items: [
          { keys: `${mod}Z`, what: "undo" },
          { keys: `${mod}?`, what: "this overview" },
        ],
      },
    ],
  },

  hidden: {
    count: (shown: number, hidden: number, stacks: number) => {
      const parts = [`${count(shown)} ${shown === 1 ? "photo or video" : "photos and videos"}`];
      if (hidden > 0) parts.push(`${count(hidden)} hidden`);
      if (stacks > 0) parts.push(`${count(stacks)} ${stacks === 1 ? "stack" : "stacks"}`);
      return parts.join(", ");
    },
  },

  live: {
    title: (name: string) => `Live Photo of ${name}`,
    label: "Live Photo",
  },

  messages: {
    /** For the time and date of a message. */
    locale: "en-US",
    title: "Messages",
    copy: "Copy",
    copied: "Copied",
    copyTitle: "all messages as text, to send on",
    clear: "Clear",
    none: "no messages since the app opened",
    logAt: (path: string) => `Everything is also in ${path}`,
    button: "messages",
    unseen: (n: number) => `${count(n)} new ${n === 1 ? "message" : "messages"}`,
  },

  marks: {
    rated: (n: number, by: string) => `${stars(n)} (${by})`,
    hearts: (who: string[]) => `heart from ${and(who)}`,
    noMarksYet: "no stars and hearts on this Mac yet: connect to the NAS once first",
    noOwner: "this device has no owner yet",
    change: (n: number, change: { stars: number } | { heart: boolean }) => {
      if ("stars" in change) {
        if (change.stars === 0) return `${photos(n)}: stars removed`;
        return `${photos(n)}: ${stars(change.stars)}`;
      }
      return change.heart ? `heart on ${photos(n)}` : `heart removed from ${photos(n)}`;
    },
    notSaved: (e: string) => `not saved on the NAS, changed back: ${e}`,
    waiting: (n: number) => (n === 1 ? "1 change waiting for the NAS" : `${count(n)} changes waiting for the NAS`),
    waitingTitle: "they go to the NAS by themselves once it is connected",
    differentStars: "the selected photos have different stars",
    removeStars: "remove the stars (0)",
    giveStars: (n: number) => `${stars(n)} (${n})`,
    heartOff: "remove your heart (H)",
    heartAll: "your heart on all selected photos (H)",
    heartOn: "a heart (H)",
    selected: (n: number) => photos(n),
  },

  grid: {
    hidden: "hidden",
    noPreview: "no preview",
    edited: (label: string, versions: number) => `edited in Lightroom: ${label}${versions > 1 ? ` (${versions} versions)` : ""}`,
    nasOnly: "original only on the NAS",
    fold: "fold the stack",
    unfold: "unfold the stack: show all photos",
  },

  months: {
    revealed: "hidden photos are shown (right-click to hide them again)",
    title: (counts: string, previews: number, local: number) => `${counts}; ${count(previews)} previews, ${count(local)} originals on this Mac`,
  },

  lightbox: {
    noPreview: (name: string) => `no preview for ${name}`,
    nasOnly: "the original is only on the NAS",
    versions: "versions from Lightroom",
    chosen: "FamilyLightbox shows this version and exports it",
    look: "view",
    showVersion: "Show This Version",
    rated: (n: number, by: string) => `${stars(n)} from ${by}`,
    hearts: (who: string[]) => `heart from ${who.join(", ")}`,
    fetchTitle: "copy the original from the NAS to this Mac",
    fetch: "Get Original",
    revealTitle: "show the original in the Finder",
    reveal: "Show in Finder",
    originalNasOnly: "original only on the NAS",
    openTitle: "open the original in its default app",
    open: "Open Original",
  },

  menu: {
    color: "Color",
    addTo: "Add to",
    addToRecent: "Add to (recent)",
    otherList: "Other List… ▸",
    otherListTitle: "all lists, with search",
    search: "search for a list",
    noList: "no list with that name",
  },

  nas: {
    showShares: "show the shared folders",
    retry: "Retry",
    connect: "Connect",
  },

  sync: {
    fetchTitle: "how much and how long, then get them now or later",
    fetch: "Get…",
    stop: "stop syncing",
  },

  app: {
    back: "← All Photos",
    backTitle: "back to all photos",
    missing: (n: number) => ` · ${count(n)} not on this Mac yet`,
    filterCount: (shown: number, total: number) => `${count(shown)} of ${count(total)}`,
    tileSize: "thumbnail size",
    deviceBroken: "device.json is invalid: see Settings",
    keepsGoing: "you can quit the app: it carries on afterwards",
    exporting: (done: number, total: number) => `exporting ${count(done)}/${count(total)}`,
    renameTitle: "open the sources in Settings",
    importBusy: (text: string) => `${text}: click to open the import window`,
    importIdle: "bring in new photos",
    import: "Import",
    settings: "Settings",
    lrNext: "in Lightroom, click Library → Plug-in Extras → Update Lists",
    allLocal: (next: string) => `all originals are on this Mac already: ${next}`,
    fetchDone: (done: string, next: string) => `${done}: ${next}`,
    plusHint: "drag the photos to a list, or right-click: Add to",
    open: "Open",
    removeFrom: (name: string) => `Remove from ${name}`,
    fetching: "originals are being fetched already",
    fetch: (n: number | null) => (n === null ? "Get Originals" : `Get Originals (${count(n)})`),
    fetchTitle: "from the NAS to this Mac, in their place in the library",
    allHere: "they are on this Mac already",
    hide: (n: number | null) => (n === null ? "Hide" : `Hide (${count(n)})`),
    unhide: (n: number | null) => (n === null ? "Unhide" : `Unhide (${count(n)})`),
    makeStack: (n: number) => `Make Stack (${count(n)})`,
    addToStack: (n: number) => `Add to Stack (${count(n)})`,
    takeOut: (n: number | null) => (n === null ? "Take Out of Stack" : `Take Out of Stack (${count(n)})`),
    dissolve: "Dissolve Stack",
    fold: "Fold Stack",
    unfold: "Unfold Stack",
    hideAgain: "Hide Hidden Photos Again",
    reveal: (n: number) => `Show Hidden Photos (${count(n)})`,
    nothingHidden: "nothing is hidden here",
    untilRestart: "for now, until the app restarts",
    everythingHere: "everything is on this Mac already",
    monthFetchTitle: "to this Mac, first showing how much it takes",
  },
};
