import { useEffect, useState } from "react";
import { config, type Found } from "../lib/api";
import { invoke } from "@tauri-apps/api/core";
import { actionText, datesText, monthsText, planSummary, progressText, syncedText } from "../lib/importplan";
import { baseName, layoutText } from "../lib/sources";
import { doneText, importLine, pendingText, type Importer } from "../lib/importjob";
import { syncLine, type Sync } from "../lib/sync";
import Progress from "./Progress";
import { t } from "../i18n";
import type { Plans } from "../lib/importplans";

type Props = { importer: Importer; planner: Plans; sync: Sync; onClose: () => void; onOpenSettings: () => void };

const button = "px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-xs disabled:opacity-40";

/** The import (F018/S04, S05): the sources found, per source the plan of what an import would do,
 * and *Importeren*, which copies to the NAS and only then tidies the source. */
export default function ImportView({ importer, planner, sync, onClose, onOpenSettings }: Props) {
  const [startError, setStartError] = useState<string | null>(null);
  // per source what a click on Importeren did or why it can't (F018/S05): never a silent button
  const [notes, setNotes] = useState<Record<string, string>>({});
  const note = (path: string, text: string | null) =>
    setNotes((n) => { const m = { ...n }; if (text) m[path] = text; else delete m[path]; return m; });
  const [found, setFound] = useState<Found[] | null>(null);
  // the plans are the app's, so they go on and stay while this view is closed
  const { plans, busy, make: makePlan } = planner;
  const [listError, setListError] = useState<string | null>(null);
  const errors: Record<string, string> = { ...planner.errors, ...(listError ? { "": listError } : {}) };
  const [open, setOpen] = useState<string | null>(null);

  // a card shows as soon as it is mounted (S07): looked for again every few seconds while open
  useEffect(() => {
    const timer = setInterval(() => {
      config.importSources().then((f) => setFound((old) => (JSON.stringify(old) === JSON.stringify(f) ? old : f))).catch(() => {});
    }, 4000);
    return () => clearInterval(timer);
  }, []);
  const eject = (path: string) => {
    invoke("eject_card", { path })
      .then(() => { note(path, null); config.importSources().then(setFound).catch(() => {}); })
      .catch((e) => note(path, t().imports.view.notEjected(String(e))));
  };
  useEffect(() => {
    config.importSources().then(setFound).catch((e) => setListError(String(e)));
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") { e.preventDefault(); onClose(); } };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const start = (f: Found) => {
    setStartError(null);
    const plan = plans[f.path];
    if (!plan) {
      note(f.path, t().imports.view.planFirst);
      makePlan(f.path);
      return;
    }
    if (plan.copy_files + plan.duplicates === 0) return note(f.path, t().imports.view.nothingToImport);
    note(f.path, null);
    importer.start(f.path, f.name).catch((e) => note(f.path, String(e)));
  };

  const shown = (found ?? []).filter((f) => !f.ignore);
  const tx = t().imports.view;

  return (
    <div className="absolute inset-0 z-30 bg-black/40 flex items-center justify-center p-4" onClick={onClose}>
      <div className="w-[44rem] max-h-full overflow-y-auto rounded-lg bg-white dark:bg-neutral-900 shadow-xl p-5 text-sm space-y-4" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center gap-2">
          <h2 className="text-base font-semibold flex-1">{tx.title}</h2>
          <button className={button} onClick={onOpenSettings}>{tx.setUpSources}</button>
          <button className="text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200" onClick={onClose} title="Esc">✕</button>
        </div>
        <p className="text-xs text-neutral-500">
          {tx.intro}{" "}
          {tx.closeBefore}<span className="text-blue-600">{tx.title}</span>{tx.closeAfter}
        </p>
        {errors[""] && <div className="text-red-600 text-xs">{errors[""]}</div>}
        {sync.state?.running && (
          <Progress
            tone="text-neutral-500"
            text={tx.syncRunning(syncLine(sync.state)?.text ?? tx.busy)}
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
            <span className="flex-1">{tx.interrupted(pendingText(j))}</span>
            <button className={button} disabled={!!importer.running} onClick={() => importer.resume(j).catch((e) => setStartError(String(e)))}>{tx.resume}</button>
          </div>
        ))}
        {found !== null && shown.length === 0 && (
          <div className="text-neutral-500">{tx.noSources}</div>
        )}
        {shown.map((f) => {
          const plan = plans[f.path];
          const progress = busy[f.path];
          const err = errors[f.path] ?? f.problem;
          return (
            <div key={f.kind + f.path} className="rounded border border-neutral-200 dark:border-neutral-800 p-3 space-y-1.5">
              <div className="flex items-center gap-2">
                <span className="text-xs text-neutral-400 w-14">{t().imports.kind[f.kind === "drop" ? "drop" : f.kind === "synced" ? "synced" : "card"]}</span>
                <span className="font-medium">{baseName(f.path)}</span>
                {f.camera && <span className="text-xs text-neutral-500">{f.camera}</span>}
                <span className="flex-1" />
                <span className="text-xs text-neutral-500">
                  {tx.to} <span className="font-mono">{f.name || "?"}</span>
                  {f.videos ? <> {tx.and} <span className="font-mono">{f.videos}</span></> : null} · {f.known ? layoutText(f.layout) : tx.newSource}
                </span>
              </div>
              <div className="flex items-center gap-2">
                <button className={button} disabled={!!progress || !!f.problem} onClick={() => makePlan(f.path)}>
                  {plan ? tx.replan : tx.makePlan}
                </button>
                <button
                  className={button}
                  disabled={!!importer.running || !!progress || !!f.problem || !!sync.state?.running}
                  title={
                    sync.state?.running ? tx.tipSyncRunning
                    : !plan ? tx.tipPlanFirst
                    : f.kind === "synced" ? tx.tipSynced
                    : f.kind === "card" ? tx.tipCard
                    : tx.tipDrop
                  }
                  onClick={() => start(f)}
                >
                  {tx.import}
                </button>
                {f.kind === "synced" && <span className="text-xs text-neutral-500">{tx.syncedOnly}</span>}
                {f.kind === "card" && (
                  <>
                    <span className="text-xs text-neutral-500 flex-1">{tx.cardOnly}</span>
                    <button className={button} disabled={importer.running?.path === f.path || !!progress} title={tx.tipEject} onClick={() => eject(f.path)}>
                      {tx.eject}
                    </button>
                  </>
                )}
              </div>
              {progress && (
                <Progress
                  text={progress === true ? tx.planFinding : tx.planReading(progressText(progress))}
                  done={progress === true ? undefined : progress.bytes_done}
                  total={progress === true ? undefined : progress.bytes_total}
                />
              )}
              {importer.running?.path === f.path && (
                <Progress
                  text={importLine(importer.running.name, importer.progress, importer.planProgress)}
                  done={importer.progress ? (importer.progress.bytes_total ? importer.progress.bytes_done : importer.progress.done) : importer.planProgress?.bytes_done}
                  total={importer.progress ? (importer.progress.bytes_total || importer.progress.total) : importer.planProgress?.bytes_total}
                />
              )}
              {notes[f.path] && !progress && <div className="text-xs text-amber-600">{notes[f.path]}</div>}
              {err && <div className="text-xs text-amber-600">{err}</div>}
              {plan && (
                <div className="text-xs space-y-0.5">
                  <div>{planSummary(plan)}{plan.excluded > 0 ? tx.excluded(plan.excluded) : ""}</div>
                  {syncedText(plan) && <div className="text-neutral-500">{syncedText(plan)}</div>}
                  {Object.keys(plan.months).length > 0 && <div className="text-neutral-500">{monthsText(plan.months)}</div>}
                  {datesText(plan.not_from_file) && <div className="text-amber-600">{datesText(plan.not_from_file)}</div>}
                  {plan.problems.slice(0, 5).map((p) => <div key={p} className="text-red-600">{p}</div>)}
                  {plan.files.length > 0 && (
                    <button className="text-neutral-500 hover:underline" onClick={() => setOpen(open === f.path ? null : f.path)}>
                      {open === f.path ? "▾" : "▸"} {tx.files}
                    </button>
                  )}
                  {open === f.path && (
                    <ul className="max-h-64 overflow-auto font-mono space-y-0.5 select-text">
                      {plan.files.slice(0, 500).map((x) => (
                        <li key={x.src} className={x.action.what === "duplicate" ? "text-neutral-400" : x.action.what === "conflict" ? "text-amber-600" : ""} title={actionText(x.action)}>
                          {baseName(x.src)} → {x.action.what === "duplicate" ? tx.inLibrary : x.target}
                        </li>
                      ))}
                      {plan.files.length > 500 && <li className="text-neutral-400">{tx.andMore(plan.files.length - 500)}</li>}
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
