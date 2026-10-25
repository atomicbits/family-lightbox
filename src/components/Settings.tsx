import { useEffect, useState } from "react";
import { config, mountLabel, nas, type DeviceState, type NasMounts, type Settings as S, type SettingsState, type Shares } from "../lib/api";
import { hostProblem, shareProblem } from "../lib/device";

type Props = { device: DeviceState | null; onClose: () => void };

const SHARE_LABELS: [keyof Shares, string][] = [
  ["library", "Originelen"],
  ["previews", "Previews"],
  ["lists", "Lijsten"],
  ["exports", "Exports"],
];

/** Settings panel: the device (read-only) and the NAS host and share names (editable). */
export default function Settings({ device, onClose }: Props) {
  const [state, setState] = useState<SettingsState | null>(null);
  const [draft, setDraft] = useState<S | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [mounts, setMounts] = useState<NasMounts | null>(null);

  const load = () =>
    config.settings().then((s) => { setState(s); setDraft(s.settings); return nas.mounts().then(setMounts); });
  useEffect(() => { load(); }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  if (!state || !draft) return null;

  const problems = [hostProblem(draft.nas_host), ...SHARE_LABELS.map(([k]) => shareProblem(draft.shares[k]))].filter(
    (p): p is string => p !== null,
  );
  const changed = JSON.stringify(draft) !== JSON.stringify(state.settings);

  const save = () =>
    config
      .saveSettings(draft)
      .then(() => { setMessage("bewaard"); return load(); })
      .catch((e) => setMessage(String(e)));

  const input = "w-full px-2 py-1 rounded border border-neutral-300 dark:border-neutral-700 bg-transparent";

  return (
    <div className="absolute inset-0 z-30 bg-black/40 grid place-items-center" onClick={onClose}>
      <div
        className="w-[30rem] rounded-lg bg-white dark:bg-neutral-900 shadow-xl p-5 text-sm space-y-4"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="text-base font-semibold">Instellingen</h2>

        <section className="space-y-1">
          <h3 className="font-medium">Dit toestel</h3>
          {device?.device ? (
            <div>
              <span className="font-mono">{device.device.name}</span>
              <span className="text-neutral-500"> · van {device.device.owner}</span>
            </div>
          ) : (
            <div className="text-neutral-500">nog geen naam</div>
          )}
          {device?.error && <div className="text-red-600 whitespace-pre-wrap">{device.error}</div>}
          {device && <div className="text-xs text-neutral-400 font-mono">{device.path}</div>}
        </section>

        <section className="space-y-2">
          <h3 className="font-medium">NAS</h3>
          {state.error && (
            <div className="text-red-600 whitespace-pre-wrap">
              {state.error}
              {"\n"}De standaardwaarden zijn in gebruik; het bestand wordt pas overschreven als je bewaart.
            </div>
          )}
          <label className="block space-y-1">
            <span>Naam van de NAS</span>
            <input className={input} value={draft.nas_host} onChange={(e) => setDraft({ ...draft, nas_host: e.target.value })} />
          </label>
          <div className="grid grid-cols-2 gap-2">
            {SHARE_LABELS.map(([k, label]) => (
              <label key={k} className="block space-y-1">
                <span>{label}</span>
                <input
                  className={input}
                  value={draft.shares[k]}
                  onChange={(e) => setDraft({ ...draft, shares: { ...draft.shares, [k]: e.target.value } })}
                />
              </label>
            ))}
          </div>
          {problems.map((p) => <div key={p} className="text-amber-600">{p}</div>)}
          {mounts && (
            <div className="space-y-0.5">
              <div className="text-neutral-500">Gevonden op deze Mac</div>
              {SHARE_LABELS.map(([k, label]) => {
                const where = mountLabel(mounts[k]);
                return (
                  <div key={k} className="flex gap-2">
                    <span className="w-24 shrink-0">{label}</span>
                    <span className={where ? "font-mono text-xs leading-5" : "text-neutral-400"}>
                      {where ?? "niet verbonden"}
                      {mounts[k].length > 1 && ` (+${mounts[k].length - 1})`}
                    </span>
                  </div>
                );
              })}
            </div>
          )}
          <div className="text-xs text-neutral-400 font-mono">{state.path}</div>
        </section>

        <div className="flex items-center gap-2">
          {message && <span className="text-neutral-500 whitespace-pre-wrap">{message}</span>}
          <span className="flex-1" />
          <button className="px-3 py-1.5 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800" onClick={onClose}>
            Sluiten
          </button>
          <button
            className="px-3 py-1.5 rounded bg-neutral-900 text-white dark:bg-white dark:text-neutral-900 disabled:opacity-40"
            disabled={(!changed && !state.error) || problems.length > 0}
            onClick={save}
          >
            Bewaar
          </button>
        </div>
      </div>
    </div>
  );
}
