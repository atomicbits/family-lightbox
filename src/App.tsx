import { useCallback, useEffect, useRef, useState } from "react";
import { api, config, type DeviceState, type Item, type MonthSummary, type Status } from "./lib/api";
import { monthLabel, yearOf } from "./lib/months";
import MonthList from "./components/MonthList";
import Grid from "./components/Grid";
import Lightbox from "./components/Lightbox";
import DeviceSetup from "./components/DeviceSetup";
import Settings from "./components/Settings";
import NasIndicator from "./components/NasIndicator";
import { useNas } from "./lib/nas";
import { ARROWS, moveInGrid } from "./lib/grid";

export default function App() {
  const [status, setStatus] = useState<Status | null>(null);
  const [months, setMonths] = useState<MonthSummary[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [month, setMonth] = useState<string | null>(null);
  const [items, setItems] = useState<Item[]>([]);
  const [selected, setSelected] = useState<number | null>(null);
  const [open, setOpen] = useState(false);
  const [device, setDevice] = useState<DeviceState | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const nas = useNas();
  const [cols, setCols] = useState(1);
  const lastRefresh = useRef(0);

  // NAS at start, once the device is known: quiet connect without a dialog, then the check.
  const hasDevice = device?.device != null;
  useEffect(() => {
    if (!hasDevice) return;
    lastRefresh.current = Date.now();
    nas.start();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hasDevice]);

  // Back in the app: look again (at most every 30 s), so a share ejected in the Finder shows up.
  useEffect(() => {
    if (!hasDevice) return;
    const onFocus = () => {
      if (Date.now() - lastRefresh.current < 30_000) return;
      lastRefresh.current = Date.now();
      nas.refresh();
    };
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [hasDevice, nas.refresh]);

  useEffect(() => {
    config.device().then(setDevice);
    api.status().then(setStatus);
    api
      .months()
      .then((m) => {
        setMonths(m);
        if (m.length && !month) setMonth(m[0].month);
      })
      .catch((e) => setError(String(e)));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!month) return;
    setSelected(null);
    setOpen(false);
    api.items(month).then(setItems).catch((e) => setError(String(e)));
  }, [month]);

  const step = useCallback(
    (delta: number) =>
      setSelected((s) => {
        if (items.length === 0) return null;
        const n = (s ?? -1) + delta;
        return Math.min(items.length - 1, Math.max(0, n));
      }),
    [items.length],
  );

  // toetsenbord op het raster: pijltjes, spatie/enter opent, Esc sluit
  const dialog = showSettings || (device !== null && device.device === null && device.error === null);
  useEffect(() => {
    if (open || dialog) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement) return;
      const move = ARROWS[e.key];
      if (move) {
        e.preventDefault(); // otherwise up/down scroll the grid instead of moving the selection
        setSelected((s) => moveInGrid(s, move, cols, items.length));
      } else if ((e.key === " " || e.key === "Enter") && selected !== null) { e.preventDefault(); setOpen(true); }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, dialog, selected, cols, items.length]);

  const current = selected !== null ? items[selected] : undefined;

  return (
    <div className="relative h-full flex flex-col bg-white dark:bg-neutral-950 text-neutral-900 dark:text-neutral-100">
      <header className="flex items-center gap-3 px-4 h-10 border-b border-neutral-200 dark:border-neutral-800 text-sm">
        <span className="font-semibold">
          {month ? `${monthLabel(month)} ${yearOf(month)}` : "FamilyLightbox"}
        </span>
        {month && (
          <span className="text-neutral-400">
            {items.length} {items.length === 1 ? "item" : "items"}
            {current && ` · ${selected! + 1} gekozen`}
          </span>
        )}
        <span className="flex-1" />
        {status && !status.index_exists && (
          <span className="text-amber-600">geen index: draai ./media index</span>
        )}
        {device?.error && (
          <button className="text-red-600" onClick={() => setShowSettings(true)}>
            device.json klopt niet: zie instellingen
          </button>
        )}
        {hasDevice && <NasIndicator nas={nas} onOpenSettings={() => setShowSettings(true)} />}
        <button
          className="px-2 py-0.5 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-500"
          title="Instellingen"
          onClick={() => setShowSettings(true)}
        >
          ⚙
        </button>
      </header>
      {error ? (
        <div className="flex-1 grid place-items-center p-8 text-center whitespace-pre-wrap text-neutral-500">
          {error}
        </div>
      ) : (
        <div className="flex-1 min-h-0 flex relative">
          <div className="w-56 shrink-0">
            <MonthList months={months} selected={month} onSelect={setMonth} />
          </div>
          <main className="flex-1 min-w-0">
            <Grid
              items={items}
              selected={selected}
              onSelect={setSelected}
              onOpen={(i) => { setSelected(i); setOpen(true); }}
              onColumns={setCols}
            />
          </main>
          {open && current && (
            <Lightbox
              item={current}
              index={selected!}
              count={items.length}
              onClose={() => setOpen(false)}
              onStep={step}
            />
          )}
        </div>
      )}
      {device && device.device === null && device.error === null && (
        <DeviceSetup onDone={(d) => setDevice({ ...device, device: d })} />
      )}
      {showSettings && (
        <Settings device={device} nas={nas} onClose={() => { setShowSettings(false); nas.refresh(); }} />
      )}
    </div>
  );
}
