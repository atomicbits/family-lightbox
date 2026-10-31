// A running import in the UI (F018/S05): its progress from the Rust side, the line in the footer,
// and the imports that didn't finish.
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { count } from "./sync";
import type { PlanProgress } from "./importplan";

export type Step = "stage" | "nas" | "local" | "source" | "done";
export type JobProgress = { step: string; done: number; total: number };
export type ImportDone = { id: string; name: string; step: Step; copied: number; duplicates: number; failed: number; problems: string[]; error: string | null };
export type PendingJob = { id: string; name: string; source: string; step: Step; created: string };

const STEP: Record<string, string> = {
  stage: "kopiëren naar de tussenmap",
  nas: "naar de NAS",
  local: "in de werkkopie",
  source: "opruimen naar _imported",
};

/** "Hero13: naar de NAS 2/5", or the planning while it reads the files. */
export function importLine(name: string, p: JobProgress | null, plan: PlanProgress | null): string {
  if (p) return `importeren ${name}: ${STEP[p.step] ?? p.step} ${count(p.done)}/${count(p.total)}`;
  if (plan) return `importeren ${name}: bestanden lezen ${count(plan.files_done)}/${count(plan.files_total)}`;
  return `importeren ${name}…`;
}

/** "Hero13: 82 geïmporteerd · 71 al in de bibliotheek · 1 mislukt", or why it stopped. */
export function doneText(d: ImportDone): string {
  if (d.error) return `${d.name}: gestopt (${d.error})`;
  const parts = [
    `${count(d.copied)} geïmporteerd`,
    d.duplicates > 0 ? `${count(d.duplicates)} al in de bibliotheek` : null,
    d.failed > 0 ? `${count(d.failed)} mislukt` : null,
  ];
  return `${d.name}: ${parts.filter(Boolean).join(" · ")}`;
}

/** Where an unfinished import stopped. */
export function pendingText(j: PendingJob): string {
  return `${j.name}, gestopt bij: ${STEP[j.step] ?? j.step}`;
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
