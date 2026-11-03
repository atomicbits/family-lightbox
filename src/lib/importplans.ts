// The import's plans (F018/S04), kept by the app rather than the import view, so a plan being made
// goes on, and its result stays, when the view is closed and opened again.
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { PlanProgress, SourcePlan } from "./importplan";

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
