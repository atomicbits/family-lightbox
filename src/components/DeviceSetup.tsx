import { useState } from "react";
import { config, type Device } from "../lib/api";
import { deviceNameProblem, ownerProblem, proposeDeviceName } from "../lib/device";

type Props = { onDone: (d: Device) => void };

const KNOWN = ["lynn", "peter"];

/** First start: who is this device? Can't be dismissed; browsing keeps working behind it. */
export default function DeviceSetup({ onDone }: Props) {
  const [choice, setChoice] = useState<string>("lynn");
  const [other, setOther] = useState("");
  const owner = choice === "other" ? other.trim() : choice;
  const [name, setName] = useState(proposeDeviceName("lynn"));
  const [edited, setEdited] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const pickOwner = (c: string, o = other) => {
    setChoice(c);
    const next = c === "other" ? o.trim() : c;
    if (!edited) setName(proposeDeviceName(next));
  };

  const ownerErr = ownerProblem(owner);
  const nameErr = deviceNameProblem(name);

  const save = () => {
    setBusy(true);
    setError(null);
    config
      .createDevice(name, owner)
      .then(onDone)
      .catch((e) => setError(String(e)))
      .finally(() => setBusy(false));
  };

  return (
    <div className="absolute inset-0 z-30 bg-black/40 grid place-items-center">
      <div className="w-[26rem] rounded-lg bg-white dark:bg-neutral-900 shadow-xl p-5 text-sm space-y-4">
        <h2 className="text-base font-semibold">Welkom bij FamilyLightbox</h2>
        <p className="text-neutral-500">
          Van wie is deze laptop? De app onthoudt dat, zodat de lijsten later weten wie wat koos.
        </p>
        <div className="space-y-1">
          {KNOWN.map((k) => (
            <label key={k} className="flex items-center gap-2">
              <input type="radio" checked={choice === k} onChange={() => pickOwner(k)} />
              {k.charAt(0).toUpperCase() + k.slice(1)}
            </label>
          ))}
          <label className="flex items-center gap-2">
            <input type="radio" checked={choice === "other"} onChange={() => pickOwner("other")} />
            iemand anders:
            <input
              className="flex-1 px-2 py-1 rounded border border-neutral-300 dark:border-neutral-700 bg-transparent"
              value={other}
              placeholder="korte naam, bv. karel"
              onFocus={() => pickOwner("other")}
              onChange={(e) => { setOther(e.target.value); pickOwner("other", e.target.value); }}
            />
          </label>
          {owner && ownerErr && <div className="text-amber-600">eigenaar: {ownerErr}</div>}
        </div>
        <label className="block space-y-1">
          <span>Naam van dit toestel</span>
          <input
            className="w-full px-2 py-1 rounded border border-neutral-300 dark:border-neutral-700 bg-transparent font-mono"
            value={name}
            onChange={(e) => { setName(e.target.value); setEdited(true); }}
          />
          {nameErr && <div className="text-amber-600">{nameErr}</div>}
        </label>
        {error && <div className="text-red-600 whitespace-pre-wrap">{error}</div>}
        <div className="flex justify-end">
          <button
            className="px-3 py-1.5 rounded bg-neutral-900 text-white dark:bg-white dark:text-neutral-900 disabled:opacity-40"
            disabled={busy || !!ownerErr || !!nameErr}
            onClick={save}
          >
            Bewaar
          </button>
        </div>
      </div>
    </div>
  );
}
