import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { lastUsedLabel, sourceLabel, type CameraCount } from "../lib/filter";
import { renameSummary, toggleSource, type RenamePlan } from "../lib/rename";
import Progress from "./Progress";
import { renamingText, type Renamer } from "../lib/renamer";
import { count, t } from "../i18n";

const button = "px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-xs disabled:opacity-40";
const small = "px-1.5 py-0.5 rounded border border-neutral-300 dark:border-neutral-700 bg-transparent text-xs font-mono";

/** The settings' *Bronnen* tab (F022/S05): the library's sources, and renaming or merging some of
 * them, with the plan first and a snapshot of today confirmed before anything moves. */
export default function SourcesTab({ renamer }: { renamer: Renamer }) {
  const [sources, setSources] = useState<CameraCount[] | null>(null);
  const [chosen, setChosen] = useState<string[]>([]);
  const [to, setTo] = useState("");
  const [plan, setPlan] = useState<RenamePlan | null>(null);
  const [planning, setPlanning] = useState(false);
  const [snapshot, setSnapshot] = useState(false);
  const running = renamer.running;
  const [message, setMessage] = useState<{ text: string; error: boolean } | null>(null);
  const [pending, setPending] = useState<[string[], string] | null>(null);

  const load = () => {
    invoke<CameraCount[]>("sources").then((s) => setSources(s.filter(([name]) => name !== ""))).catch(() => setSources([]));
    invoke<[string[], string] | null>("rename_pending").then(setPending).catch(() => {});
  };
  useEffect(() => { load(); }, []);
  // the end of a rename, also one started before this tab opened
  useEffect(() => {
    const d = renamer.last;
    if (!d) return;
    setPlan(null);
    setSnapshot(false);
    setMessage(d.error ? { text: t().settings.sources.renameStopped(d.to, d.error), error: true } : { text: t().settings.sources.renamed(d.to), error: false });
    if (!d.error) setChosen([]);
    load();
  }, [renamer.last]);

  const makePlan = (from: string[], target: string) => {
    setPlanning(true);
    setMessage(null);
    invoke<RenamePlan>("rename_plan", { from, to: target })
      .then(setPlan)
      .catch((e) => setMessage({ text: String(e), error: true }))
      .finally(() => setPlanning(false));
  };
  const start = (from: string[], target: string) => {
    setMessage(null);
    renamer.start(from, target, snapshot, plan?.months.length ?? 0).catch((e) => setMessage({ text: String(e), error: true }));
  };

  const x = t().settings.sources;
  return (
    <section className="space-y-3 text-xs">
      <p className="text-neutral-500">{x.intro}</p>
      {pending && (
        <div className="rounded border border-amber-300 dark:border-amber-800 p-2 flex items-center gap-2">
          <span className="flex-1">{x.interruptedBefore}<span className="font-mono">{pending[1]}</span>{x.interruptedAfter}</span>
          <label className="flex items-center gap-1"><input type="checkbox" checked={snapshot} onChange={(e) => setSnapshot(e.target.checked)} />{x.snapshotToday}</label>
          <button className={button} disabled={!snapshot || !!running} onClick={() => start(pending[0], pending[1])}>{x.continue}</button>
        </div>
      )}
      <div className="max-h-56 overflow-y-auto rounded border border-neutral-200 dark:border-neutral-800">
        {sources === null && <div className="p-2 text-neutral-400">{x.loading}</div>}
        {sources?.map(([name, n, last]) => (
          <label key={name} className="flex items-center gap-2 px-2 py-0.5 hover:bg-neutral-50 dark:hover:bg-neutral-800/50">
            <input type="checkbox" checked={chosen.includes(name)} onChange={() => { setChosen(toggleSource(chosen, name)); setPlan(null); }} />
            <span className="font-mono flex-1">{sourceLabel(name)}</span>
            <span className="text-neutral-400 tabular-nums">{lastUsedLabel(last)}</span>
            <span className="text-neutral-400 tabular-nums w-16 text-right">{count(n)}</span>
          </label>
        ))}
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-neutral-500">{chosen.length ? `${chosen.join(" + ")} →` : x.chooseFirst}</span>
        <input className={`${small} w-44`} value={to} placeholder={x.newName} onChange={(e) => { setTo(e.target.value); setPlan(null); }} />
        <button className={button} disabled={!chosen.length || !to.trim() || planning || !!running} onClick={() => makePlan(chosen, to.trim())}>
          {planning ? x.planning : x.makePlan}
        </button>
      </div>
      {planning && <Progress text={x.readingPlan} />}
      {plan && (
        <div className="rounded border border-neutral-200 dark:border-neutral-800 p-2 space-y-1">
          <div>{renameSummary(plan)}</div>
          <div className="text-neutral-500">{x.backupNote}</div>
          <ul className="max-h-40 overflow-y-auto text-neutral-500 font-mono">
            {plan.months.map((m) => (
              <li key={m.month}>
                {x.month(m.month, m.moves.length, m.duplicates.length, m.conflicts)}
              </li>
            ))}
          </ul>
          <div className="flex flex-wrap items-center gap-2 pt-1">
            <label className="flex items-center gap-1">
              <input type="checkbox" checked={snapshot} onChange={(e) => setSnapshot(e.target.checked)} />
              {x.snapshotMade}
            </label>
            <span className="flex-1" />
            <button className={button} disabled={!snapshot || !!running || plan.files === 0} onClick={() => start(plan.from, plan.to)}>{x.run}</button>
          </div>
        </div>
      )}
      {running && (
        <Progress text={x.running(renamingText(running))} done={running[0]} total={running[1]} />
      )}
      {message && <div className={message.error ? "text-red-600 whitespace-pre-wrap" : "text-green-700 dark:text-green-500"}>{message.text}</div>}
    </section>
  );
}
