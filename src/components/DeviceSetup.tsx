import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { config, nas as nasApi, type Device, type Home, type LocalFolders, type Location, type NasLayout, type Settings, type Shares, type Whoami } from "../lib/api";
import { deviceNameProblem, freeText, hostProblem, ownerProblem, parentOf, pathProblem, people, setUpText, shareProblem, takenBy } from "../lib/device";
import type { Nas } from "../lib/nas";
import { NO_IMPORT } from "../lib/sources";

/** `device`: this Mac has one already, and only the local folder is still to choose (F006/S07). */
type Props = { nas: Nas; device: Device | null; onDone: (d: Device) => void };

const LOCATIONS: { key: keyof Shares; label: string }[] = [
  { key: "library", label: "originelen" },
  { key: "previews", label: "previews" },
  { key: "lists", label: "lijsten" },
  { key: "exports", label: "exports" },
];

const field = "px-2 py-1 rounded border border-neutral-300 dark:border-neutral-700 bg-transparent";
const button = "px-3 py-1.5 rounded bg-neutral-900 text-white dark:bg-white dark:text-neutral-900 disabled:opacity-40";

/** First start (F006/S06): connect to your NAS and its home share with your own account; the
 * library described there is used, or a new one set up in it (F019/S02); then who you are,
 * proposed from that account and the devices the NAS knows. No names are built in. Browsing keeps working
 * behind it; it can't be dismissed, because writing lists needs a device. */
