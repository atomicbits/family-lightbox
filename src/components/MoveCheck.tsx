import { useState } from "react";
import { nas as nasApi, type DryCheck } from "../lib/api";
import { locationText } from "../lib/nas";

const LABEL: Record<string, string> = { library: "Originelen", previews: "Previews", lists: "Lijsten", exports: "Exports" };

/** Before moving the library to another share (F019/S04): what the app would find there, and which
 * devices still have an app that doesn't read `familylightbox.json`. Uses nothing, changes nothing. */
export default function MoveCheck() {
  const [open, setOpen] = useState(false);
  const [share, setShare] = useState("");
  const [result, setResult] = useState<DryCheck | null>(null);
  const [busy, setBusy] = useState(false);
  const run = () => {
    setBusy(true);
    nasApi.dryCheck(share.trim()).then(setResult).catch((e) => setResult({ problem: String(e), name: "", found: [], devices: [] })).finally(() => setBusy(false));
  };
  return (
    <div className="text-xs space-y-1">
      <button className="text-neutral-500 hover:underline" onClick={() => setOpen(!open)}>
        {open ? "▾" : "▸"} een verhuis naar een andere gedeelde map nakijken
      </button>
      {open && (
        <div className="space-y-1 pl-3">
          <div className="flex gap-2 items-center">
            <input className="px-1.5 py-0.5 rounded border border-neutral-300 dark:border-neutral-700 bg-transparent font-mono w-40" value={share} placeholder="gedeelde map" onChange={(e) => setShare(e.target.value)} />
            <button className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 disabled:opacity-40" disabled={busy || !share.trim()} onClick={run}>
              {busy ? "Nakijken…" : "Nakijken"}
            </button>
          </div>
          {result?.problem && <div className="text-amber-600">{result.problem}</div>}
          {result && !result.problem && (
            <>
              <div className="text-neutral-500">{result.name ? `${result.name}: ` : ""}wat de app daar zou vinden</div>
              {result.found.map((f) => (
                <div key={f.what} className="flex gap-2">
                  <span className={f.ok ? "text-green-600" : "text-amber-600"}>{f.ok ? "✓" : "!"}</span>
                  <span className="w-20">{LABEL[f.what] ?? f.what}</span>
                  <span className="font-mono w-36 truncate">{locationText(f.location)}</span>
                  <span className="text-neutral-500">{f.note}</span>
                </div>
              ))}
              {result.devices.map((d) => (
                <div key={d.name} className="flex gap-2">
                  <span className={d.ok ? "text-green-600" : "text-amber-600"}>{d.ok ? "✓" : "!"}</span>
                  <span className="font-mono">{d.name}</span>
                  <span className="text-neutral-500">{d.owner} · {d.app ? `versie ${d.app}` : "een versie van vóór 0.5"}{d.ok ? "" : ": moet eerst bijwerken"}</span>
                </div>
              ))}
            </>
          )}
        </div>
      )}
    </div>
  );
}
