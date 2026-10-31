import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { config, type Found } from "../lib/api";
import { actionText, datesText, monthsText, planSummary, progressText, type PlanProgress, type SourcePlan } from "../lib/importplan";
import { baseName, layoutText } from "../lib/sources";

type Props = { onClose: () => void; onOpenSettings: () => void };

const button = "px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-xs disabled:opacity-40";

/** The import (F018/S04): the sources found, and per source the plan of what an import would do.
 * Nothing is copied here; importing comes with S05. */
export default function ImportView({ onClose, onOpenSettings }: Props) {
  const [found, setFound] = useState<Found[] | null>(null);
  const [plans, setPlans] = useState<Record<string, SourcePlan>>({});
  const [busy, setBusy] = useState<Record<string, PlanProgress | true>>({});
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [open, setOpen] = useState<string | null>(null);

  useEffect(() => {
    config.importSources().then(setFound).catch((e) => setErrors({ "": String(e) }));
    const off = listen<[string, PlanProgress]>("import-plan-progress", (e) => setBusy((b) => (e.payload[0] in b ? { ...b, [e.payload[0]]: e.payload[1] } : b)));
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") { e.preventDefault(); onClose(); } };
    window.addEventListener("keydown", onKey);
    return () => { off.then((f) => f()); window.removeEventListener("keydown", onKey); };
  }, [onClose]);

  const makePlan = (path: string) => {
    setBusy((b) => ({ ...b, [path]: true }));
    setErrors((e) => { const n = { ...e }; delete n[path]; return n; });
    invoke<SourcePlan>("import_plan", { path })
      .then((p) => setPlans((ps) => ({ ...ps, [path]: p })))
      .catch((e) => setErrors((es) => ({ ...es, [path]: String(e) })))
      .finally(() => setBusy((b) => { const n = { ...b }; delete n[path]; return n; }));
  };

  const shown = (found ?? []).filter((f) => !f.ignore);

  return (
    <div className="absolute inset-0 z-30 bg-black/40 flex items-center justify-center p-4" onClick={onClose}>
      <div className="w-[44rem] max-h-full overflow-y-auto rounded-lg bg-white dark:bg-neutral-900 shadow-xl p-5 text-sm space-y-4" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center gap-2">
          <h2 className="text-base font-semibold flex-1">Importeren</h2>
          <button className={button} onClick={onOpenSettings}>Bronnen instellen…</button>
          <button className="text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200" onClick={onClose} title="Esc">✕</button>
        </div>
        <p className="text-xs text-neutral-500">
          Eerst een plan: welke bestanden nieuw zijn, waar ze komen, en wat de bibliotheek al heeft (op inhoud, onder welke naam ook). Er verandert nog niets.
        </p>
        {errors[""] && <div className="text-red-600 text-xs">{errors[""]}</div>}
        {found !== null && shown.length === 0 && (
          <div className="text-neutral-500">Nog geen bronnen: voeg in de instellingen een aflevermap toe, of steek een kaart in.</div>
        )}
        {shown.map((f) => {
          const plan = plans[f.path];
          const progress = busy[f.path];
          const err = errors[f.path] ?? f.problem;
          return (
            <div key={f.kind + f.path} className="rounded border border-neutral-200 dark:border-neutral-800 p-3 space-y-1.5">
              <div className="flex items-center gap-2">
                <span className="text-xs text-neutral-400 w-14">{f.kind === "drop" ? "submap" : f.kind === "synced" ? "sync" : "kaart"}</span>
                <span className="font-medium">{baseName(f.path)}</span>
                {f.camera && <span className="text-xs text-neutral-500">{f.camera}</span>}
                <span className="flex-1" />
                <span className="text-xs text-neutral-500">
                  naar <span className="font-mono">{f.name || "?"}</span> · {f.known ? layoutText(f.layout) : "nieuwe bron, met dagmap"}
                </span>
              </div>
              <div className="flex items-center gap-2">
                <button className={button} disabled={!!progress || !!f.problem || f.kind === "synced"} onClick={() => makePlan(f.path)}>
                  {plan ? "Opnieuw plannen" : "Plan maken"}
                </button>
                <button className={button} disabled title="importeren komt in een volgende versie (F018/S05)">Importeren</button>
                {progress && <span className="text-xs text-neutral-500">{progress === true ? "bestanden zoeken…" : progressText(progress)}</span>}
                {f.kind === "synced" && <span className="text-xs text-neutral-500">gesynchroniseerde mappen komen in een volgende stap</span>}
              </div>
              {err && <div className="text-xs text-amber-600">{err}</div>}
              {plan && (
                <div className="text-xs space-y-0.5">
                  <div>{planSummary(plan)}{plan.excluded > 0 ? ` · ${plan.excluded} uitgesloten` : ""}</div>
                  {Object.keys(plan.months).length > 0 && <div className="text-neutral-500">{monthsText(plan.months)}</div>}
                  {datesText(plan.not_from_file) && <div className="text-amber-600">{datesText(plan.not_from_file)}</div>}
                  {plan.problems.slice(0, 5).map((p) => <div key={p} className="text-red-600">{p}</div>)}
                  {plan.files.length > 0 && (
                    <button className="text-neutral-500 hover:underline" onClick={() => setOpen(open === f.path ? null : f.path)}>
                      {open === f.path ? "▾" : "▸"} de bestanden
                    </button>
                  )}
                  {open === f.path && (
                    <ul className="max-h-64 overflow-auto font-mono space-y-0.5 select-text">
                      {plan.files.slice(0, 500).map((x) => (
                        <li key={x.src} className={x.action.what === "duplicate" ? "text-neutral-400" : x.action.what === "conflict" ? "text-amber-600" : ""} title={actionText(x.action)}>
                          {baseName(x.src)} → {x.action.what === "duplicate" ? "al in de bibliotheek" : x.target}
                        </li>
                      ))}
                      {plan.files.length > 500 && <li className="text-neutral-400">… en nog {plan.files.length - 500}</li>}
                    </ul>
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
