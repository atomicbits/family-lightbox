import { useEffect, useState } from "react";
import { accessLabel, anyMissing, config, mountLabel, nas as nasApi, registrationLabel, type DeviceState, type LocalFolders, type NasLayout, type Settings as S, type SettingsState, type Shares } from "../lib/api";
import { layoutSourceText, locationText, type Nas } from "../lib/nas";
import type { Sync } from "../lib/sync";
import { outcomeText, type Index } from "../lib/indexer";
import { hostProblem, shareProblem } from "../lib/device";
import UpdateLine from "./UpdateLine";
import ImportSettings from "./ImportSettings";
import MoveCheck from "./MoveCheck";
import SourcesTab from "./SourcesTab";
import LightroomSection from "./LightroomSection";
import type { Renamer } from "../lib/renamer";
import type { Update } from "../lib/update";
import { missingText, type Overview, type Previews } from "../lib/previews";
import { LANGUAGE_EVENT, t, type LangSetting } from "../i18n";
import { changedTabs, loadTab, saveTab, tabs, type TabId } from "../lib/settingsTabs";

type Props = { renamer: Renamer; device: DeviceState | null; nas: Nas; sync: Sync; index: Index; update: Update; previews: Previews; onShowMessages: () => void; onClose: () => void };

const SHARE_KEYS: (keyof Shares)[] = ["library", "previews", "lists", "exports"];

