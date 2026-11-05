// A rename of sources (F022), kept by the app rather than the settings: it runs in the core, so
// the settings may be closed and the app restarted; an interrupted one continues by itself once
// the NAS is there, as its snapshot was confirmed when it started.
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { t } from "../i18n";

export type RenameDone = { to: string; error: string | null };

export type Renamer = {
  /** Months done and of how many, while one runs. */
  running: [number, number] | null;
  last: RenameDone | null;
  start: (from: string[], to: string, snapshotToday: boolean, months: number) => Promise<void>;
  /** Continues an interrupted rename, if there is one. */
  resume: () => Promise<void>;
};

export function useRenamer(onDone: () => void): Renamer {
  const [running, setRunning] = useState<[number, number] | null>(null);
  const [last, setLast] = useState<RenameDone | null>(null);
  useEffect(() => {
    const offs = [
      listen<[number, number]>("rename-progress", (e) => setRunning(e.payload)),
      listen<RenameDone>("rename-done", (e) => { setRunning(null); setLast(e.payload); onDone(); }),
    ];
    return () => { offs.forEach((o) => o.then((f) => f())); };
  }, [onDone]);
  const start = useCallback(async (from: string[], to: string, snapshotToday: boolean, months: number) => {
    setLast(null);
    await invoke("rename_start", { from, to, snapshotToday });
    setRunning([0, months]);
  }, []);
  const resume = useCallback(async () => {
    const p = await invoke<[string[], string] | null>("rename_pending");
    if (p) await start(p[0], p[1], false, 0);
  }, [start]);
  return { running, last, start, resume };
}

/** The footer's text while a rename runs: "hernoemen: maand 2 van 4". */
export function renamingText(r: [number, number]): string {
  return r[1] ? t().settings.renamer.progress(r[0], r[1]) : t().settings.renamer.busy;
}
