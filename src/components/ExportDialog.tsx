import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { customProblem, customText, exportPlanText, presetText, shortFolder, type Custom, type ExportPlan, type Preset } from "../lib/exporter";
import { t } from "../i18n";

type Props = { list: { id: string; name: string }; onStart: (preset: string, dest: string | null, custom: Custom | null) => void; onClose: () => void };

const field = "w-20 px-1.5 py-0.5 rounded border border-neutral-300 dark:border-neutral-700 bg-transparent font-mono text-xs";

/** Exporting a list (F024/S03): a preset, the folder, what it makes; Lightroom's edits aren't in it. */
export default function ExportDialog({ list, onStart, onClose }: Props) {
  const [presets, setPresets] = useState<Preset[]>([]);
  const [preset, setPreset] = useState("fotoboek-a4");
  const [plan, setPlan] = useState<ExportPlan | null>(null);
  const [folder, setFolder] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  // an own setting, by hand, behind its own button (Peter, 2026-11-05)
  const [own, setOwn] = useState<Custom | null>(null);
  const ownProblem = own ? customProblem(own) : null;
  const w = t().lists.export;
  useEffect(() => { invoke<Preset[]>("export_presets").then(setPresets).catch((e) => setError(String(e))); }, []);
  useEffect(() => {
    setPlan(null);
    if (own && customProblem(own)) return;
    invoke<ExportPlan>("export_plan", { list: list.id, preset, custom: own }).then(setPlan).catch((e) => setError(String(e)));
  }, [list.id, preset, own]);
  const fromList = presets.find((p) => p.id === preset);
  const chosen: Preset | undefined = own ? { id: "eigen", label: w.own, what: "", long: own.long, quality: own.quality / 100, dpi: own.dpi, originals: false } : fromList;
  const num = (v: string) => (v.trim() === "" ? NaN : Number(v));
  const dest = folder ?? plan?.dest ?? null;
  const choose = async () => {
    const f = await open({ directory: true, title: w.chooseTitle });
    if (typeof f === "string") setFolder(f);
  };
  return (
    <div className="absolute inset-0 z-40 bg-black/40 flex items-center justify-center p-4" onClick={onClose}>
      <div className="w-[32rem] max-w-full rounded-lg bg-white dark:bg-neutral-900 shadow-xl p-5 text-sm space-y-3" onClick={(e) => e.stopPropagation()}>
        <h2 className="text-base font-semibold">{w.title(list.name)}</h2>
        <div className="space-y-1">
          {presets.map((p) => (
            <label key={p.id} className="flex items-start gap-2">
              <input type="radio" name="preset" checked={!own && preset === p.id} onChange={() => { setOwn(null); setPreset(p.id); }} className="mt-1" />
              <span>
                {presetText(p)}
                <span className="block text-xs text-neutral-500">{p.what}</span>
              </span>
            </label>
          ))}
        </div>
        {!own ? (
          <button
            className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-xs"
            onClick={() => setOwn({ long: fromList?.long ?? 3000, quality: Math.round((fromList?.quality || 0.92) * 100), dpi: fromList?.dpi ?? 300 })}
          >
            {w.ownButton}
          </button>
        ) : (
          <div className="rounded border border-neutral-200 dark:border-neutral-800 p-2 space-y-1 text-xs">
            <div className="flex items-center gap-2">
              <input type="radio" checked readOnly />
              <span className="font-medium flex-1">{w.ownChosen(customText(own))}</span>
              <button className="text-neutral-500 hover:underline" onClick={() => setOwn(null)}>{w.remove}</button>
            </div>
            <div className="flex flex-wrap items-center gap-3 pl-5">
              <label className="flex items-center gap-1">{w.longSide} <input className={field} value={Number.isNaN(own.long) ? "" : own.long} onChange={(e) => setOwn({ ...own, long: num(e.target.value) })} /> px</label>
              <label className="flex items-center gap-1">{w.quality} <input className={field} value={Number.isNaN(own.quality) ? "" : own.quality} onChange={(e) => setOwn({ ...own, quality: num(e.target.value) })} /></label>
              <label className="flex items-center gap-1">{w.dpi} <input className={field} value={own.dpi ?? ""} placeholder={w.none} onChange={(e) => setOwn({ ...own, dpi: e.target.value.trim() === "" ? null : num(e.target.value) })} /></label>
            </div>
            {ownProblem && <div className="pl-5 text-amber-600">{ownProblem}</div>}
          </div>
        )}
        {plan && chosen && <p>{exportPlanText(plan, chosen)}</p>}
        <div className="flex items-center gap-2 text-xs">
          <span className="text-neutral-500">{w.to}</span>
          <span className="font-mono flex-1 truncate" title={dest ?? undefined}>{dest ? (folder ? dest : w.nasFolder(shortFolder(dest))) : w.nasMissing}</span>
          <button className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700" onClick={choose}>{w.choose}</button>
        </div>
        <p className="text-xs text-neutral-500">
          {w.from} {w.closeOk}
        </p>
        {error && <p className="text-red-600 text-xs">{error}</p>}
        <div className="flex justify-end gap-2">
          <button className="px-3 py-1.5 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800" onClick={onClose}>{w.cancel}</button>
          <button className="px-3 py-1.5 rounded bg-neutral-900 text-white dark:bg-white dark:text-neutral-900 disabled:opacity-40" disabled={!plan || !dest || !!ownProblem} onClick={() => onStart(own ? "eigen" : preset, folder, own)}>
            {w.start}
          </button>
        </div>
      </div>
    </div>
  );
}