/** Settings panel (F021/S01): the topics on tabs, with one Bewaar for all of them. */
export default function Settings({ renamer, device, nas, sync, index, update, previews, onShowMessages, onClose }: Props) {
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
  const [layout, setLayout] = useState<NasLayout | null>(null);
  useEffect(() => { nasApi.layout().then(setLayout).catch(() => {}); }, [nas.check]);
  const { mounts, attempts, check, busy: connecting } = nas;
  const [tab, setTabState] = useState<TabId>(loadTab);
  const setTab = (id: TabId) => { setTabState(id); saveTab(id); };

  const load = () => config.settings().then((s) => { setState(s); setDraft(s.settings); });
  useEffect(() => { load(); nas.refresh(); }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  if (!state || !draft) return null;
  const x = t().settings.panel;

  const problems = [hostProblem(draft.nas_host), draft.home_share ? shareProblem(draft.home_share) : null].filter(
    (p): p is string => p !== null,
  );
  const changed = JSON.stringify(draft) !== JSON.stringify(state.settings);
  const unsaved = changedTabs(draft, state.settings);

  const save = () =>
    config
      .saveSettings(draft)
      .then(() => { setMessage(x.saved); setSaves((n) => n + 1); return load().then(nas.refresh); })
      .catch((e) => setMessage(String(e)));

  const connect = () => nas.connect(true);

  const input = "w-full px-2 py-1 rounded border border-neutral-300 dark:border-neutral-700 bg-transparent";

  return (
    <div className="absolute inset-0 z-30 bg-black/40 flex items-center justify-center p-4" onClick={onClose}>
      <div
        className="w-[48rem] max-w-full h-[40rem] max-h-full rounded-lg bg-white dark:bg-neutral-900 shadow-xl text-sm flex flex-col min-w-0"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="px-5 pt-4 border-b border-neutral-200 dark:border-neutral-800 shrink-0">
          <h2 className="text-base font-semibold">{x.title}</h2>
          <div className="flex gap-1 mt-3 -mb-px overflow-x-auto" role="tablist">
            {tabs().map((b) => (
              <button
                key={b.id}
                role="tab"
                aria-selected={tab === b.id}
                className={`px-3 py-1.5 whitespace-nowrap border-b-2 ${tab === b.id ? "border-neutral-900 dark:border-white font-medium" : "border-transparent text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200"}`}
                onClick={() => setTab(b.id)}
              >
                {b.label}
                {unsaved.has(b.id) && <span className="ml-1 text-amber-600" title={x.notSaved}>●</span>}
              </button>
            ))}
          </div>
        </div>
        <div className="flex-1 min-h-0 overflow-y-auto overflow-x-hidden px-5 py-4 space-y-5 break-words">
        {tab === "general" && (
          <>
            <section className="space-y-1">
              <h3 className="font-medium">{x.thisDevice}</h3>
              {device?.device ? (
                <div>
                  <span className="font-mono">{device.device.name}</span>
                  <span className="text-neutral-500">{x.ofOwner(device.device.owner)}</span>
                </div>
              ) : (
                <div className="text-neutral-500">{x.noName}</div>
              )}
              {device?.error && <div className="text-red-600 whitespace-pre-wrap">{device.error}</div>}
              {check && device?.device && (
                <div className={check.registration.kind.endsWith("registered") ? "text-neutral-500" : "text-amber-600"}>
                  {registrationLabel(check.registration)}
                </div>
              )}
              {device && <div className="text-xs text-neutral-400 font-mono break-all">{device.path}</div>}
            </section>

            {folders && (
              <section className="space-y-1">
                <h3 className="font-medium">{x.localCopies}</h3>
                <div className="text-xs text-neutral-500 font-mono space-y-0.5 break-all">
                  <div>{x.local.library}: {folders.library}</div>
                  <div>{x.local.previews}: {folders.previews}</div>
                  <div>{x.local.lists}: {folders.lists}</div>
                </div>
              </section>
            )}


            <section className="space-y-1">
              <h3 className="font-medium">{t().language.title}</h3>
              <select
                className="px-2 py-1 rounded border border-neutral-300 dark:border-neutral-700 bg-transparent"
                value={state.settings.language ?? ""}
                onChange={(e) => {
                  // saved and used at once, apart from the other settings (F011/S01)
                  const language = e.target.value as LangSetting;
                  config
                    .saveSettings({ ...state.settings, language })
                    .then(() => {
                      setState({ ...state, settings: { ...state.settings, language } });
                      setDraft({ ...draft, language });
                      window.dispatchEvent(new CustomEvent(LANGUAGE_EVENT, { detail: language }));
                    })
                    .catch((err) => setMessage(String(err)));
                }}
              >
                <option value="">{t().language.system}</option>
                <option value="en">English</option>
                <option value="nl">Nederlands</option>
              </select>
              <div className="text-xs text-neutral-500">{t().language.hint}</div>
            </section>

            <LightroomSection />

            <section className="space-y-1">
              <div className="flex items-center gap-2">
                <h3 className="font-medium">{x.about}</h3>
                <span className="flex-1" />
                <button className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700" onClick={onShowMessages}>
                  {x.messages}
                </button>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-neutral-500">FamilyLightbox {update.version ?? ""}</span>
                <span className="flex-1" />
                <button
                  className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 disabled:opacity-40"
                  disabled={update.state.checking || update.state.installing}
                  title={x.checkTitle}
                  onClick={() => update.check(true)}
                >
                  {update.state.checking ? x.checking : x.checkUpdates}
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
          </>
        )}

        {tab === "nas" && (
          <>
            <section className="space-y-2">
              <h3 className="font-medium">{x.nas}</h3>
              {state.error && (
                <div className="text-red-600 whitespace-pre-wrap">
                  {state.error}
                  {"\n"}{x.defaultsInUse}
                </div>
              )}
              <label className="block space-y-1">
                <span>{x.nasName}</span>
                <input className={input} value={draft.nas_host} onChange={(e) => setDraft({ ...draft, nas_host: e.target.value })} />
              </label>
              <label className="block space-y-1">
                <span>{x.homeShare}<span className="font-mono">familylightbox.json</span></span>
                <input className={input} value={draft.home_share} placeholder={draft.shares.lists} onChange={(e) => setDraft({ ...draft, home_share: e.target.value.trim() })} />
              </label>
              <div className="text-xs text-neutral-500">
                {x.layout}: {layoutSourceText(check?.layout_source ?? null)}
                {layout?.name ? ` · ${layout.name}` : ""}
              </div>
              {check?.layout_problem && <div className="text-xs text-amber-600 whitespace-pre-wrap">{check.layout_problem}</div>}
              <MoveCheck />
              {problems.map((p) => <div key={p} className="text-amber-600">{p}</div>)}
              {nas.error && <div className="text-red-600 whitespace-pre-wrap">{nas.error}</div>}
              {mounts && (
                <div className="space-y-0.5">
                  <div className="flex items-center gap-2">
                    <span className="text-neutral-500">{x.foundHere}</span>
                    <span className="flex-1" />
                    <button
                      className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 disabled:opacity-40"
                      disabled={connecting || changed || !anyMissing(mounts)}
                      title={changed ? x.saveFirst : x.connectTitle}
                      onClick={connect}
                    >
                      {connecting ? x.connecting : x.connect}
                    </button>
                    <button
                      className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 disabled:opacity-40"
                      disabled={sync.state?.running || !mounts.library.some((m) => m.chosen)}
                      title={x.syncTitle}
                      onClick={() => sync.start("all", false).catch(() => {})}
                    >
                      {sync.state?.running ? x.syncing : x.sync}
                    </button>
                  </div>
                  {SHARE_KEYS.map((k) => {
                    const label = t().settings.shares[k];
                    const where = mountLabel(mounts[k]);
                    const attempt = attempts.find((a) => a.share === k);
                    const access = check ? accessLabel(check[k]) : null;
                    return (
                      <div key={k} className="flex flex-wrap gap-x-2 min-w-0">
                        <span className="w-24 shrink-0" title={layout ? locationText(layout[k]) : undefined}>{label}</span>
                        {layout && <span className="text-xs text-neutral-500 leading-5 w-32 shrink-0 truncate font-mono">{locationText(layout[k])}</span>}
                        <span className={where ? "font-mono text-xs leading-5 break-all min-w-0" : "text-neutral-400"}>
                          {where ?? x.notConnected}
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
              <div className="text-xs text-neutral-400 font-mono break-all">{state.path}</div>
            </section>
          </>
        )}

        {tab === "import" && (
          <>
            <ImportSettings value={draft.import} onChange={(i) => setDraft({ ...draft, import: i })} version={saves} />
          </>
        )}

        {tab === "sources" && <SourcesTab renamer={renamer} />}
        {tab === "library" && (
          <>
            <section className="space-y-1">
              <div className="flex items-center gap-2">
                <h3 className="font-medium">{x.index}</h3>
                <span className="flex-1" />
                <button
                  className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 disabled:opacity-40"
                  disabled={index.state?.running}
                  title={x.rebuildTitle}
                  onClick={() => index.rebuild({ visible: true, onlyIfStale: false }).catch(() => {})}
                >
                  {index.state?.running ? x.updatingIndex : x.updateIndex}
                </button>
              </div>
              {index.state?.outcome && <div className="text-neutral-500">{outcomeText(index.state.outcome)}</div>}
              {index.line?.tone === "amber" && <div className="text-amber-600 whitespace-pre-wrap">{index.line.title ?? index.line.text}</div>}
            </section>

            <section className="space-y-1">
              <div className="flex items-center gap-2">
                <h3 className="font-medium">{x.previews}</h3>
                <span className="flex-1" />
                <button
                  className="px-2 py-0.5 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700 disabled:opacity-40"
                  disabled={!overview || overview.failed.length === 0}
                  title={x.retryTitle}
                  onClick={() => previews.retry().then(loadOverview).catch((e) => setPreviewsError(String(e)))}
                >
                  {x.retry}
                </button>
              </div>
              {overview && <div className="text-neutral-500">{missingText(overview)}</div>}
              {previews.line && <div className="text-neutral-500">{previews.line.text}</div>}
              {overview && overview.failed.length > 0 && (
                <>
                  <div className="text-amber-600">{x.failed(overview.failed.length)}</div>
                  <ul className="max-h-32 overflow-auto text-xs font-mono text-neutral-500 space-y-0.5">
                    {overview.failed.map((f) => (
                      <li key={f.path} title={f.error} className="break-all">{f.path}: {f.error}</li>
                    ))}
                  </ul>
                </>
              )}
              {previewsError && <div className="text-amber-600">{previewsError}</div>}
            </section>
          </>
        )}
        </div>

        <div className="flex items-center gap-2 px-5 py-3 border-t border-neutral-200 dark:border-neutral-800 shrink-0">
          {message && <span className="text-neutral-500 whitespace-pre-wrap">{message}</span>}
          <span className="flex-1" />
          <button className="px-3 py-1.5 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800" onClick={onClose}>
            {x.close}
          </button>
          <button
            className="px-3 py-1.5 rounded bg-neutral-900 text-white dark:bg-white dark:text-neutral-900 disabled:opacity-40"
            disabled={(!changed && !state.error) || problems.length > 0}
            onClick={save}
          >
            {x.save}
          </button>
        </div>
      </div>
    </div>
  );
}
