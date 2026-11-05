// A running import in the UI (F018/S05): its progress from the Rust side, the line in the footer,
// and the imports that didn't finish.
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { gb } from "./sync";
import { t } from "../i18n";
import type { PlanProgress } from "./importplan";

export type Step = "stage" | "nas" | "local" | "source" | "done";
export type JobProgress = { step: string; done: number; total: number; bytes_done?: number; bytes_total?: number };
export type ImportDone = { id: string; name: string; step: Step; copied: number; duplicates: number; failed: number; problems: string[]; error: string | null };
export type PendingJob = { id: string; name: string; source: string; step: Step; created: string };

const step = (s: string) => t().imports.job.steps[s] ?? s;

/** "Hero13: naar de NAS 2/5", or the planning while it reads the files. */
export function importLine(name: string, p: JobProgress | null, plan: PlanProgress | null): string {
  const tx = t().imports.job;
  if (p?.bytes_total) return tx.lineBytes(name, step(p.step), p.done, p.total, gb(p.bytes_done ?? 0), gb(p.bytes_total));
  if (p) return tx.lineFiles(name, step(p.step), p.done, p.total);
  if (plan) return tx.lineReading(name, plan.files_done, plan.files_total);
  return tx.lineStarting(name);
}

/** "Hero13: 82 geïmporteerd · 71 al in de bibliotheek · 1 mislukt", or why it stopped. */
export function doneText(d: ImportDone): string {
  const tx = t().imports.job;
  if (d.error) return tx.stopped(d.name, d.error);
  const parts = [
    tx.imported(d.copied),
    d.duplicates > 0 ? t().imports.plan.inLibrary(d.duplicates) : null,
    d.failed > 0 ? tx.failed(d.failed) : null,
  ];
  return `${d.name}: ${parts.filter(Boolean).join(" · ")}`;
}

/** Where an unfinished import stopped. */
export function pendingText(j: PendingJob): string {
  return t().imports.job.pending(j.name, step(j.step));
}

export type Importer = {
  running: { name: string; path: string } | null;
  progress: JobProgress | null;
  planProgress: PlanProgress | null;
  last: ImportDone | null;
  pending: PendingJob[];
  start: (path: string, name: string) => Promise<void>;
  resume: (job: PendingJob) => Promise<void>;
  refresh: () => void;
};

export function useImporter(onDone: () => void): Importer {
  const [running, setRunning] = useState<{ name: string; path: string } | null>(null);
  const [progress, setProgress] = useState<JobProgress | null>(null);
  const [planProgress, setPlanProgress] = useState<PlanProgress | null>(null);
  const [last, setLast] = useState<ImportDone | null>(null);
  const [pending, setPending] = useState<PendingJob[]>([]);
  const refresh = useCallback(() => { invoke<PendingJob[]>("import_pending").then(setPending).catch(() => {}); }, []);

  useEffect(() => {
    refresh();
    const offs = [
      listen<[string, JobProgress]>("import-progress", (e) => setProgress(e.payload[1])),
      listen<[string, PlanProgress]>("import-plan-progress", (e) => setPlanProgress(e.payload[1])),
      listen<ImportDone>("import-done", (e) => { setLast(e.payload); setRunning(null); setProgress(null); setPlanProgress(null); refresh(); onDone(); }),
      listen<string>("import-failed", (e) => {
        setLast({ id: "", name: "import", step: "stage", copied: 0, duplicates: 0, failed: 0, problems: [], error: e.payload });
        setRunning(null); setProgress(null); setPlanProgress(null); refresh();
      }),
    ];
    return () => { offs.forEach((o) => o.then((f) => f())); };
  }, [refresh, onDone]);

  const start = useCallback(async (path: string, name: string) => {
    setLast(null);
    await invoke("import_start", { path });
    setRunning({ name, path });
  }, []);
  const resume = useCallback(async (job: PendingJob) => {
    setLast(null);
    await invoke("import_resume", { id: job.id });
    setRunning({ name: job.name, path: job.source });
  }, []);
  return { running, progress, planProgress, last, pending, start, resume, refresh };
}
