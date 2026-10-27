import { useEffect, useState } from "react";
import { accessLabel, anyMissing, config, mountLabel, registrationLabel, type DeviceState, type Settings as S, type SettingsState, type Shares } from "../lib/api";
import type { Nas } from "../lib/nas";
import type { Sync } from "../lib/sync";
import { outcomeText, type Index } from "../lib/indexer";
import { hostProblem, shareProblem } from "../lib/device";

type Props = { device: DeviceState | null; nas: Nas; sync: Sync; index: Index; onClose: () => void };

const SHARE_LABELS: [keyof Shares, string][] = [
  ["library", "Originelen"],
  ["previews", "Previews"],
  ["lists", "Lijsten"],
  ["exports", "Exports"],
];

/** Settings panel: the device (read-only) and the NAS host and share names (editable). */
export default function Settings({ device, nas, sync, index, onClose }: Props) {
  const [state, setState] = useState<SettingsState | null>(null);
  const [draft, setDraft] = useState<S | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const { mounts, attempts, check, busy: connecting } = nas;

  const load = () => config.settings().then((s) => { setState(s); setDraft(s.settings); });
  useEffect(() => { load(); nas.refresh(); }, []);

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
      .then(() => { setMessage("bewaard"); return load().then(nas.refresh); })
      .catch((e) => setMessage(String(e)));

  const connect = () => nas.connect(true);

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
          {check && device?.device && (
            <div className={check.registration.kind.endsWith("registered") ? "text-neutral-500" : "text-amber-600"}>
              {registrationLabel(check.registration)}
            </div>
          )}
          {device && <div className="text-xs text-neutral-400 font-mono">{device.path}</div>}
        </section>

        <section className="space-y-1">
          <div className="flex items-center gap-2">
            <h3 className="font-medium">Index</h3>
            <span className="flex-1" />
            <button
              className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 disabled:opacity-40"
              disabled={index.state?.running}
              title="de index opnieuw opbouwen uit de manifesten in de werkkopie"
              onClick={() => index.rebuild({ visible: true, onlyIfStale: false }).catch(() => {})}
            >
              {index.state?.running ? "Index bijwerken…" : "Index bijwerken"}
            </button>
          </div>
          {index.state?.outcome && <div className="text-neutral-500">{outcomeText(index.state.outcome)}</div>}
          {index.line?.tone === "amber" && <div className="text-amber-600 whitespace-pre-wrap">{index.line.title ?? index.line.text}</div>}
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
          {nas.error && <div className="text-red-600 whitespace-pre-wrap">{nas.error}</div>}
          {mounts && (
            <div className="space-y-0.5">
              <div className="flex items-center gap-2">
                <span className="text-neutral-500">Gevonden op deze Mac</span>
                <span className="flex-1" />
                <button
                  className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 disabled:opacity-40"
                  disabled={connecting || changed || !anyMissing(mounts)}
                  title={changed ? "bewaar eerst de instellingen" : "verbind de gedeelde mappen die nog ontbreken"}
                  onClick={connect}
                >
                  {connecting ? "Verbinden…" : "Verbinden"}
                </button>
                <button
                  className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 disabled:opacity-40"
                  disabled={sync.state?.running || !mounts.library.some((m) => m.chosen)}
                  title="manifesten en previews van de NAS ophalen"
                  onClick={() => sync.start("all", false).catch(() => {})}
                >
                  {sync.state?.running ? "Synchroniseren…" : "Synchroniseren"}
                </button>
              </div>
              {SHARE_LABELS.map(([k, label]) => {
                const where = mountLabel(mounts[k]);
                const attempt = attempts.find((a) => a.share === k);
                const access = check ? accessLabel(check[k]) : null;
                return (
                  <div key={k} className="flex gap-2">
                    <span className="w-24 shrink-0">{label}</span>
                    <span className={where ? "font-mono text-xs leading-5" : "text-neutral-400"}>
                      {where ?? "niet verbonden"}
                      {mounts[k].length > 1 && ` (+${mounts[k].length - 1})`}
                    </span>
                    {access && (
                      <span className={check![k].problem ? "text-amber-600" : "text-neutral-500"}>{access}</span>
                    )}
                    {attempt && attempt.outcome.kind !== "mounted" && (
                      <span className={attempt.outcome.kind === "already_mounted" ? "text-neutral-500" : "text-amber-600"}>
                        {attempt.message}
                      </span>
                    )}
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
