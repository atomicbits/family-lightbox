import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api, config, type DeviceState, type Item, type MonthSummary, type Status } from "./lib/api";
import { monthLabel, yearOf } from "./lib/months";
import MonthList from "./components/MonthList";
import Grid from "./components/Grid";
import Lightbox from "./components/Lightbox";
import DeviceSetup from "./components/DeviceSetup";
import Settings from "./components/Settings";
import NasIndicator from "./components/NasIndicator";
import { useNas } from "./lib/nas";
import { useSync } from "./lib/sync";
import SyncLine from "./components/SyncLine";
import ConfirmPreviews from "./components/ConfirmPreviews";
import { ARROWS, TILE_MAX, TILE_MIN, clampTile, loadTile, saveTile } from "./lib/grid";
import { moveAcross, type Sel } from "./lib/timeline";

export default function App() {
  const [status, setStatus] = useState<Status | null>(null);
  const [months, setMonths] = useState<MonthSummary[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<Sel | null>(null);
  const [open, setOpen] = useState(false);
  const [top, setTop] = useState(0);
  const [jump, setJump] = useState<{ section: number; nonce: number } | null>(null);
  const [device, setDevice] = useState<DeviceState | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const nas = useNas();
  const sync = useSync();
  const [confirming, setConfirming] = useState(false);
  const [cols, setCols] = useState(1);
  const [tile, setTile] = useState(loadTile);
  const changeTile = (n: number) => { const t = clampTile(n); setTile(t); saveTile(t); };
  const lastRefresh = useRef(0);

  // NAS at start, once the device is known: quiet connect without a dialog, then the check.
  const hasDevice = device?.device != null;
  useEffect(() => {
    if (!hasDevice) return;
    lastRefresh.current = Date.now();
    // Then sync from the NAS in the background, once Library is connected (F007/S05).
    nas.start().then((m) => {
      if (m?.library.some((x) => x.chosen)) sync.start("all", false).catch(() => {});
    });
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
    api.months().then(setMonths).catch((e) => setError(String(e)));
  }, []);

  // The library as sections, newest month first, with their counts from the index.
  const sections = useMemo(() => months.map((m) => ({ month: m.month, count: m.photos + m.videos })), [months]);
  const counts = useMemo(() => sections.map((s) => s.count), [sections]);

  // Items per month, fetched once when a month comes (nearly) into view.
  const cache = useRef(new Map<string, Item[]>());
  const inFlight = useRef(new Set<string>());
  const [, setLoaded] = useState(0);
  const need = useCallback(
    (wanted: number[]) => {
      for (const s of wanted) {
        const month = sections[s]?.month;
        if (!month || cache.current.has(month) || inFlight.current.has(month)) continue;
        inFlight.current.add(month);
        api
          .items(month)
          .then((items) => { cache.current.set(month, items); setLoaded((n) => n + 1); })
          .catch((e) => setError(String(e)))
          .finally(() => inFlight.current.delete(month));
      }
    },
    [sections],
  );
  const itemsOf = (month: string) => cache.current.get(month);

  useEffect(() => { if (selected) need([selected.section]); }, [selected, need]);

  const move = useCallback(
    (m: "left" | "right" | "up" | "down") => setSelected((s) => moveAcross(s, m, cols, counts)),
    [cols, counts],
  );

  // toetsenbord op het raster: pijltjes, spatie/enter opent, Esc sluit
  const dialog = showSettings || confirming || (device !== null && device.device === null && device.error === null);
  useEffect(() => {
    if (open || dialog) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement) return;
      const m = ARROWS[e.key];
      if (m) {
        e.preventDefault(); // otherwise up/down scroll the grid instead of moving the selection
        move(m);
      } else if ((e.key === " " || e.key === "Enter") && selected !== null) { e.preventDefault(); setOpen(true); }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, dialog, selected, move]);

  const current = selected ? itemsOf(sections[selected.section].month)?.[selected.index] : undefined;
  const topSection = sections[top];
  const lastSection = sections.length - 1;

  return (
    <div className="relative h-full flex flex-col bg-white dark:bg-neutral-950 text-neutral-900 dark:text-neutral-100">
      <header className="flex items-center gap-3 px-4 h-10 border-b border-neutral-200 dark:border-neutral-800 text-sm">
        <span className="font-semibold">
          {topSection ? `${monthLabel(topSection.month)} ${yearOf(topSection.month)}` : "FamilyLightbox"}
        </span>
        {topSection && (
          <span className="text-neutral-400">
            {topSection.count} {topSection.count === 1 ? "item" : "items"}
            {selected?.section === top && ` · ${selected.index + 1} / ${topSection.count}`}
          </span>
        )}
        <span className="flex-1" />
        <label className="flex items-center gap-1.5 text-neutral-400" title="grootte van de miniaturen">
          <span className="text-xs">▫</span>
          <input
            type="range"
            min={TILE_MIN}
            max={TILE_MAX}
            step={8}
            value={tile}
            onChange={(e) => changeTile(Number(e.target.value))}
            // let go of focus after dragging, so the arrow keys move through the photos again
            onPointerUp={(e) => e.currentTarget.blur()}
            className="w-28 accent-neutral-500"
          />
          <span className="text-base leading-none">□</span>
        </label>
        {status && !status.index_exists && (
          <span className="text-amber-600">geen index: draai ./media index</span>
        )}
        {device?.error && (
          <button className="text-red-600" onClick={() => setShowSettings(true)}>
            device.json klopt niet: zie instellingen
          </button>
        )}
        {hasDevice && <SyncLine sync={sync} onConfirm={() => setConfirming(true)} />}
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
            <MonthList
              months={months}
              selected={topSection?.month ?? null}
              onSelect={(m) => setJump({ section: sections.findIndex((s) => s.month === m), nonce: Date.now() })}
            />
          </div>
          <main className="flex-1 min-w-0">
            <Grid
              sections={sections}
              itemsOf={itemsOf}
              selected={selected}
              onSelect={setSelected}
              onOpen={(s) => { setSelected(s); setOpen(true); }}
              onColumns={setCols}
              onNeed={need}
              onTopSection={setTop}
              jump={jump}
              tile={tile}
            />
          </main>
          {open && selected && (current ? (
            <Lightbox
              item={current}
              index={selected.index}
              count={sections[selected.section].count}
              canPrev={selected.section > 0 || selected.index > 0}
              canNext={selected.section < lastSection || selected.index < sections[selected.section].count - 1}
              onClose={() => setOpen(false)}
              onStep={(d) => move(d > 0 ? "right" : "left")}
            />
          ) : (
            <div className="absolute inset-0 z-20 bg-black/95 grid place-items-center text-neutral-400" onClick={() => setOpen(false)}>
              laden…
            </div>
          ))}
        </div>
      )}
      {device && device.device === null && device.error === null && (
        <DeviceSetup onDone={(d) => setDevice({ ...device, device: d })} />
      )}
      {showSettings && (
        <Settings device={device} nas={nas} sync={sync} onClose={() => { setShowSettings(false); nas.refresh(); }} />
      )}
      {confirming && sync.state?.summary && (
        <ConfirmPreviews
          summary={sync.state.summary}
          onYes={() => { setConfirming(false); sync.start("previews", true).catch(() => {}); }}
          onLater={() => setConfirming(false)}
        />
      )}
    </div>
  );
}
