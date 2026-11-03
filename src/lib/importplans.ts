// The import's plans (F018/S04), kept by the app rather than the import view, so a plan being made
// goes on, and its result stays, when the view is closed and opened again.
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { PlanProgress, SourcePlan } from "./importplan";
import { importLine, type JobProgress } from "./importjob";
import { fraction } from "./progress";

export type Plans = {
  plans: Record<string, SourcePlan>;
  /** Per source path a plan being made, with its progress once known. */
  busy: Record<string, PlanProgress | true>;
  errors: Record<string, string>;
  make: (path: string) => void;
};

export function usePlans(): Plans {
  const [plans, setPlans] = useState<Record<string, SourcePlan>>({});
  const [busy, setBusy] = useState<Record<string, PlanProgress | true>>({});
  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    const off = listen<[string, PlanProgress]>("import-plan-progress", (e) =>
      setBusy((b) => (e.payload[0] in b ? { ...b, [e.payload[0]]: e.payload[1] } : b)));
    return () => { off.then((f) => f()); };
  }, []);

  const make = useCallback((path: string) => {
    setBusy((b) => ({ ...b, [path]: true }));
    setErrors((e) => { const n = { ...e }; delete n[path]; return n; });
    invoke<SourcePlan>("import_plan", { path })
      .then((p) => setPlans((ps) => ({ ...ps, [path]: p })))
      .catch((e) => setErrors((es) => ({ ...es, [path]: String(e) })))
      .finally(() => setBusy((b) => { const n = { ...b }; delete n[path]; return n; }));
  }, []);

  return { plans, busy, errors, make };
}

const base = (p: string) => p.slice(p.lastIndexOf("/") + 1);

/** What the footer's *Importeren* shows while a plan is made or an import runs (F018/S05), or
 * null: the text, and how far as 0–1 when that is known. */
export function importActivity(
  busy: Record<string, PlanProgress | true>,
  running: { name: string } | null,
  progress: JobProgress | null,
  planProgress: PlanProgress | null,
): { text: string; done: number | null } | null {
  if (running) {
    const done = progress
      ? progress.bytes_total ? fraction(progress.bytes_done, progress.bytes_total) : fraction(progress.done, progress.total)
      : planProgress ? fraction(planProgress.bytes_done, planProgress.bytes_total) : null;
    return { text: importLine(running.name, progress, planProgress), done };
  }
  const paths = Object.keys(busy);
  if (paths.length === 0) return null;
  const p = busy[paths[0]];
  const more = paths.length > 1 ? ` (+${paths.length - 1})` : "";
  if (p === true) return { text: `plan ${base(paths[0])}: bestanden zoeken…${more}`, done: null };
  const done = fraction(p.bytes_done, p.bytes_total);
  return { text: `plan ${base(paths[0])}: ${Math.floor((done ?? 0) * 100)}%${more}`, done };
}
