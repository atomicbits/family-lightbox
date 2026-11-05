// Fetching originals from the NAS (F009/S03): the plan's text, and the job kept by the app, so the
// window may be closed and the app restarted; an interrupted fetch goes on once the NAS is there.
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { count, gb } from "./sync";
import { t } from "../i18n";

export type FetchPlan = { files: { path: string; size: number; sidecar: boolean }[]; bytes: number; here: number; conflicts: string[]; unknown: string[]; fits: boolean | null };
export type FetchProgress = { done: number; total: number; bytes_done: number; bytes_total: number };
export type FetchDone = { fetched: number; problems: string[]; error: string | null };

/** Above this a fetch warns before it starts, with its duration (F009/S03). */
export const BIG_FETCH = 5e9;

/** "40 originelen ophalen (1,2 GB) · 3 staan er al", or why not. */
export function fetchPlanText(p: FetchPlan): string {
  const originals = p.files.filter((f) => !f.sidecar).length;
  const f = t().lists.fetch;
  const parts = [
    originals ? f.plan(f.originals(originals), gb(p.bytes)) : f.nothing,
    p.here ? f.here(p.here) : null,
    p.conflicts.length ? f.conflicts(p.conflicts.length) : null,
    p.unknown.length ? f.unknown(p.unknown.length) : null,
  ];
  return parts.filter(Boolean).join(" · ");
}

/** The footer's text while fetching: "originelen ophalen 12/40". */
export function fetchingText(p: FetchProgress): string {
  return t().lists.fetch.fetching(count(p.done), count(p.total));
}

/** After a fetch: "40 originelen opgehaald", with what went wrong. */
export function fetchDoneText(d: FetchDone): string {
  if (d.error) return t().lists.fetch.stopped(d.error);
  const n = t().lists.fetch.fetched(t().lists.fetch.originals(d.fetched));
  return d.problems.length ? `${n} · ${t().lists.notDone(d.problems.length)}` : n;
}

export type Fetcher = {
  running: FetchProgress | null;
  last: FetchDone | null;
  plan: (paths: string[]) => Promise<FetchPlan>;
  start: (paths: string[]) => Promise<void>;
  resume: () => Promise<void>;
};

export function useFetcher(onDone: (d: FetchDone) => void): Fetcher {
  const [running, setRunning] = useState<FetchProgress | null>(null);
  const [last, setLast] = useState<FetchDone | null>(null);
  useEffect(() => {
    const offs = [
      listen<FetchProgress>("fetch-progress", (e) => setRunning(e.payload)),
      listen<FetchDone>("fetch-done", (e) => { setRunning(null); setLast(e.payload); onDone(e.payload); }),
    ];
    return () => { offs.forEach((o) => o.then((f) => f())); };
  }, [onDone]);
  const plan = useCallback((paths: string[]) => invoke<FetchPlan>("fetch_plan", { paths }), []);
  const start = useCallback(async (paths: string[]) => {
    setLast(null);
    await invoke("fetch_start", { paths });
    setRunning({ done: 0, total: 0, bytes_done: 0, bytes_total: 0 });
  }, []);
  const resume = useCallback(async () => {
    if (await invoke<boolean>("fetch_pending")) await start([]);
  }, [start]);
  return { running, last, plan, start, resume };
}
