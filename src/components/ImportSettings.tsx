import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { config, type Found, type Import } from "../lib/api";
import { NO_IMPORT, baseName, layoutText, patterns, withSource } from "../lib/sources";

type Props = {
  value: Import | undefined;
  onChange: (i: Import) => void;
  /** Changes after a save, so what is found is looked up again. */
  version: number;
};

const small = "px-1.5 py-0.5 rounded border border-neutral-300 dark:border-neutral-700 bg-transparent text-xs";
const button = "px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 text-xs";

/** The settings' *Importeren* (F018/S03): drop folders and synced folders on this Mac, and per
 * source found its name in the library, whether to skip it, and its exclusions. */
export default function ImportSettings({ value, onChange, version }: Props) {
  const imp = value ?? NO_IMPORT;
  const [found, setFound] = useState<Found[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    config.importSources().then(setFound).catch((e) => setError(String(e)));
  }, [version]);

  const pick = async (title: string) => {
    const f = await open({ directory: true, title });
    return typeof f === "string" ? f : null;
  };
  const addDrop = async () => {
    const f = await pick("Map waarin nieuwe foto's per bron in een submap komen");
    if (f && !imp.drop_folders.includes(f)) onChange({ ...imp, drop_folders: [...imp.drop_folders, f] });
  };
  const addSynced = async () => {
    const f = await pick("Map die een andere app gesynchroniseerd houdt");
    if (f && !imp.synced.some((s) => s.path === f)) onChange({ ...imp, synced: [...imp.synced, { path: f, photos: "", videos: "" }] });
  };
  const folderOf = (f: Found) => baseName(f.path);

  return (
    <section className="space-y-2">
      <h3 className="font-medium">Importeren</h3>
      <p className="text-xs text-neutral-500">
        Waar nieuwe foto's binnenkomen op deze laptop. Elke submap van een aflevermap is een bron; een gesynchroniseerde map (bv. van een telefoon) wordt nooit gewijzigd. Kaarten worden vanzelf herkend.
      </p>

      <div className="space-y-1">
        <div className="flex items-center gap-2">
          <span className="text-xs text-neutral-500 flex-1">Aflevermappen</span>
          <button className={button} onClick={addDrop}>Toevoegen…</button>
        </div>
        {imp.drop_folders.map((d) => (
          <div key={d} className="flex items-center gap-2 text-xs font-mono">
            <span className="flex-1 truncate" title={d}>{d}</span>
            <button className="text-neutral-400 hover:text-red-600" title="weghalen (de map zelf blijft)" onClick={() => onChange({ ...imp, drop_folders: imp.drop_folders.filter((x) => x !== d) })}>✕</button>
          </div>
        ))}
      </div>

      <div className="space-y-1">
        <div className="flex items-center gap-2">
          <span className="text-xs text-neutral-500 flex-1">Gesynchroniseerde mappen</span>
          <button className={button} onClick={addSynced}>Toevoegen…</button>
        </div>
        {imp.synced.map((s, i) => (
          <div key={s.path} className="space-y-1 rounded border border-neutral-200 dark:border-neutral-800 p-1.5">
            <div className="flex items-center gap-2 text-xs font-mono">
              <span className="flex-1 truncate" title={s.path}>{s.path}</span>
              <button className="text-neutral-400 hover:text-red-600" title="weghalen (de map zelf blijft)" onClick={() => onChange({ ...imp, synced: imp.synced.filter((_, j) => j !== i) })}>✕</button>
            </div>
            <div className="flex gap-2 text-xs items-center">
              <span className="text-neutral-500">foto's naar</span>
              <input className={`${small} font-mono w-36`} value={s.photos} placeholder="bron" onChange={(e) => onChange({ ...imp, synced: imp.synced.map((x, j) => (j === i ? { ...x, photos: e.target.value } : x)) })} />
              <span className="text-neutral-500">video's naar</span>
              <input className={`${small} font-mono w-36`} value={s.videos} placeholder="bron" onChange={(e) => onChange({ ...imp, synced: imp.synced.map((x, j) => (j === i ? { ...x, videos: e.target.value } : x)) })} />
            </div>
          </div>
        ))}
      </div>

      {error && <div className="text-red-600 text-xs">{error}</div>}
      {found && found.length > 0 && (
        <div className="space-y-1">
          <span className="text-xs text-neutral-500">Gevonden bronnen</span>
          {found.map((f) => {
            const folder = folderOf(f);
            const set = imp.sources[folder];
            return (
              <div key={f.kind + f.path} className="text-xs space-y-0.5 rounded border border-neutral-200 dark:border-neutral-800 p-1.5">
                <div className="flex items-center gap-2">
                  <span className="text-neutral-400 w-14">{f.kind === "drop" ? "submap" : f.kind === "synced" ? "sync" : "kaart"}</span>
                  <span className="font-mono truncate flex-1" title={f.path}>{f.kind === "card" ? `${folder}${f.camera ? ` · ${f.camera}` : ""}` : folder}</span>
                  <span className="text-neutral-500" title={f.known ? "zo staat deze bron in de bibliotheek" : "nieuw in de bibliotheek"}>
                    {f.known ? layoutText(f.layout) : "nieuw, met dagmap"}
                  </span>
                </div>
                {f.kind === "drop" && (
                  <div className="flex items-center gap-2 pl-16 flex-wrap">
                    <span className="text-neutral-500">bron</span>
                    <input className={`${small} font-mono w-36`} value={set?.name ?? ""} placeholder={folder} onChange={(e) => onChange(withSource(imp, folder, { name: e.target.value.trim() }))} />
                    <label className="flex items-center gap-1"><input type="checkbox" checked={set?.ignore ?? false} onChange={(e) => onChange(withSource(imp, folder, { ignore: e.target.checked }))} />negeren</label>
                    <label className="flex items-center gap-1" title="na een import die op de NAS nagekeken is"><input type="checkbox" checked={set?.delete_imported ?? false} onChange={(e) => onChange(withSource(imp, folder, { delete_imported: e.target.checked }))} />_imported leegmaken</label>
                    <input className={`${small} font-mono flex-1 min-w-24`} defaultValue={(set?.exclude ?? []).join(", ")} placeholder="uitsluiten, bv. *-mask.png" onBlur={(e) => onChange(withSource(imp, folder, { exclude: patterns(e.target.value) }))} />
                  </div>
                )}
                {f.kind !== "drop" && <div className="pl-16 text-neutral-500">naar <span className="font-mono">{f.name || "?"}</span>{f.videos ? <> en <span className="font-mono">{f.videos}</span></> : null}</div>}
                {f.problem && <div className="pl-16 text-amber-600">{f.problem}</div>}
              </div>
            );
          })}
        </div>
      )}
    </section>
  );
}
