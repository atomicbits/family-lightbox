import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api, config, type DeviceState, type Item, type MonthSummary } from "./lib/api";
import { invoke } from "@tauri-apps/api/core";
import { fileName, monthLabel, yearOf } from "./lib/months";
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
import { changeText, marksBlocked, useMarks } from "./lib/marks";
import MarksBar from "./components/MarksBar";
import { addToMonths, markMade, usePreviews, type Done } from "./lib/previews";
import { listen } from "@tauri-apps/api/event";
import { reselect, useIndex, type Anchor } from "./lib/indexer";
import { ARROWS, TILE_MAX, TILE_MIN, clampTile, loadTile, saveTile } from "./lib/grid";
import { moveAcross, parseKey, pickAfterClick, rangeBetween, sameSections, selKey, type Sel, type Section } from "./lib/timeline";

const toSections = (months: MonthSummary[]): Section[] =>
  months.map((m) => ({ month: m.month, count: m.photos + m.videos }));

/** A list's photos for the grid (F003/S04): grouped by month, newest month first. */
type ListView = { id: string; sections: Section[]; byMonth: Map<string, Item[]>; total: number; missing: number };

function toListView(id: string, rows: (Item & { month: string })[], missing: number): ListView {
  const byMonth = new Map<string, Item[]>();
  for (const { month, ...item } of rows) {
    if (!byMonth.has(month)) byMonth.set(month, []);
    byMonth.get(month)!.push(item);
  }
  const sections = [...byMonth.keys()].sort().reverse().map((month) => ({ month, count: byMonth.get(month)!.length }));
  return { id, sections, byMonth, total: rows.length, missing };
}

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
  const marks = useMarks(lists.version);
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

  // A list shown instead of the library (F003/S04), loaded again when the lists or the index change.
  const listId = lists.active?.id ?? null;
  const [listView, setListView] = useState<ListView | null>(null);
  const listViewRef = useRef(listView);
  listViewRef.current = listView;
  const builds = index.state?.builds ?? 0;
  useEffect(() => {
    if (!listId) return setListView(null);
    invoke<{ items: (Item & { month: string })[]; missing: number }>("list_items", { id: listId })
      .then((r) => {
        const next = toListView(listId, r.items, r.missing);
        const prev = listViewRef.current;
        // the same list again (a change here or on the other Mac): keep the selection on its photo
        if (prev?.id === listId) {
          setPicked(new Set());
          setSelected((sel) => {
            const month = sel && prev.sections[sel.section]?.month;
            if (!sel || !month) return sel;
            const path = prev.byMonth.get(month)?.[sel.index]?.path ?? null;
            return reselect({ month, index: sel.index, path }, next.sections, next.byMonth.get(month));
          });
        }
        setListView(next);
      })
      .catch((e) => lists.say(String(e)));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [listId, lists.state, builds]);
  const showingList = listId !== null && listView?.id === listId;

  // The library (or the list) as sections, newest month first, with their counts.
  // The same object while the months and their counts stay the same: new previews only change the
  // month list's preview counts, and a new layout would move the grid (F012/S04).
  const stableSections = useRef<Section[]>([]);
  const sections = useMemo(() => {
    const next = showingList ? listView!.sections : listId ? [] : toSections(months);
    if (!sameSections(stableSections.current, next)) stableSections.current = next;
    return stableSections.current;
  }, [months, listView, listId, showingList]);

  // Another view: start at the top, without a selection.
  useEffect(() => {
    setSelected(null);
    setPicked(new Set());
    setAnchor(null);
    setJump({ section: 0, nonce: Date.now() });
  }, [listId]);
  const counts = useMemo(() => sections.map((s) => s.count), [sections]);

  // Items per month, fetched once when a month comes (nearly) into view.
  const cache = useRef(new Map<string, Item[]>());
  const inFlight = useRef(new Set<string>());
  const [, setLoaded] = useState(0);
  // The sections the grid asked for last: after a rebuild empties the cache with the layout
  // unchanged, the grid doesn't ask again, so the app does.
  const lastWanted = useRef<number[]>([]);
  const [reneed, setReneed] = useState(0);
  const need = useCallback(
    (wanted: number[]) => {
      lastWanted.current = wanted;
      if (listId) return; // a list's items are all loaded at once
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
    [sections, listId],
  );
  useEffect(() => { if (reneed > 0) need(lastWanted.current); }, [reneed]); // eslint-disable-line react-hooks/exhaustive-deps
  const itemsOf = (month: string) => (listId ? listView?.byMonth.get(month) : cache.current.get(month));

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
      let items = listId ? listView?.byMonth.get(month) ?? [] : cache.current.get(month);
      if (!items) {
        items = await api.items(month);
        cache.current.set(month, items);
      }
      if (items[index]) out.push(items[index]);
    }
    return out;
  }, [sections, listId, listView]);
  const pickedItems = useCallback(() => itemsAt(picked), [itemsAt, picked]);

  // Stars and hearts for the selection (F013/S03): the picked tiles, or the selected one.
  const me = device?.device?.owner ?? null;
  const blockedMarks = marksBlocked(marks.state, me);
  const selKeys = useMemo(() => (picked.size > 1 ? [...picked] : selected ? [selKey(selected)] : []), [picked, selected]);
  const selHashes = selKeys
    .map((k) => { const { section, index } = parseKey(k); return itemsOf(sections[section]?.month)?.[index]?.hash; })
    .filter((h): h is string => !!h);
  const applyMarks = useCallback(async (keys: string[], change: { stars: number } | { heart: boolean }) => {
    if (blockedMarks) return lists.say(blockedMarks);
    try {
      const hashes = [...new Set((await itemsAt(keys)).map((i) => i.hash))];
      if (hashes.length === 0) return;
      if ("stars" in change) await marks.rate(hashes, change.stars);
      else await marks.heart(hashes, change.heart);
      lists.say(changeText(hashes.length, change), false);
    } catch (e) {
      lists.say(String(e));
    }
  }, [blockedMarks, itemsAt, marks.rate, marks.heart, lists.say]);
  // 1–5 and 0 set the stars, H the heart; true when the key was one of them
  const markKey = useCallback((e: KeyboardEvent, keys: string[], hearted: boolean): boolean => {
    if (e.metaKey || e.ctrlKey || e.altKey || keys.length === 0) return false;
    if (/^[0-5]$/.test(e.key)) { e.preventDefault(); applyMarks(keys, { stars: Number(e.key) }); return true; }
    if (e.key === "h" || e.key === "H") { e.preventDefault(); applyMarks(keys, { heart: !hearted }); return true; }
    return false;
  }, [applyMarks]);
  const allHearted = !!me && selHashes.length > 0 && selHashes.every((h) => marks.state.hearts[h]?.[me] !== undefined);

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
  // keuze; in een lijst haalt ⌫ de gekozen foto's eruit
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
      } else if (markKey(e, selKeys, allHearted)) {
        // stars or heart
      } else if (e.key === "Escape" && picked.size > 0) { setPicked(new Set()); setAnchor(null); }
      else if ((e.key === " " || e.key === "Enter") && selected !== null) { e.preventDefault(); setOpen(true); }
      else if ((e.key === "Backspace" || e.key === "Delete") && listId) {
        e.preventDefault();
        const id = listId;
        (picked.size > 1 ? pickedItems() : Promise.resolve(current ? [current] : []))
          .then((items) => { if (items.length) return lists.removeFrom(id, items); })
          .catch((err) => setError(String(err)));
      } else if ((e.key === "+" || e.key === "=") && !listId) {
        lists.say("sleep de foto's naar een lijst, of klik rechts: Toevoegen aan", false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, dialog, selected, anchor, picked, cols, counts, move, current, listId, lists.removeFrom, lists.say, pickedItems, markKey, selKeys, allHearted]);

  // After every rebuild: months again, an empty cache, and the selection and the month at the
  // top kept where they were.
  useEffect(() => {
    if (builds === 0) return;
    if (listId) {
      // the list reloads by itself; the library starts over when it is shown again
      ++generation.current;
      cache.current = new Map();
      inFlight.current.clear();
      api.months().then(setMonths).catch((e) => setError(String(e)));
      return;
    }
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
        setReneed((n) => n + 1);
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

  // A month clicked while a list was shown: jump to it once the library is back.
  const pendingMonth = useRef<string | null>(null);
  useEffect(() => {
    if (listId || !pendingMonth.current) return;
    const s = sections.findIndex((x) => x.month === pendingMonth.current);
    if (s < 0) return;
    pendingMonth.current = null;
    setJump({ section: s, nonce: Date.now() });
  }, [listId, sections]);

  // The month on screen gets its previews first (F012/S03).
  const topMonth = topSection?.month ?? null;
  const focusPreviews = previews.focus;
  useEffect(() => { focusPreviews(topMonth); }, [topMonth, focusPreviews]);

  return (
    <div className="relative h-full flex flex-col bg-white dark:bg-neutral-950 text-neutral-900 dark:text-neutral-100">
      <header className="flex items-center gap-3 px-4 h-10 border-b border-neutral-200 dark:border-neutral-800 text-sm">
        {listId && lists.active ? (
          <>
            <button className="text-neutral-400 hover:text-neutral-600" title="terug naar alle foto's" onClick={() => lists.setActive(null)}>
              ← Alle foto's
            </button>
            <span className="font-semibold">{lists.active.name}</span>
            <span className="text-neutral-400">
              {listView ? photosText(listView.total) : "laden…"}
              {topSection && ` · ${monthLabel(topSection.month)} ${yearOf(topSection.month)}`}
              {listView && listView.missing > 0 && ` · ${listView.missing} nog niet op deze Mac`}
            </span>
          </>
        ) : (
          <span className="font-semibold">
            {topSection ? `${monthLabel(topSection.month)} ${yearOf(topSection.month)}` : "FamilyLightbox"}
          </span>
        )}
        {selHashes.length > 0 && (
          <MarksBar
            label={selHashes.length > 1 ? photosText(selHashes.length) : current ? fileName(current.path) : ""}
            hashes={selHashes}
            marks={marks.state}
            me={me}
            blocked={blockedMarks}
            onRate={(stars) => applyMarks(selKeys, { stars })}
            onHeart={(heart) => applyMarks(selKeys, { heart })}
          />
        )}
        {/* Texts that come and go fill the space up to the slider, aligned right, so the slider and
            what follows stay put. */}
        <div className="flex-1 min-w-0 flex items-center justify-end gap-3 overflow-hidden whitespace-nowrap">
          {picked.size > 1 && (
            <span className="text-blue-600" title={listId ? "⌫ of klik rechts haalt ze uit de lijst; Esc wist de keuze" : "sleep ze naar een lijst of klik rechts; Esc wist de keuze"}>
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
        </div>
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
                selected={listId ? null : topSection?.month ?? null}
                onSelect={(m) => {
                  if (listId) {
                    pendingMonth.current = m;
                    lists.setActive(null);
                  } else setJump({ section: sections.findIndex((s) => s.month === m), nonce: Date.now() });
                }}
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
              marked={listId ? lists.members : null}
              marks={marks.state}
              me={device?.device?.owner ?? null}
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
              member={listId ? lists.members.get(current.hash) ?? null : null}
              rating={marks.state.ratings[current.hash]}
              hearts={marks.state.hearts[current.hash]}
              me={me}
              bar={
                <MarksBar
                  label=""
                  hashes={[current.hash]}
                  marks={marks.state}
                  me={me}
                  blocked={blockedMarks}
                  dark
                  onRate={(stars) => applyMarks([selKey(selected)], { stars })}
                  onHeart={(heart) => applyMarks([selKey(selected)], { heart })}
                />
              }
              onMarkKey={(e) => markKey(e, [selKey(selected)], !!me && marks.state.hearts[current.hash]?.[me] !== undefined)}
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
          lists={(lists.state?.lists ?? []).filter((l) => !l.archived && l.id !== listId)}
          onAddTo={(id) => itemsAt(menu.keys).then((items) => lists.addTo(id, items)).catch((e) => setError(String(e)))}
          onClose={() => setMenu(null)}
          actions={[
            ...(menu.keys.length === 1 ? [{ label: "Openen", run: () => { setSelected(parseKey(menu.keys[0])); setOpen(true); } }] : []),
            ...(listId && lists.active
              ? [{
                  label: `Uit ${lists.active.name} halen`,
                  disabled: !!lists.blocked,
                  title: lists.blocked ?? undefined,
                  run: () => { itemsAt(menu.keys).then((items) => lists.removeFrom(listId, items)).catch((e) => setError(String(e))); },
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
