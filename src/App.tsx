import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api, config, type DeviceState, type Item, type MonthSummary } from "./lib/api";
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
import UpdateLine from "./components/UpdateLine";
import { useUpdate } from "./lib/update";
import ConfirmPreviews from "./components/ConfirmPreviews";
import PreviewsLine from "./components/PreviewsLine";
import ListsPanel from "./components/ListsPanel";
import PhotoMenu from "./components/PhotoMenu";
import { photosText, useLists } from "./lib/lists";
import { addToMonths, markMade, usePreviews, type Done } from "./lib/previews";
import { listen } from "@tauri-apps/api/event";
import { reselect, useIndex, type Anchor } from "./lib/indexer";
import { ARROWS, TILE_MAX, TILE_MIN, clampTile, loadTile, saveTile } from "./lib/grid";
import { moveAcross, parseKey, pickAfterClick, rangeBetween, sameSections, selKey, type Sel, type Section } from "./lib/timeline";

const toSections = (months: MonthSummary[]): Section[] =>
  months.map((m) => ({ month: m.month, count: m.photos + m.videos }));

export default function App() {
  const [months, setMonths] = useState<MonthSummary[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<Sel | null>(null);
  // Several tiles picked with shift or ⌘ (F003/S05), and where a shift range starts.
  const [picked, setPicked] = useState<Set<string>>(new Set());
  const [anchor, setAnchor] = useState<Sel | null>(null);
  // Dragging tiles to a list, and the right-click menu: both act on these tiles.
  const dragKeys = useRef<string[]>([]);
  const [dragging, setDragging] = useState(false);
  const [menu, setMenu] = useState<{ x: number; y: number; keys: string[] } | null>(null);
  const [open, setOpen] = useState(false);
  const [top, setTop] = useState(0);
  const [jump, setJump] = useState<{ section: number; nonce: number } | null>(null);
  const [device, setDevice] = useState<DeviceState | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const nas = useNas();
  const sync = useSync();
  const index = useIndex();
  const update = useUpdate();
  const previews = usePreviews();
  const lists = useLists();
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
      if (m?.library.some((x) => x.chosen)) sync.start("all", false, true).catch(() => {});
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
      // Peter may have imported or pushed with the CLI meanwhile (F008/S02)
      index.rebuild({ visible: false, onlyIfStale: true }).catch(() => {});
    };
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [hasDevice, nas.refresh, index.rebuild]);

  const [waitingForIndex, setWaitingForIndex] = useState(false);
  // Bumped on every reload after a rebuild, so answers to older requests are dropped.
  const generation = useRef(0);

  // At start: show the index there is, and rebuild it in the background (F008/S02). Without an
  // index the grid waits for the first build.
  useEffect(() => {
    config.device().then(setDevice);
    const gen = generation.current;
    api.status().then((s) => {
      if (s.index_exists)
        api.months().then((m) => { if (gen === generation.current) setMonths(m); }).catch((e) => setError(String(e)));
      else setWaitingForIndex(true);
      index.rebuild({ visible: !s.index_exists, onlyIfStale: false }).catch((e) => setError(String(e)));
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // The library as sections, newest month first, with their counts from the index.
  // The same object while the months and their counts stay the same: new previews only change the
  // month list's preview counts, and a new layout would move the grid (F012/S04).
  const stableSections = useRef<Section[]>([]);
  const sections = useMemo(() => {
    const next = toSections(months);
    if (!sameSections(stableSections.current, next)) stableSections.current = next;
    return stableSections.current;
  }, [months]);
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
        const gen = generation.current;
        api
          .items(month)
          .then((items) => {
            if (gen !== generation.current) return;
            cache.current.set(month, items);
            setLoaded((n) => n + 1);
          })
          .catch((e) => setError(String(e)))
          .finally(() => inFlight.current.delete(month));
      }
    },
    [sections],
  );
  const itemsOf = (month: string) => cache.current.get(month);

  useEffect(() => { if (selected) need([selected.section]); }, [selected, need]);

  // A preview made in the background replaces its placeholder right away (F012/S04).
  useEffect(() => {
    const off = listen<Done[]>("previews-made", (e) => {
      const byMonth = new Map<string, Set<string>>();
      for (const d of e.payload) {
        if (!byMonth.has(d.month)) byMonth.set(d.month, new Set());
        byMonth.get(d.month)!.add(d.path);
      }
      let changed = false;
      for (const [month, paths] of byMonth) {
        const items = cache.current.get(month);
        const next = items && markMade(items, paths);
        if (next) { cache.current.set(month, next); changed = true; }
      }
      if (changed) setLoaded((n) => n + 1);
      setMonths((m) => addToMonths(m, e.payload));
    });
    return () => { off.then((f) => f()); };
  }, []);

  const current = selected ? itemsOf(sections[selected.section]?.month)?.[selected.index] : undefined;

  const move = useCallback(
    (m: "left" | "right" | "up" | "down") => setSelected((s) => moveAcross(s, m, cols, counts)),
    [cols, counts],
  );

  const select = useCallback((sel: Sel, mods: { shift: boolean; meta: boolean }) => {
    const r = pickAfterClick(picked, anchor, selected, sel, mods, counts);
    setPicked(r.picked);
    setAnchor(r.anchor);
    setSelected(sel);
  }, [picked, anchor, selected, counts]);

  // Tiles as items; months not loaded yet are fetched first.
  const itemsAt = useCallback(async (keys: Iterable<string>): Promise<Item[]> => {
    const out: Item[] = [];
    for (const k of keys) {
      const { section, index } = parseKey(k);
      const month = sections[section]?.month;
      if (!month) continue;
      let items = cache.current.get(month);
      if (!items) {
        items = await api.items(month);
        cache.current.set(month, items);
      }
      if (items[index]) out.push(items[index]);
    }
    return out;
  }, [sections]);
  const pickedItems = useCallback(() => itemsAt(picked), [itemsAt, picked]);

  // What a drag or the menu acts on: the picked tiles when this one is among them, else only it.
  const actOn = useCallback((sel: Sel): string[] => {
    const k = selKey(sel);
    if (picked.size > 1 && picked.has(k)) return [...picked];
    select(sel, { shift: false, meta: false });
    return [k];
  }, [picked, select]);

  useEffect(() => {
    const end = () => setDragging(false);
    window.addEventListener("dragend", end);
    // the webview's own menu (Open Image in New Window, …) has nothing for this app
    const noMenu = (e: MouseEvent) => {
      if (!(e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement)) e.preventDefault();
    };
    window.addEventListener("contextmenu", noMenu);
    return () => { window.removeEventListener("dragend", end); window.removeEventListener("contextmenu", noMenu); };
  }, []);

  // toetsenbord op het raster: pijltjes (met shift: een reeks), spatie/enter opent, Esc wist de
  // keuze, + zet in de actieve lijst
  const dialog = showSettings || confirming || (device !== null && device.device === null && device.error === null);
  useEffect(() => {
    if (open || dialog) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement) return;
      const m = ARROWS[e.key];
      if (m) {
        e.preventDefault(); // otherwise up/down scroll the grid instead of moving the selection
        if (e.shiftKey && selected) {
          const next = moveAcross(selected, m, cols, counts);
          if (next) {
            const from = anchor ?? selected;
            setAnchor(from);
            setPicked(new Set(rangeBetween(from, next, counts).map(selKey)));
            setSelected(next);
          }
        } else {
          setPicked(new Set());
          setAnchor(null);
          move(m);
        }
      } else if (e.key === "Escape" && picked.size > 0) { setPicked(new Set()); setAnchor(null); }
      else if ((e.key === " " || e.key === "Enter") && selected !== null) { e.preventDefault(); setOpen(true); }
      else if (e.key === "+" || e.key === "=") {
        e.preventDefault();
        if (picked.size > 1) pickedItems().then(lists.toggleMany).catch((err) => setError(String(err)));
        else if (current) lists.toggle(current);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, dialog, selected, anchor, picked, cols, counts, move, current, lists.toggle, lists.toggleMany, pickedItems]);

  // After every rebuild: months again, an empty cache, and the selection and the month at the
  // top kept where they were.
  const builds = index.state?.builds ?? 0;
  useEffect(() => {
    if (builds === 0) return;
    const anchor: Anchor | null =
      selected && sections[selected.section]
        ? { month: sections[selected.section].month, index: selected.index, path: current?.path ?? null }
        : null;
    const topMonth = sections[top]?.month;
    const gen = ++generation.current;
    (async () => {
      try {
        const m = await api.months();
        const items = anchor ? await api.items(anchor.month).catch(() => undefined) : undefined;
        if (gen !== generation.current) return;
        const next = toSections(m);
        cache.current = new Map(anchor && items ? [[anchor.month, items]] : []);
        inFlight.current.clear();
        setError(null);
        setWaitingForIndex(false);
        setMonths(m);
        setSelected(reselect(anchor, next, items));
        setPicked(new Set());
        const t = next.findIndex((x) => x.month === topMonth);
        if (t >= 0 && t !== top) setJump({ section: t, nonce: Date.now() });
      } catch (e) {
        if (gen === generation.current) { setWaitingForIndex(false); setError(String(e)); }
      }
    })();
    // only when a build finished; the rest is read as it is at that moment
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [builds]);
  const topSection = sections[top];
  const lastSection = sections.length - 1;

  // The month on screen gets its previews first (F012/S03).
  const topMonth = topSection?.month ?? null;
  const focusPreviews = previews.focus;
  useEffect(() => { focusPreviews(topMonth); }, [topMonth, focusPreviews]);

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
        {picked.size > 1 && (
          <span className="text-blue-600" title="sleep ze naar een lijst, klik rechts, of + voor de actieve lijst; Esc wist de keuze">
            {picked.size} geselecteerd
          </span>
        )}
        {lists.message && (
          <span className={lists.message.error ? "text-amber-600" : "text-neutral-500"}>{lists.message.text}</span>
        )}
        {index.line && (
          <span className={index.line.tone === "amber" ? "text-amber-600" : "text-neutral-500"} title={index.line.title}>
            {index.line.text}
          </span>
        )}
        {device?.error && (
          <button className="text-red-600" onClick={() => setShowSettings(true)}>
            device.json klopt niet: zie instellingen
          </button>
        )}
        <UpdateLine update={update} syncing={!!sync.state?.running} />
        {hasDevice && <SyncLine sync={sync} onConfirm={() => setConfirming(true)} />}
        <PreviewsLine previews={previews} />
        {hasDevice && <NasIndicator nas={nas} onOpenSettings={() => setShowSettings(true)} />}
        <button
          className="px-2 py-0.5 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-500"
          title="Instellingen"
          onClick={() => setShowSettings(true)}
        >
          ⚙
        </button>
      </header>
      {error || waitingForIndex ? (
        <div className="flex-1 grid place-items-center p-8 text-center whitespace-pre-wrap text-neutral-500">
          {error ?? "index wordt gemaakt…"}
        </div>
      ) : (
        <div className="flex-1 min-h-0 flex relative">
          <div className="w-56 shrink-0 flex flex-col">
            <div className="flex-1 min-h-0">
              <MonthList
                months={months}
                selected={topSection?.month ?? null}
                onSelect={(m) => setJump({ section: sections.findIndex((s) => s.month === m), nonce: Date.now() })}
              />
            </div>
            <div className="max-h-[45%] flex flex-col min-h-0">
              <ListsPanel
                lists={lists}
                month={topSection?.month ?? null}
                dragging={dragging}
                onDrop={(id) => {
                  setDragging(false);
                  itemsAt(dragKeys.current).then((items) => lists.addTo(id, items)).catch((e) => setError(String(e)));
                }}
              />
            </div>
          </div>
          <main className="flex-1 min-w-0">
            <Grid
              sections={sections}
              itemsOf={itemsOf}
              selected={selected}
              onSelect={select}
              picked={picked}
              onDragTile={(sel) => { dragKeys.current = actOn(sel); setDragging(true); }}
              onMenu={(sel, x, y) => setMenu({ x, y, keys: actOn(sel) })}
              onOpen={(s) => { setSelected(s); setOpen(true); }}
              onColumns={setCols}
              onNeed={need}
              onTopSection={setTop}
              jump={jump}
              tile={tile}
              marked={lists.active ? lists.members : null}
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
              list={lists.active ? { name: lists.active.name, member: lists.members.get(current.hash) ?? null, blocked: lists.blocked } : null}
              onToggle={() => lists.toggle(current)}
            />
          ) : (
            <div className="absolute inset-0 z-20 bg-black/95 grid place-items-center text-neutral-400" onClick={() => setOpen(false)}>
              laden…
            </div>
          ))}
        </div>
      )}
      {menu && (
        <PhotoMenu
          x={menu.x}
          y={menu.y}
          what={photosText(menu.keys.length)}
          blocked={lists.blocked}
          lists={(lists.state?.lists ?? []).filter((l) => !l.archived)}
          onAddTo={(id) => itemsAt(menu.keys).then((items) => lists.addTo(id, items)).catch((e) => setError(String(e)))}
          onClose={() => setMenu(null)}
          actions={[
            ...(menu.keys.length === 1 ? [{ label: "Openen", run: () => { setSelected(parseKey(menu.keys[0])); setOpen(true); } }] : []),
            ...(lists.active
              ? [{
                  label: `Uit ${lists.active.name} halen`,
                  disabled: !!lists.blocked,
                  title: lists.blocked ?? undefined,
                  run: () => { const id = lists.active!.id; itemsAt(menu.keys).then((items) => lists.removeFrom(id, items.filter((i) => lists.members.has(i.hash)))); },
                }]
              : []),
          ]}
        />
      )}
      {device && device.device === null && device.error === null && (
        <DeviceSetup onDone={(d) => setDevice({ ...device, device: d })} />
      )}
      {showSettings && (
        <Settings device={device} nas={nas} sync={sync} index={index} update={update} previews={previews} onClose={() => { setShowSettings(false); nas.refresh(); }} />
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
