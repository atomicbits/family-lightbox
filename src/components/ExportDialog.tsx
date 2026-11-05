import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { exportPlanText, presetText, shortFolder, type ExportPlan, type Preset } from "../lib/exporter";

type Props = { list: { id: string; name: string }; onStart: (preset: string, dest: string | null) => void; onClose: () => void };

/** Exporting a list (F024/S03): a preset, the folder, what it makes; Lightroom's edits aren't in it. */
export default function ExportDialog({ list, onStart, onClose }: Props) {
  const [presets, setPresets] = useState<Preset[]>([]);
  const [preset, setPreset] = useState("fotoboek-a4");
  const [plan, setPlan] = useState<ExportPlan | null>(null);
  const [folder, setFolder] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => { invoke<Preset[]>("export_presets").then(setPresets).catch((e) => setError(String(e))); }, []);
  useEffect(() => {
    setPlan(null);
    invoke<ExportPlan>("export_plan", { list: list.id, preset }).then(setPlan).catch((e) => setError(String(e)));
  }, [list.id, preset]);
  const chosen = presets.find((p) => p.id === preset);
  const dest = folder ?? plan?.dest ?? null;
  const choose = async () => {
    const f = await open({ directory: true, title: "Map voor de export" });
    if (typeof f === "string") setFolder(f);
  };
  return (
    <div className="absolute inset-0 z-40 bg-black/40 flex items-center justify-center p-4" onClick={onClose}>
      <div className="w-[32rem] max-w-full rounded-lg bg-white dark:bg-neutral-900 shadow-xl p-5 text-sm space-y-3" onClick={(e) => e.stopPropagation()}>
        <h2 className="text-base font-semibold">{list.name} exporteren</h2>
        <div className="space-y-1">
          {presets.map((p) => (
            <label key={p.id} className="flex items-start gap-2">
              <input type="radio" name="preset" checked={preset === p.id} onChange={() => setPreset(p.id)} className="mt-1" />
              <span>
                {presetText(p)}
                <span className="block text-xs text-neutral-500">{p.what}</span>
              </span>
            </label>
          ))}
        </div>
        {plan && chosen && <p>{exportPlanText(plan, chosen)}</p>}
        <div className="flex items-center gap-2 text-xs">
          <span className="text-neutral-500">Naar</span>
          <span className="font-mono flex-1 truncate" title={dest ?? undefined}>{dest ? (folder ? dest : `Exports op de NAS: ${shortFolder(dest)}`) : "Exports op de NAS is niet verbonden: kies een map"}</span>
          <button className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700" onClick={choose}>Kies een map…</button>
        </div>
        <p className="text-xs text-neutral-500">
          Uit de originelen, niet de previews; wat niet op deze Mac staat, wordt van de NAS gelezen. Bewerkingen uit Lightroom zitten er niet in: exporteer bewerkte foto's vanuit Lightroom (de collectie van deze lijst).
          Je mag dit venster sluiten en de app herstarten; de voortgang staat onderaan.
        </p>
        {error && <p className="text-red-600 text-xs">{error}</p>}
        <div className="flex justify-end gap-2">
          <button className="px-3 py-1.5 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800" onClick={onClose}>Annuleren</button>
          <button className="px-3 py-1.5 rounded bg-neutral-900 text-white dark:bg-white dark:text-neutral-900 disabled:opacity-40" disabled={!plan || !dest} onClick={() => onStart(preset, folder)}>
            Exporteren
          </button>
        </div>
      </div>
    </div>
  );
}
