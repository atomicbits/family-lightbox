// The keyboard shortcuts, for the overlay that ⌘? (Ctrl+? elsewhere) opens.
import { t } from "../i18n";

/** ⌘ on a Mac, Ctrl elsewhere. */
export function modKey(platform: string = typeof navigator === "undefined" ? "" : navigator.platform): string {
  return /mac/i.test(platform) ? "⌘" : "Ctrl";
}

/** Whether a key press opens the shortcuts: ⌘? or Ctrl+? (? is shift-/). */
export function isShortcutsKey(e: { key: string; metaKey: boolean; ctrlKey: boolean; shiftKey: boolean }): boolean {
  return (e.metaKey || e.ctrlKey) && (e.key === "?" || (e.shiftKey && e.key === "/"));
}

export type Shortcut = { keys: string; what: string };
export type ShortcutGroup = { title: string; items: Shortcut[] };

export function shortcuts(mod: string): ShortcutGroup[] {
  return t().browse.shortcuts.groups(mod);
}
