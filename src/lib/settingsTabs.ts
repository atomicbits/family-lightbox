import type { Settings } from "./api";
import { t } from "../i18n";

/** The settings' tabs (F021/S01), in the order of the tab bar. */
export const TAB_IDS = ["general", "nas", "import", "library", "sources"] as const;

export type TabId = (typeof TAB_IDS)[number];

/** The tabs with their labels in the language in use. */
export function tabs(): { id: TabId; label: string }[] {
  return TAB_IDS.map((id) => ({ id, label: t().settings.tabs[id] }));
}

const TAB_KEY = "settings-tab";

/** The tabs whose fields differ from what is saved. */
export function changedTabs(draft: Settings, saved: Settings): Set<TabId> {
  const out = new Set<TabId>();
  const same = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);
  if (draft.nas_host !== saved.nas_host || draft.home_share !== saved.home_share || !same(draft.shares, saved.shares)) out.add("nas");
  if (!same(draft.import, saved.import)) out.add("import");
  if (!same(draft.local, saved.local) || (draft.language ?? "") !== (saved.language ?? "")) out.add("general");
  return out;
}

export function readTab(v: unknown): TabId {
  return TAB_IDS.find((id) => id === v) ?? "general";
}

/** The tab last open on this Mac; storage may be missing or blocked. */
export function loadTab(): TabId {
  try {
    return readTab(localStorage.getItem(TAB_KEY));
  } catch {
    return "general";
  }
}

export function saveTab(t: TabId): void {
  try {
    localStorage.setItem(TAB_KEY, t);
  } catch {
    // not remembered; the settings work the same
  }
}