export default function DeviceSetup({ nas, device, onDone }: Props) {
  const [step, setStep] = useState<"nas" | "library" | "who" | "folder" | null>(null);
  const [host, setHost] = useState("");
  const [homeShare, setHomeShare] = useState("");
  const [home, setHome] = useState<Home | null>(null);
  const [libName, setLibName] = useState("");
  const [pointed, setPointed] = useState<Record<keyof Shares, Location> | null>(null);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [local, setLocal] = useState<LocalFolders | null>(null);
  const [folder, setFolder] = useState("");
  const [free, setFree] = useState<number | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [who, setWho] = useState<Whoami | null>(null);
  const [owner, setOwner] = useState("");
  const [name, setName] = useState("");
  const [nameEdited, setNameEdited] = useState(false);

  const toWho = async () => {
    const w = await nasApi.whoami();
    setWho(w);
    setOwner(w.owner);
    setName(w.device || (w.owner ? `${w.owner}-mac` : ""));
    setStep("who");
  };

  // a NAS that is set up and connected already (shares mounted in the Finder) skips step 1
  useEffect(() => {
    (async () => {
      const s = await config.settings();
      setSettings(s.settings);
      setHost(s.settings.nas_host);
      setHomeShare(s.settings.home_share);
      const l = await config.localFolders();
      setLocal(l);
      setFolder(l.chosen ? parentOf(l) ?? "" : l.proposal);
      if (device) return setStep("folder");
      const m = await nasApi.mounts();
      if (s.settings.nas_host && m.lists.some((x) => x.chosen)) await toWho();
      else setStep("nas");
    })().catch((e) => { setError(String(e)); setStep("nas"); });
  }, []);

  /** The home share connected: its library used, or the choice to set one up (F019/S02). */
  const afterConnect = async () => {
    const h = await nasApi.home();
    setHome(h);
    if (!h.connected) return setError(`de gedeelde map '${h.share}' is niet verbonden: kijk de naam van de NAS, de map en je account na, en probeer opnieuw`);
    if (h.found) return toWho();
    if (h.problem) return setError(h.problem);
    setStep("library");
  };

  const connect = async () => {
    setBusy(true);
    setError(null);
    try {
      // the whole settings, so a chosen local folder stays
      const base = settings ?? { home_share: "", shares: { library: "Library", previews: "Previews", lists: "Lists", exports: "Exports" }, local: { library: "", previews: "", lists: "" }, import: NO_IMPORT };
      const next = { ...base, nas_host: host.trim(), home_share: homeShare.trim() };
      await config.saveSettings(next);
      setSettings(next);
      await nas.connect(true);
      await afterConnect();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  // the free space where the local copies would go
  useEffect(() => {
    if (!folder) return setFree(null);
    config.freeSpace(folder).then(setFree).catch(() => setFree(null));
  }, [folder]);

  const pickFolder = async () => {
    const f = await open({ directory: true, defaultPath: folder || undefined, title: "Map voor de lokale kopieën" });
    if (typeof f === "string") setFolder(f);
  };

  /** A new library in the home share, or the file pointing at folders that exist; then the shares
   * it names are connected. */
  const setUp = async (layout: NasLayout | null) => {
    setBusy(true);
    setError(null);
    try {
      await nasApi.setUp(libName, layout);
      await nas.connect(true);
      await afterConnect();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  const pointedLayout = (): NasLayout | null =>
    pointed && { contract_version: "1.4", name: libName, library: pointed.library, previews: pointed.previews, lists: pointed.lists, exports: pointed.exports };

  /** A new folder is kept and the app starts again so everything uses it (F006/S07). */
  const save = async () => {
    setBusy(true);
    setError(null);
    try {
      const choose = local !== null && !local.chosen;
      if (choose) await config.chooseLocal(folder);
      const d = device ?? (await config.createDevice(name, owner));
      if (choose) await config.relaunch();
      else onDone(d);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const hostErr = host.trim() ? hostProblem(host.trim()) : "geef de naam of het adres van je NAS";
  const homeErr = homeShare.trim() ? shareProblem(homeShare.trim()) : "geef de gedeelde map van de bibliotheek";
  const pointedErr = pointed
    ? LOCATIONS.map((l) => shareProblem(pointed[l.key].share.trim()) ?? pathProblem(pointed[l.key].path.trim())).find((x) => x) ?? null
    : null;
  const known = who?.known ?? [];
  const ownerErr = ownerProblem(owner);
  const taken = takenBy(name, owner, known);
  const nameErr = deviceNameProblem(name) ?? (taken ? `deze naam is op de NAS al van ${taken}` : null);
  const pickOwner = (o: string) => {
    setOwner(o);
    if (!nameEdited && !who?.device) setName(o ? `${o}-mac` : "");
  };
  const folderPicker = (
    <div className="space-y-1">
      <span>Map voor de lokale kopieën</span>
      <div className="flex gap-2 items-center">
        <input className={`flex-1 ${field} font-mono text-xs`} value={folder} onChange={(e) => setFolder(e.target.value)} />
        <button className="px-2 py-1 rounded bg-neutral-100 hover:bg-neutral-200 dark:bg-neutral-800 dark:hover:bg-neutral-700" onClick={pickFolder}>Kies…</button>
      </div>
      <div className="text-xs text-neutral-500">
        previews, lijsten en de foto's die je lokaal houdt{free !== null ? ` · ${freeText(free)}` : ""}
      </div>
    </div>
  );
  const failed = nas.attempts.filter((a) => a.outcome.kind !== "mounted" && a.outcome.kind !== "already_mounted");

  return (
    <div className="absolute inset-0 z-30 bg-black/40 flex items-center justify-center p-4">
      <div className="w-[28rem] rounded-lg max-h-full overflow-y-auto bg-white dark:bg-neutral-900 shadow-xl p-5 text-sm space-y-4">
        <h2 className="text-base font-semibold">Welkom bij FamilyLightbox</h2>
        {step === null && <p className="text-neutral-500">even kijken…</p>}

        {step === "nas" && (
          <>
            <p className="text-neutral-500">
              Eerst je NAS en de gedeelde map waar de foto's en de lijsten staan. Bij <em>Verbinden</em> vraagt macOS je account op de NAS.
            </p>
            <label className="block space-y-1">
              <span>Naam of adres van de NAS</span>
              <input className={`w-full ${field} font-mono`} value={host} placeholder="bv. mijnnas of 192.168.1.20" autoFocus onChange={(e) => setHost(e.target.value)} />
              {host.trim() && hostErr && <div className="text-amber-600">{hostErr}</div>}
            </label>
            <label className="block space-y-1">
              <span>Gedeelde map van de bibliotheek</span>
              <input className={`w-full ${field} font-mono`} value={homeShare} placeholder="bv. FamilyLightbox" onChange={(e) => setHomeShare(e.target.value)} />
              <div className="text-xs text-neutral-500">daar staat de bibliotheek, of daar wordt een nieuwe ingericht</div>
              {homeShare.trim() && homeErr && <div className="text-amber-600">{homeErr}</div>}
            </label>
            {error && <div className="text-red-600 whitespace-pre-wrap">{error}</div>}
            {failed.map((a) => (
              <div key={a.share} className="text-xs text-amber-600">{a.name}: {a.message}</div>
            ))}
            <div className="flex justify-between items-center">
              <button className="text-xs text-neutral-500 hover:underline" disabled={busy} onClick={() => toWho().catch((e) => setError(String(e)))}>
                zonder NAS verder
              </button>
              <button className={button} disabled={busy || !!hostErr || !!homeErr} onClick={connect}>
                {busy ? "verbinden…" : "Verbinden"}
              </button>
            </div>
          </>
        )}

        {step === "library" && home && (
          <>
            <p className="text-neutral-500">
              In <span className="font-mono">{home.share}</span> staat nog geen bibliotheek.
            </p>
            <label className="block space-y-1">
              <span>Naam van de bibliotheek</span>
              <input className={`w-full ${field}`} value={libName} placeholder="bv. Onze foto's" onChange={(e) => setLibName(e.target.value)} />
            </label>
            {!pointed ? (
              <>
                <p className="text-xs text-neutral-500">{setUpText(home.share, home.existing)}</p>
                <button
                  className="text-xs text-neutral-500 hover:underline"
                  onClick={() => {
                    const s = settings?.shares ?? { library: "Library", previews: "Previews", lists: "Lists", exports: "Exports" };
                    setPointed({ library: { share: s.library, path: "" }, previews: { share: s.previews, path: "" }, lists: { share: s.lists, path: "" }, exports: { share: s.exports, path: "" } });
                  }}
                >
                  ▸ de bibliotheek staat elders: bestaande mappen aanwijzen
                </button>
              </>
            ) : (
              <div className="space-y-2">
                <div className="text-xs text-neutral-500">
                  Per onderdeel een gedeelde map en een map daarin (leeg: de gedeelde map zelf). Dit komt in familylightbox.json in{" "}
                  <span className="font-mono">{home.share}</span>.
                </div>
                {LOCATIONS.map((l) => (
                  <div key={l.key} className="grid grid-cols-[5rem_1fr_1fr] gap-2 items-center">
                    <span className="text-xs text-neutral-500">{l.label}</span>
                    <input className={`${field} font-mono`} value={pointed[l.key].share} onChange={(e) => setPointed({ ...pointed, [l.key]: { ...pointed[l.key], share: e.target.value } })} />
                    <input className={`${field} font-mono`} value={pointed[l.key].path} placeholder="map" onChange={(e) => setPointed({ ...pointed, [l.key]: { ...pointed[l.key], path: e.target.value } })} />
                  </div>
                ))}
                {pointedErr && <div className="text-amber-600">{pointedErr}</div>}
                <button className="text-xs text-neutral-500 hover:underline" onClick={() => setPointed(null)}>
                  ▾ toch een nieuwe bibliotheek
                </button>
              </div>
            )}
            {error && <div className="text-red-600 whitespace-pre-wrap">{error}</div>}
            {failed.map((a) => (
              <div key={a.share} className="text-xs text-amber-600">{a.name}: {a.message}</div>
            ))}
            <div className="flex justify-between items-center">
              <button className="text-xs text-neutral-500 hover:underline" disabled={busy} onClick={() => { setError(null); setStep("nas"); }}>
                ‹ NAS
              </button>
              {pointed ? (
                <button className={button} disabled={busy || !!pointedErr} onClick={() => setUp(pointedLayout())}>
                  {busy ? "bewaren…" : "Mappen aanwijzen"}
                </button>
              ) : (
                <button className={button} disabled={busy} onClick={() => setUp(null)}>
                  {busy ? "inrichten…" : "Hier een nieuwe bibliotheek inrichten"}
                </button>
              )}
            </div>
          </>
        )}

        {step === "folder" && (
          <>
            <p className="text-neutral-500">
              Kies waar FamilyLightbox op deze laptop zijn kopieën bewaart. Daarna start de app opnieuw.
            </p>
            {folderPicker}
            {error && <div className="text-red-600 whitespace-pre-wrap">{error}</div>}
            <div className="flex justify-end">
              <button className={button} disabled={busy || !folder.trim()} onClick={save}>
                Bewaar
              </button>
            </div>
          </>
        )}

        {step === "who" && (
          <>
            <p className="text-neutral-500">
              {who?.user ? <>Verbonden als <span className="font-mono">{who.user}</span>. </> : null}
              Wie gebruikt deze laptop? De lijsten, sterren en hartjes onthouden wie wat koos.
            </p>
            {known.length > 0 && (
              <div className="space-y-1">
                <div className="text-xs text-neutral-500">al bekend op de NAS</div>
                <div className="flex flex-wrap gap-1.5">
                  {people(known).map((p) => (
                    <button
                      key={p.owner}
                      className={"px-2 py-0.5 rounded border text-xs " + (owner === p.owner ? "border-blue-500 bg-blue-50 dark:bg-blue-950" : "border-neutral-300 dark:border-neutral-700")}
                      title={`toestellen: ${p.devices.join(", ")}`}
                      onClick={() => pickOwner(p.owner)}
                    >
                      {p.owner}
                    </button>
                  ))}
                </div>
              </div>
            )}
            <label className="block space-y-1">
              <span>Jouw naam</span>
              <input className={`w-full ${field} font-mono`} value={owner} placeholder="korte naam, kleine letters" onChange={(e) => pickOwner(e.target.value)} />
              {owner && ownerErr && <div className="text-amber-600">{ownerErr}</div>}
            </label>
            <label className="block space-y-1">
              <span>Naam van deze laptop{who?.computer ? <span className="text-neutral-400"> ({who.computer})</span> : null}</span>
              <input className={`w-full ${field} font-mono`} value={name} onChange={(e) => { setName(e.target.value); setNameEdited(true); }} />
              {name && nameErr && <div className="text-amber-600">{nameErr}</div>}
            </label>
            {local && !local.chosen && step === "who" && folderPicker}
            {error && <div className="text-red-600 whitespace-pre-wrap">{error}</div>}
            <div className="flex justify-between items-center">
              <button className="text-xs text-neutral-500 hover:underline" disabled={busy} onClick={() => { setError(null); setStep("nas"); }}>
                ‹ NAS
              </button>
              <button className={button} disabled={busy || !!ownerErr || !!nameErr || (local !== null && !local.chosen && !folder.trim())} onClick={save}>
                Bewaar
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
