// The app's own index in the UI (F008/S02): the state from the Rust index runner, the line in the
// header, and keeping the selection in place when the grid reloads after a rebuild.
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Item } from "./api";
import { count } from "./sync";
import type { Sel, Section } from "./timeline";

export type BuildOutcome = {
  manifests: number;
  items: number;
  photos: number;
  videos: number;
  sidecars: number;
  other: number;
  local: number;
  previews: number;
  /** "2016/12: <reason>" per manifest that couldn't be read. */
  broken: string[];
  millis: number;
};

export type IndexState = {
  running: boolean;
  outcome: BuildOutcome | null;
  error: string | null;
  /** Builds finished since the app started. */
  builds: number;
};

export type IndexLine = { text: string; tone: "muted" | "amber"; title?: string };

/** The header line: only while a build someone waits for runs, or when something is wrong. */
export function indexLine(s: IndexState | null, waiting: boolean): IndexLine | null {
  if (!s) return null;
  if (s.running && waiting) return { text: "index bijwerken…", tone: "muted" };
  if (s.error) return { text: `index: ${s.error}`, tone: "amber" };
  const broken = s.outcome?.broken ?? [];
  if (broken.length > 0) {
    const folders = broken.map((b) => b.split(": ")[0]).join(", ");
    const what = broken.length === 1 ? "manifest onleesbaar" : "manifesten onleesbaar";
    return { text: `index: ${broken.length} ${what}: ${folders}`, tone: "amber", title: broken.join("\n") };
  }
  return null;
}

/** "124 837 items uit 364 manifesten · 1,3 s" for the settings panel. */
export function outcomeText(o: BuildOutcome): string {
  const secs = (o.millis / 1000).toFixed(1).replace(".", ",");
  return `${count(o.items)} items uit ${count(o.manifests)} manifesten · ${secs} s`;
}

/** What was selected before a reload: its month, its position and, when known, its path. */
export type Anchor = { month: string; index: number; path: string | null };

/**
 * The selection after a reload: the same item by path when it is still in its month, otherwise
 * the same position in the same month (clamped), or nothing when the month is gone.
 */
export function reselect(anchor: Anchor | null, sections: Section[], items: Item[] | undefined): Sel | null {
  if (!anchor) return null;
  const section = sections.findIndex((s) => s.month === anchor.month);
  if (section < 0 || sections[section].count === 0) return null;
  const byPath = anchor.path && items ? items.findIndex((i) => i.path === anchor.path) : -1;
  const index = byPath >= 0 ? byPath : Math.min(anchor.index, sections[section].count - 1);
  return { section, index };
}

export type Index = {
  state: IndexState | null;
  line: IndexLine | null;
  /** Rebuild; `visible` shows "index bijwerken…" until it is done, `onlyIfStale` skips it when
   * no manifest changed. */
  rebuild: (opts: { visible: boolean; onlyIfStale: boolean }) => Promise<void>;
};

export function useIndex(): Index {
  const [state, setState] = useState<IndexState | null>(null);
  const [waiting, setWaiting] = useState(false);

  useEffect(() => {
    invoke<IndexState>("index_status").then(setState).catch(() => {});
    const off = listen<IndexState>("index-done", (e) => {
      setState(e.payload);
      if (!e.payload.running) setWaiting(false);
    });
    return () => { off.then((f) => f()); };
  }, []);

  const rebuild = useCallback(async ({ visible, onlyIfStale }: { visible: boolean; onlyIfStale: boolean }) => {
    const asked = await invoke<boolean>("index_rebuild", { onlyIfStale });
    if (asked) {
      setState((s) => ({ ...(s ?? { outcome: null, error: null, builds: 0 }), running: true }));
      if (visible) setWaiting(true);
    }
  }, []);

  return { state, line: indexLine(state, waiting), rebuild };
}
