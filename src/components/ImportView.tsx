import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { config, type Found } from "../lib/api";
import { actionText, datesText, monthsText, planSummary, progressText, syncedText, type PlanProgress, type SourcePlan } from "../lib/importplan";
import { baseName, layoutText } from "../lib/sources";
import { doneText, importLine, pendingText, type Importer } from "../lib/importjob";
import { syncLine, type Sync } from "../lib/sync";
import Progress from "./Progress";

type Props = { importer: Importer; sync: Sync; onClose: () => void; onOpenSettings: () => void };

const button = "px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-xs disabled:opacity-40";

/** The import (F018/S04, S05): the sources found, per source the plan of what an import would do,
 * and *Importeren*, which copies to the NAS and only then tidies the source. */
export default function ImportView({ importer, sync, onClose, onOpenSettings }: Props) {
  const [startError, setStartError] = useState<string | null>(null);
  // per source what a click on Importeren did or why it can't (F018/S05): never a silent button
  const [notes, setNotes] = useState<Record<string, string>>({});
  const note = (path: string, text: string | null) =>
    setNotes((n) => { const m = { ...n }; if (text) m[path] = text; else delete m[path]; return m; });
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

  const start = (f: Found) => {
    setStartError(null);
    const plan = plans[f.path];
    if (!plan) {
      note(f.path, "eerst het plan: kijk het na en klik dan nog eens op Importeren");
      makePlan(f.path);
      return;
    }
    if (plan.copy_files + plan.duplicates === 0) return note(f.path, "niets nieuws om te importeren");
    note(f.path, null);
    importer.start(f.path, f.name).catch((e) => note(f.path, String(e)));
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
        {sync.state?.running && (
          <Progress
            tone="text-neutral-500"
            text={`er loopt een synchronisatie met de NAS (${syncLine(sync.state)?.text ?? "bezig"}); importeren kan daarna`}
            done={sync.state.phase === "copying" ? sync.state.bytes_done : sync.state.phase === "manifests" ? sync.state.files_done : undefined}
            total={sync.state.phase === "copying" ? sync.state.bytes_total : sync.state.phase === "manifests" ? sync.state.files_total : undefined}
          />
        )}
        {importer.last && !importer.running && (
          <div className={"text-xs " + (importer.last.error || importer.last.failed ? "text-amber-600" : "text-green-700 dark:text-green-500")}>
            {doneText(importer.last)}
            {importer.last.problems.slice(0, 5).map((p) => <div key={p} className="text-amber-600">{p}</div>)}
          </div>
        )}
        {startError && <div className="text-xs text-red-600">{startError}</div>}
        {importer.pending.filter((j) => j.source !== importer.running?.path).map((j) => (
          <div key={j.id} className="flex items-center gap-2 text-xs rounded border border-amber-300 dark:border-amber-800 p-2">
            <span className="flex-1">Een import werd onderbroken: {pendingText(j)}. De originelen staan nog in de bron tot alles op de NAS staat.</span>
            <button className={button} disabled={!!importer.running} onClick={() => importer.resume(j).catch((e) => setStartError(String(e)))}>Doorgaan</button>
          </div>
        ))}
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
                  naar <span className="font-mono">{f.name || "?"}</span>
                  {f.videos ? <> en <span className="font-mono">{f.videos}</span></> : null} · {f.known ? layoutText(f.layout) : "nieuwe bron, met dagmap"}
                </span>
              </div>
              <div className="flex items-center gap-2">
                <button className={button} disabled={!!progress || !!f.problem} onClick={() => makePlan(f.path)}>
                  {plan ? "Opnieuw plannen" : "Plan maken"}
                </button>
                <button
                  className={button}
                  disabled={!!importer.running || !!progress || !!f.problem || !!sync.state?.running}
                  title={
                    sync.state?.running ? "er loopt een synchronisatie: importeren kan daarna"
                    : !plan ? "maakt eerst het plan, om na te kijken"
                    : f.kind === "synced" ? "kopieert naar de NAS; de gesynchroniseerde map zelf blijft zoals ze is"
                    : "kopieert naar de NAS, en verplaatst daarna de originelen naar _imported"
                  }
                  onClick={() => start(f)}
                >
                  Importeren
                </button>
                {f.kind === "synced" && <span className="text-xs text-neutral-500">alleen kopiëren: de map zelf blijft zoals ze is</span>}
              </div>
              {progress && (
                <Progress
                  text={progress === true ? "plan maken: bestanden zoeken…" : `plan maken: bestanden lezen · ${progressText(progress)}`}
                  done={progress === true ? undefined : progress.bytes_done}
                  total={progress === true ? undefined : progress.bytes_total}
                />
              )}
              {importer.running?.path === f.path && (
                <Progress
                  text={importLine(importer.running.name, importer.progress, importer.planProgress)}
                  done={importer.progress?.done ?? importer.planProgress?.bytes_done}
                  total={importer.progress?.total ?? importer.planProgress?.bytes_total}
                />
              )}
              {notes[f.path] && !progress && <div className="text-xs text-amber-600">{notes[f.path]}</div>}
              {err && <div className="text-xs text-amber-600">{err}</div>}
              {plan && (
                <div className="text-xs space-y-0.5">
                  <div>{planSummary(plan)}{plan.excluded > 0 ? ` · ${plan.excluded} uitgesloten` : ""}</div>
                  {syncedText(plan) && <div className="text-neutral-500">{syncedText(plan)}</div>}
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
