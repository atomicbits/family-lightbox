import { useEffect, useState } from "react";
import { accessLabel, anyMissing, config, mountLabel, registrationLabel, type DeviceState, type LocalFolders, type Settings as S, type SettingsState, type Shares } from "../lib/api";
import type { Nas } from "../lib/nas";
import type { Sync } from "../lib/sync";
import { outcomeText, type Index } from "../lib/indexer";
import { hostProblem, shareProblem } from "../lib/device";
import UpdateLine from "./UpdateLine";
import ImportSettings from "./ImportSettings";
import type { Update } from "../lib/update";
import { missingText, type Overview, type Previews } from "../lib/previews";

type Props = { device: DeviceState | null; nas: Nas; sync: Sync; index: Index; update: Update; previews: Previews; onShowMessages: () => void; onClose: () => void };

const SHARE_LABELS: [keyof Shares, string][] = [
  ["library", "Originelen"],
  ["previews", "Previews"],
  ["lists", "Lijsten"],
  ["exports", "Exports"],
];

/** Settings panel: the device (read-only) and the NAS host and share names (editable). */
export default function Settings({ device, nas, sync, index, update, previews, onShowMessages, onClose }: Props) {
  const [overview, setOverview] = useState<Overview | null>(null);
  const [previewsError, setPreviewsError] = useState<string | null>(null);
  const making = previews.state?.running ?? false;
  const loadOverview = () => previews.overview().then(setOverview).catch((e) => setPreviewsError(String(e)));
  // again when a run ends, so the counts and failures are current
  useEffect(() => { if (!making) loadOverview(); }, [making]);
  const [state, setState] = useState<SettingsState | null>(null);
  const [folders, setFolders] = useState<LocalFolders | null>(null);
  useEffect(() => { config.localFolders().then(setFolders).catch(() => {}); }, []);
  const [draft, setDraft] = useState<S | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [saves, setSaves] = useState(0);
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
      .then(() => { setMessage("bewaard"); setSaves((n) => n + 1); return load().then(nas.refresh); })
      .catch((e) => setMessage(String(e)));

  const connect = () => nas.connect(true);

  const input = "w-full px-2 py-1 rounded border border-neutral-300 dark:border-neutral-700 bg-transparent";

  return (
    <div className="absolute inset-0 z-30 bg-black/40 flex items-center justify-center p-4" onClick={onClose}>
      <div
        className="w-[30rem] rounded-lg max-h-full overflow-y-auto bg-white dark:bg-neutral-900 shadow-xl p-5 text-sm space-y-4"
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

        {folders && (
          <section className="space-y-1">
            <h3 className="font-medium">Lokale kopieën</h3>
            <div className="text-xs text-neutral-500 font-mono space-y-0.5">
              <div>originelen: {folders.library}</div>
              <div>previews: {folders.previews}</div>
              <div>lijsten: {folders.lists}</div>
            </div>
          </section>
        )}

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

        <section className="space-y-1">
          <div className="flex items-center gap-2">
            <h3 className="font-medium">Previews</h3>
            <span className="flex-1" />
            <button
              className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 disabled:opacity-40"
              disabled={!overview || overview.failed.length === 0}
              title="de mislukte previews opnieuw proberen te maken"
              onClick={() => previews.retry().then(loadOverview).catch((e) => setPreviewsError(String(e)))}
            >
              Opnieuw proberen
            </button>
          </div>
          {overview && <div className="text-neutral-500">{missingText(overview)}</div>}
          {previews.line && <div className="text-neutral-500">{previews.line.text}</div>}
          {overview && overview.failed.length > 0 && (
            <>
              <div className="text-amber-600">{overview.failed.length} mislukt:</div>
              <ul className="max-h-32 overflow-auto text-xs font-mono text-neutral-500 space-y-0.5">
                {overview.failed.map((f) => (
                  <li key={f.path} title={f.error}>{f.path}: {f.error}</li>
                ))}
              </ul>
            </>
          )}
          {previewsError && <div className="text-amber-600">{previewsError}</div>}
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

        <ImportSettings value={draft.import} onChange={(i) => setDraft({ ...draft, import: i })} version={saves} />

        <section className="space-y-1">
          <div className="flex items-center gap-2">
            <h3 className="font-medium">Over</h3>
            <span className="flex-1" />
            <button className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700" onClick={onShowMessages}>
              Meldingen
            </button>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-neutral-500">FamilyLightbox {update.version ?? ""}</span>
            <span className="flex-1" />
            <button
              className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 disabled:opacity-40"
              disabled={update.state.checking || update.state.installing}
              title="kijken of er een nieuwere versie is"
              onClick={() => update.check(true)}
            >
              {update.state.checking ? "Controleren…" : "Controleren op updates"}
            </button>
          </div>
          {/* a newer version is installed from here too, not only from the bar behind this panel */}
          {update.state.offer || update.state.installing ? (
            <UpdateLine update={update} syncing={!!sync.state?.running} />
          ) : (
            update.state.checked && <div className="text-neutral-500">{update.state.checked}</div>
          )}
          {update.state.checkError && <div className="text-red-600 whitespace-pre-wrap">{update.state.checkError}</div>}
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
