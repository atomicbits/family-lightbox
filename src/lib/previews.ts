// Making missing previews in the UI (F012/S03): the state from the Rust job's events, the line in
// the header with Pauzeren / Hervatten, and telling the job which month is on screen.
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Item, MonthSummary } from "./api";
import { count } from "./sync";

export type MakeState = {
  running: boolean;
  paused: boolean;
  total: number;
  done: number;
  made: number;
  copied: number;
  failed: number;
  skipped: number;
  nas_failed: number;
  message: string | null;
};

/** A preview that is there now, with the month the grid shows it under (F012/S04). */
export type Done = { path: string; month: string };

/** For the settings: photos and videos without a preview on this Mac, and what failed. */
export type Overview = { missing: number; failed: { path: string; error: string }[] };

/** The items of a month with these previews marked, or null when none of them is in it. */
export function markMade(items: Item[], paths: Set<string>): Item[] | null {
  let changed = false;
  const next = items.map((i) => {
    if (i.preview || !paths.has(i.path)) return i;
    changed = true;
    return { ...i, preview: true };
  });
  return changed ? next : null;
}

/** The month counts with these previews added. */
export function addToMonths(months: MonthSummary[], done: Done[]): MonthSummary[] {
  const per = new Map<string, number>();
  for (const d of done) per.set(d.month, (per.get(d.month) ?? 0) + 1);
  return months.map((m) => {
    const n = per.get(m.month);
    return n ? { ...m, previews: Math.min(m.photos + m.videos, m.previews + n) } : m;
  });
}

/** "85 874 foto's en video's zonder preview" or that all are there. */
export function missingText(o: Overview): string {
  if (o.missing === 0) return "alle foto's en video's hebben een preview";
  return `${count(o.missing)} ${o.missing === 1 ? "foto of video" : "foto's en video's"} zonder preview`;
}

export type PreviewsLine = {
  text: string;
  tone: "muted" | "amber";
  button: { label: string; action: "pause" | "resume" } | null;
};

/** The header line: while making, while paused, or when writing to the NAS failed. */
export function previewsLine(s: MakeState | null): PreviewsLine | null {
  if (!s) return null;
  const progress = s.total > 0 ? `: ${count(s.done)} / ${count(s.total)}` : "";
  if (s.paused) return { text: `Previews gepauzeerd${progress}`, tone: "muted", button: { label: "Hervatten", action: "resume" } };
  if (s.running && s.total > 0) return { text: `Previews maken${progress}`, tone: "muted", button: { label: "Pauzeren", action: "pause" } };
  if (!s.running && s.nas_failed > 0) {
    const what = s.nas_failed === 1 ? "preview staat" : "previews staan";
    return { text: `${count(s.nas_failed)} ${what} nog niet op de NAS`, tone: "amber", button: null };
  }
  return null;
}

export type Previews = {
  state: MakeState | null;
  line: PreviewsLine | null;
  pause: () => Promise<void>;
  resume: () => Promise<void>;
  /** The month at the top of the grid goes first. */
  focus: (month: string | null) => void;
  overview: () => Promise<Overview>;
  /** Forget the failures and make them again; returns how many. */
  retry: () => Promise<number>;
};

export function usePreviews(): Previews {
  const [state, setState] = useState<MakeState | null>(null);

  useEffect(() => {
    invoke<MakeState>("previews_status").then(setState).catch(() => {});
    const off = [
      listen<MakeState>("previews-progress", (e) => setState(e.payload)),
      listen<MakeState>("previews-done", (e) => setState(e.payload)),
    ];
    return () => { off.forEach((p) => p.then((f) => f())); };
  }, []);

  const refresh = useCallback(() => invoke<MakeState>("previews_status").then(setState).catch(() => {}), []);
  const pause = useCallback(async () => { await invoke("previews_pause"); await refresh(); }, [refresh]);
  const resume = useCallback(async () => { await invoke("previews_resume"); await refresh(); }, [refresh]);
  const focus = useCallback((month: string | null) => { invoke("previews_focus", { month }).catch(() => {}); }, []);

  const overview = useCallback(() => invoke<Overview>("previews_overview"), []);
  const retry = useCallback(() => invoke<number>("previews_retry"), []);

  return { state, line: previewsLine(state), pause, resume, focus, overview, retry };
}
