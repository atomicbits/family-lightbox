import { useEffect, useState } from "react";
import { config, nas as nasApi, type Device, type Shares, type Whoami } from "../lib/api";
import { deviceNameProblem, hostProblem, ownerProblem, people, shareProblem, takenBy } from "../lib/device";
import type { Nas } from "../lib/nas";

type Props = { nas: Nas; onDone: (d: Device) => void };

const SHARES: { key: keyof Shares; label: string }[] = [
  { key: "library", label: "originelen" },
  { key: "previews", label: "previews" },
  { key: "lists", label: "lijsten" },
  { key: "exports", label: "exports" },
];

const field = "px-2 py-1 rounded border border-neutral-300 dark:border-neutral-700 bg-transparent";
const button = "px-3 py-1.5 rounded bg-neutral-900 text-white dark:bg-white dark:text-neutral-900 disabled:opacity-40";

/** First start (F006/S06): connect to your NAS with your own account, then who you are, proposed
 * from that account and the devices the NAS knows. No names are built in. Browsing keeps working
 * behind it; it can't be dismissed, because writing lists needs a device. */
export default function DeviceSetup({ nas, onDone }: Props) {
  const [step, setStep] = useState<"nas" | "who" | null>(null);
  const [host, setHost] = useState("");
  const [shares, setShares] = useState<Shares | null>(null);
  const [otherNames, setOtherNames] = useState(false);
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
      setHost(s.settings.nas_host);
      setShares(s.settings.shares);
      const m = await nasApi.mounts();
      if (s.settings.nas_host && m.lists.some((x) => x.chosen)) await toWho();
      else setStep("nas");
    })().catch((e) => { setError(String(e)); setStep("nas"); });
  }, []);

  const connect = async () => {
    if (!shares) return;
    setBusy(true);
    setError(null);
    try {
      await config.saveSettings({ nas_host: host.trim(), shares });
      await nas.connect(true);
      const m = await nasApi.mounts();
      if (m.lists.some((x) => x.chosen)) await toWho();
      else setError("de gedeelde map voor de lijsten is niet verbonden: kijk de naam van de NAS en je account na, en probeer opnieuw");
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const save = () => {
    setBusy(true);
    setError(null);
    config
      .createDevice(name, owner)
      .then(onDone)
      .catch((e) => setError(String(e)))
      .finally(() => setBusy(false));
  };

  const hostErr = host.trim() ? hostProblem(host.trim()) : "geef de naam of het adres van je NAS";
  const shareErr = shares ? SHARES.map((s) => shareProblem(shares[s.key])).find((x) => x) ?? null : null;
  const known = who?.known ?? [];
  const ownerErr = ownerProblem(owner);
  const taken = takenBy(name, owner, known);
  const nameErr = deviceNameProblem(name) ?? (taken ? `deze naam is op de NAS al van ${taken}` : null);
  const pickOwner = (o: string) => {
    setOwner(o);
    if (!nameEdited && !who?.device) setName(o ? `${o}-mac` : "");
  };
  const failed = nas.attempts.filter((a) => a.outcome.kind !== "mounted" && a.outcome.kind !== "already_mounted");

  return (
    <div className="absolute inset-0 z-30 bg-black/40 grid place-items-center">
      <div className="w-[28rem] rounded-lg bg-white dark:bg-neutral-900 shadow-xl p-5 text-sm space-y-4">
        <h2 className="text-base font-semibold">Welkom bij FamilyLightbox</h2>
        {step === null && <p className="text-neutral-500">even kijken…</p>}

        {step === "nas" && shares && (
          <>
            <p className="text-neutral-500">
              Eerst je NAS, waar de foto's en de lijsten staan. Bij <em>Verbinden</em> vraagt macOS je account op de NAS.
            </p>
            <label className="block space-y-1">
              <span>Naam of adres van de NAS</span>
              <input className={`w-full ${field} font-mono`} value={host} placeholder="bv. mijnnas of 192.168.1.20" autoFocus onChange={(e) => setHost(e.target.value)} />
              {host.trim() && hostErr && <div className="text-amber-600">{hostErr}</div>}
            </label>
            <div>
              <button className="text-xs text-neutral-500 hover:underline" onClick={() => setOtherNames((o) => !o)}>
                {otherNames ? "▾" : "▸"} andere namen voor de gedeelde mappen
              </button>
              {otherNames && (
                <div className="mt-2 grid grid-cols-2 gap-2">
                  {SHARES.map((s) => (
                    <label key={s.key} className="space-y-0.5">
                      <span className="text-xs text-neutral-500">{s.label}</span>
                      <input className={`w-full ${field} font-mono`} value={shares[s.key]} onChange={(e) => setShares({ ...shares, [s.key]: e.target.value })} />
                    </label>
                  ))}
                </div>
              )}
              {shareErr && <div className="text-amber-600">{shareErr}</div>}
            </div>
            {error && <div className="text-red-600 whitespace-pre-wrap">{error}</div>}
            {failed.map((a) => (
              <div key={a.share} className="text-xs text-amber-600">{a.name}: {a.message}</div>
            ))}
            <div className="flex justify-between items-center">
              <button className="text-xs text-neutral-500 hover:underline" disabled={busy} onClick={() => toWho().catch((e) => setError(String(e)))}>
                zonder NAS verder
              </button>
              <button className={button} disabled={busy || !!hostErr || !!shareErr} onClick={connect}>
                {busy ? "verbinden…" : "Verbinden"}
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
            {error && <div className="text-red-600 whitespace-pre-wrap">{error}</div>}
            <div className="flex justify-between items-center">
              <button className="text-xs text-neutral-500 hover:underline" disabled={busy} onClick={() => { setError(null); setStep("nas"); }}>
                ‹ NAS
              </button>
              <button className={button} disabled={busy || !!ownerErr || !!nameErr} onClick={save}>
                Bewaar
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
