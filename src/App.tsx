import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api, config, type DeviceState, type Item, type MonthSummary } from "./lib/api";
import { invoke } from "@tauri-apps/api/core";
import { monthLabel, yearOf } from "./lib/months";
import MonthList from "./components/MonthList";
import Grid from "./components/Grid";
import Lightbox from "./components/Lightbox";
import DeviceSetup from "./components/DeviceSetup";
import FirstStart from "./components/FirstStart";
import ImportView from "./components/ImportView";
import { firstStartSteps } from "./lib/firststart";
import Settings from "./components/Settings";
import NasIndicator from "./components/NasIndicator";
import { nasSummary, useNas } from "./lib/nas";
import { useSync } from "./lib/sync";
import SyncLine from "./components/SyncLine";
import UpdateLine from "./components/UpdateLine";
import { useUpdate } from "./lib/update";
import ConfirmPreviews from "./components/ConfirmPreviews";
import PreviewsLine from "./components/PreviewsLine";
import ListsPanel from "./components/ListsPanel";
import PhotoMenu from "./components/PhotoMenu";
import { photosText, refOf, useLists, type ListDone } from "./lib/lists";
import { hideText, listText, skippedText, stackText, undoHeart, undoHide, undoList, undoStack, undoStars, useUndo } from "./lib/undo";
import Toast from "./components/Toast";
import { shownCount, toggled, visibleItems, withHidden, withRevealed } from "./lib/hidden";
import { NO_FILTER, isSet, matches, readFilter, toCore, type CameraCount, type FilterState } from "./lib/filter";
import FilterBar from "./components/FilterBar";
import { PALETTE, arrange, buildStacks, stackActions, stackColor, stackDelta, type Member as StackMember, type View as StackView } from "./lib/stacks";
import UndoIcon from "./components/UndoIcon";
import ShortcutsOverlay from "./components/ShortcutsOverlay";
import MessagesOverlay from "./components/MessagesOverlay";
import { useMessages } from "./lib/messages";
import { isShortcutsKey } from "./lib/shortcuts";
import { changeText, marksBlocked, newStackId, useMarks } from "./lib/marks";
import MarksBar from "./components/MarksBar";
import { addToMonths, markMade, usePreviews, type Done } from "./lib/previews";
import { listen } from "@tauri-apps/api/event";
import { reselect, useIndex, type Anchor } from "./lib/indexer";
import { ARROWS, TILE_MAX, TILE_MIN, clampTile, loadTile, saveTile } from "./lib/grid";
import { moveAcross, parseKey, pickAfterClick, rangeBetween, sameSections, selKey, type Sel, type Section } from "./lib/timeline";

const toSections = (months: MonthSummary[]): Section[] =>
  months.map((m) => ({ month: m.month, count: shownCount(m) }));

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
  const [showImport, setShowImport] = useState(false);
  const nas = useNas();
  const sync = useSync();
  const index = useIndex();
  const update = useUpdate();
  const previews = usePreviews();
  const lists = useLists();
  const messages = useMessages();
  const [cameras, setCameras] = useState<CameraCount[]>([]);
  const [showMessages, setShowMessages] = useState(false);
  const openMessages = useCallback(() => { setShowMessages(true); messages.seen(); }, [messages.seen]);
  // an index problem shows for a moment in the footer; the details are in the messages (F016/S02)
  const [indexLineShown, setIndexLineShown] = useState(true);
  const indexProblem = index.line?.tone === "amber" ? index.line.text : null;
  useEffect(() => {
    if (!indexProblem) return;
    setIndexLineShown(true);
    const t = setTimeout(() => setIndexLineShown(false), 8000);
    return () => clearTimeout(t);
  }, [indexProblem]);
  const sayError = useCallback((m: string) => {
    lists.say(m);
    invoke("log_message", { level: "warn", source: "sterren en reeksen", text: m }).catch(() => {});
  }, [lists.say]);
  // stars, hearts, hiding and stacks show at once; the NAS is written in the background
  const marks = useMarks(lists.version, device?.device?.owner ?? null, sayError);
  const undo = useUndo();
  // The message at the bottom (F013/S04): what just happened, with Ongedaan maken when it can be.
  const [toast, setToast] = useState<{ text: string; undoable: boolean; id: number } | null>(null);
  const showToast = useCallback((text: string, undoable: boolean) => {
    const id = Date.now();
    setToast({ text, undoable, id });
    setTimeout(() => setToast((t) => (t?.id === id ? null : t)), 6000);
  }, []);
  const [confirming, setConfirming] = useState(false);
  const [cols, setCols] = useState(1);
  const [tile, setTile] = useState(loadTile);
  const changeTile = (n: number) => { const t = clampTile(n); setTile(t); saveTile(t); };
  const lastRefresh = useRef(0);

  // NAS at start, once the device is known: quiet connect without a dialog, then the check.
  // the NAS, the sync and the rest start once this Mac has a device and its local folder (F006/S07)
  const [localChosen, setLocalChosen] = useState<boolean | null>(null);
  const hasDevice = device?.device != null && localChosen === true;
  const welcome = device !== null && device.error === null && (device.device === null || localChosen === false);
  useEffect(() => {
    if (!hasDevice) return;
    lastRefresh.current = Date.now();
    // Then sync from the NAS in the background, once Library is connected (F007/S05).
    nas.start().then((m) => {
      // the shares may have connected just now: lists, stars and hearts from the NAS, changeable
      lists.refresh();
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

  // The cameras for the filter bar (F002/S04), again after every index build.
  useEffect(() => {
    if (!months.length) return;
    invoke<CameraCount[]>("cameras")
      .then(setCameras)
      .catch((e) => invoke("log_message", { level: "warn", source: "filter", text: `camera's niet gelezen: ${e}` }).catch(() => {}));
  }, [months]);

  // At start: show the index there is, and rebuild it in the background (F008/S02). Without an
  // index the grid waits for the first build.
  useEffect(() => {
    config.device().then(setDevice);
    config.localFolders().then((l) => setLocalChosen(l.chosen)).catch(() => setLocalChosen(true));
    const gen = generation.current;
    api.status().then((s) => {
      if (s.index_exists)
        api.months().then((m) => { if (gen === generation.current) setMonths(m); }).catch((e) => setError(String(e)));
      else setWaitingForIndex(true);
      // a new Mac has no working copy until the first sync made it; that sync builds the index
      if (s.library_exists) index.rebuild({ visible: !s.index_exists, onlyIfStale: false }).catch((e) => setError(String(e)));
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

  // Hidden photos (F014/S02): left out of the library and its counts; a list shows them dimmed.
  const hiddenRec = marks.state.hidden;
  const [hiddenPerMonth, setHiddenPerMonth] = useState<Record<string, number>>({});
  useEffect(() => {
    invoke<Record<string, number>>("count_per_month", { hashes: Object.keys(hiddenRec) })
      .then(setHiddenPerMonth)
      .catch(() => {});
  }, [hiddenRec, builds]);
  // Months whose hidden photos are shown for now (F014/S04): only here, until a restart.
  const [revealed, setRevealed] = useState<Set<string>>(new Set());
  const [monthMenu, setMonthMenu] = useState<{ x: number; y: number; kind: "year" | "month"; key: string } | null>(null);
  // Stacks (F015/S02): their members from the index (they may lie in other months than where
  // their stack stands), and which stacks are unfolded in this window.
  const [stackMembers, setStackMembers] = useState<StackMember[]>([]);
  const [unfolded, setUnfolded] = useState<Set<string>>(new Set());
  const stackIds = marks.state.stacks;
  useEffect(() => {
    const hashes = Object.keys(stackIds);
    if (hashes.length === 0) return setStackMembers([]);
    invoke<StackMember[]>("items_by_hash", { hashes }).then(setStackMembers).catch(() => {});
  }, [stackIds, builds]);
  const stackView: StackView = useMemo(() => ({
    stacks: buildStacks(stackMembers, stackIds),
    unfolded,
    marks: marks.state,
    isHidden: (x) => x.hash in hiddenRec && !revealed.has(x.month),
  }), [stackMembers, stackIds, unfolded, marks.state, hiddenRec, revealed]);

  // The filter bar (F002/S03): kept per Mac; the core filters the index, a list is filtered here.
  const [filter, setFilterState] = useState<FilterState>(NO_FILTER);
  const topMonthRef = useRef<string | null>(null);
  useEffect(() => { invoke<unknown>("get_filter").then((v) => setFilterState(readFilter(v))).catch(() => {}); }, []);
  // the month at the top when the filter changes, to come back to once the new counts are in
  const topOnFilter = useRef<string | null>(null);
  const countsFor = useRef<string | null>(null);
  const setFilter = useCallback((f: FilterState) => {
    const month = topMonthRef.current;
    topOnFilter.current = month;
    // a filter that changes no counts moves nothing; don't jump later on something else
    setTimeout(() => { if (topOnFilter.current === month) topOnFilter.current = null; }, 3000);
    setFilterState(f);
    invoke("set_filter", { filter: isSet(f) ? f : null }).catch(() => {});
  }, []);
  const filterOn = isSet(filter);
  const coreFilter = useMemo(() => toCore(filter, marks.state, revealed), [filter, marks.state, revealed]);
  const filterKey = useMemo(() => (filterOn ? JSON.stringify(coreFilter) : ""), [filterOn, coreFilter]);
  const [filteredCounts, setFilteredCounts] = useState<Record<string, number> | null>(null);
  useEffect(() => {
    if (!filterOn) return setFilteredCounts(null);
    const key = filterKey;
    invoke<[string, number][]>("months_filtered", { filter: coreFilter })
      .then((rows) => { countsFor.current = key; setFilteredCounts(Object.fromEntries(rows)); })
      .catch((e) => lists.say(String(e)));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [filterKey, builds]);
  const listShown = useMemo(() => {
    if (!listView || !filterOn) return listView;
    const marksIn = new Set(coreFilter.marks?.hashes ?? []);
    const byMonth = new Map<string, Item[]>();
    for (const [month, items] of listView.byMonth) {
      const keep = items.filter((i) => matches(i, coreFilter, marksIn));
      if (keep.length) byMonth.set(month, keep);
    }
    const sections = listView.sections.filter((s) => byMonth.has(s.month)).map((s) => ({ month: s.month, count: byMonth.get(s.month)!.length }));
    return { ...listView, byMonth, sections };
  }, [listView, filterOn, coreFilter]);
  const fetchItems = useCallback((month: string) =>
    filterOn ? invoke<Item[]>("items_filtered", { month, filter: coreFilter }) : api.items(month),
  // eslint-disable-next-line react-hooks/exhaustive-deps
  [filterKey]);

  // With a filter, a stack keeps only its photos that match, folded by the same rule among those;
  // a stack with none is left out (F002/S03, Peter's choice on 2026-10-30).
  const stacksShown: StackView = useMemo(() => {
    if (!filterOn) return stackView;
    const marksIn = new Set(coreFilter.marks?.hashes ?? []);
    return { ...stackView, isHidden: (x) => stackView.isHidden(x) || !matches(x as unknown as Item, coreFilter, marksIn) };
  }, [filterOn, stackView, coreFilter]);

  const shownMonths = useMemo(() => {
    if (filterOn && filteredCounts) {
      return withRevealed(withHidden(months, hiddenPerMonth), revealed).map((m) => ({ ...m, filtered: filteredCounts[m.month] ?? 0 }));
    }
    const ms = withRevealed(withHidden(months, hiddenPerMonth), revealed);
    return stackView.stacks.size === 0
      ? ms
      : ms.map((m) => { const n = stackDelta(m.month, stackView).stacks; return n === (m.stacks ?? 0) ? m : { ...m, stacks: n }; });
  }, [months, hiddenPerMonth, revealed, stackView, filterOn, filteredCounts]);

  // A month's tiles from its items: hidden ones out (unless shown), stacks together (F014, F015).
  // With a filter set the core has left hidden photos and non-matching ones out already; stacks fold
  // among their matching photos (F002/S03).
  const monthTiles = useCallback((month: string, items: Item[]) =>
    arrange(month, filterOn || revealed.has(month) ? items : visibleItems(items, hiddenRec), stackIds, stacksShown),
  [filterOn, revealed, hiddenRec, stackIds, stacksShown]);

  // The library (or the list) as sections, newest month first, with their counts.
  // The same object while the months and their counts stay the same: new previews only change the
  // month list's preview counts, and a new layout would move the grid (F012/S04).
  const stableSections = useRef<Section[]>([]);
  const sections = useMemo(() => {
    const library = () =>
      toSections(shownMonths).map((s) => (stacksShown.stacks.size ? { ...s, count: s.count + stackDelta(s.month, stacksShown).delta } : s));
    const next = showingList ? listShown!.sections : listId ? [] : library();
    if (!sameSections(stableSections.current, next)) stableSections.current = next;
    return stableSections.current;
  }, [shownMonths, listShown, listId, showingList, stacksShown]);

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
        fetchItems(month)
          .then((items) => {
            if (gen !== generation.current) return;
            cache.current.set(month, items);
            setLoaded((n) => n + 1);
          })
          .catch((e) => setError(String(e)))
          .finally(() => inFlight.current.delete(month));
      }
    },
    [sections, listId, fetchItems],
  );
  // a new filter: the months' items again, through it
  useEffect(() => {
    ++generation.current;
    cache.current = new Map();
    inFlight.current.clear();
    setReneed((n) => n + 1);
  }, [filterKey]);
  useEffect(() => { if (reneed > 0) need(lastWanted.current); }, [reneed]); // eslint-disable-line react-hooks/exhaustive-deps
  const itemsOf = (month: string) => {
    if (listId) return listShown?.byMonth.get(month);
    const items = cache.current.get(month);
    return items && monthTiles(month, items);
  };

  // When photos are hidden or shown again (here or on the other Mac), keep the selection on the
  // same photo; the picks are cleared, their positions may have moved.
  const selectedPath = useRef<{ month: string; path: string } | null>(null);
  const quietSelection = useRef(false);
  useEffect(() => {
    const month = selected ? sections[selected.section]?.month : undefined;
    const item = month ? itemsOf(month)?.[selected!.index] : undefined;
    selectedPath.current = month && item ? { month, path: item.path } : null;
    // only when the selection itself moves
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selected]);
  useEffect(() => {
    setPicked(new Set());
    const sp = selectedPath.current;
    if (listId || !sp) return;
    const section = sections.findIndex((x) => x.month === sp.month);
    const items = itemsOf(sp.month);
    if (section < 0 || !items) return setSelected(null);
    let i = items.findIndex((x) => x.path === sp.path);
    // folded away: the stack's first shown photo
    const stackOf = (path: string) => stackMembers.find((m) => m.path === path && m.hash in stackIds)?.hash;
    const h = stackOf(sp.path);
    if (i < 0 && h) i = items.findIndex((x) => stackIds[x.hash] === stackIds[h]);
    const next = i >= 0 ? { section, index: i } : items.length ? { section, index: Math.min(selected?.index ?? 0, items.length - 1) } : null;
    if (next?.section !== selected?.section || next?.index !== selected?.index) {
      quietSelection.current = true; // moved by the app: the grid stays where the user is
      setSelected(next);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hiddenRec, sections, revealed, stackView]);

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
      let items = listId ? listShown?.byMonth.get(month) ?? [] : cache.current.get(month);
      if (!items) {
        items = await fetchItems(month);
        cache.current.set(month, items);
      }
      if (!listId) items = monthTiles(month, items);
      if (items[index]) out.push(items[index]);
    }
    return out;
  }, [sections, listId, listShown, monthTiles, fetchItems]);

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
      if (hashes.length === 0 || !me) return;
      const text = changeText(hashes.length, change);
      const s = marks.state;
      if ("stars" in change) {
        const before = Object.fromEntries(hashes.map((h) => [h, s.ratings[h]?.stars ?? 0]));
        marks.change({ op: "rate", photos: hashes, stars: change.stars });
        undo.push({ kind: "stars", text, after: change.stars, before });
      } else {
        const before = Object.fromEntries(hashes.map((h) => [h, s.hearts[h]?.[me] !== undefined]));
        marks.change({ op: "heart", photos: hashes, on: change.heart });
        undo.push({ kind: "heart", text, after: change.heart, before });
      }
      showToast(text, true);
    } catch (e) {
      lists.say(String(e));
    }
  }, [blockedMarks, me, itemsAt, marks.state, marks.change, lists.say, undo.push, showToast]);

  // Hiding and showing again, from the right-click menu (F014/S03), with the message and undo.
  const applyHide = useCallback(async (keys: string[], hidden: boolean) => {
    if (blockedMarks) return lists.say(blockedMarks);
    try {
      const hashes = [...new Set((await itemsAt(keys)).map((i) => i.hash))].filter((h) => (h in marks.state.hidden) !== hidden);
      if (hashes.length === 0) return;
      const before = Object.fromEntries(hashes.map((h) => [h, h in marks.state.hidden]));
      marks.change({ op: "hide", photos: hashes, hidden });
      const text = hideText(hashes.length, hidden);
      undo.push({ kind: "hide", text, after: hidden, before });
      showToast(text, true);
    } catch (e) {
      lists.say(String(e));
    }
  }, [blockedMarks, itemsAt, marks.state.hidden, marks.change, lists.say, undo.push, showToast]);

  // The menu's Verbergen / Niet meer verbergen, each for its own photos among those clicked.
  const hideActions = (keys: string[]) => {
    const hashes = keys
      .map((k) => { const { section, index } = parseKey(k); return itemsOf(sections[section]?.month)?.[index]?.hash; })
      .filter((h): h is string => !!h);
    const hiddenOnes = hashes.filter((h) => h in marks.state.hidden).length;
    const common = { disabled: !!blockedMarks, title: blockedMarks ?? undefined };
    return [
      ...(hiddenOnes < hashes.length
        ? [{ label: hiddenOnes ? `Verbergen (${hashes.length - hiddenOnes})` : "Verbergen", run: () => { applyHide(keys, true); }, ...common }]
        : []),
      ...(hiddenOnes > 0
        ? [{ label: hiddenOnes < hashes.length ? `Niet meer verbergen (${hiddenOnes})` : "Niet meer verbergen", run: () => { applyHide(keys, false); }, ...common }]
        : []),
    ];
  };

  // Making, adding to, taking out of and dissolving stacks (F015/S04), with the message and undo.
  const changeStack = useCallback(async (kind: "make" | "add" | "out" | "dissolve", hashes: string[], stack: string | null) => {
    if (blockedMarks) return lists.say(blockedMarks);
    if (hashes.length === 0) return;
    try {
      const before = Object.fromEntries(hashes.map((h) => [h, marks.state.stacks[h] ?? null]));
      const out = kind === "out" || kind === "dissolve";
      const id = kind === "make" ? newStackId(device?.device?.name ?? "app") : stack;
      if (out) marks.change({ op: "unstack", photos: hashes });
      else if (id) marks.change({ op: "stack", photos: hashes, stack: id });
      const after = out ? null : id;
      const text = stackText(kind, hashes.length);
      undo.push({ kind: "stack", text, after, before });
      showToast(text, true);
    } catch (e) {
      lists.say(String(e));
    }
  }, [blockedMarks, marks.state.stacks, marks.change, device, lists.say, undo.push, showToast]);

  // A stack's color (F015/S05), shared, with the message and undo.
  const changeColor = useCallback(async (stack: string, color: string, before: string) => {
    if (blockedMarks) return lists.say(blockedMarks);
    try {
      marks.change({ op: "color", stack, color });
      const text = `reeks ${color} gemaakt`;
      undo.push({ kind: "color", text, stack, after: color, before });
      showToast(text, true);
    } catch (e) {
      lists.say(String(e));
    }
  }, [blockedMarks, marks.change, lists.say, undo.push, showToast]);

  /** The colors to offer when the clicked photos are of one stack. */
  const stackSwatches = (keys: string[]) => {
    if (listId) return undefined;
    const hashes = keys
      .map((k) => { const { section, index } = parseKey(k); return itemsOf(sections[section]?.month)?.[index]?.hash; })
      .filter((h): h is string => !!h);
    const id = stackActions(hashes, stackIds).dissolve;
    if (!id) return undefined;
    const current = stackColor(id, marks.state.stack_colors[id]).name;
    return { colors: PALETTE, current, onPick: (name: string) => { if (name !== current) changeColor(id, name, current); } };
  };

  const stackMenu = (keys: string[]) => {
    if (listId) return [];
    const hashes = keys
      .map((k) => { const { section, index } = parseKey(k); return itemsOf(sections[section]?.month)?.[index]?.hash; })
      .filter((h): h is string => !!h);
    const a = stackActions(hashes, stackIds);
    const common = { disabled: !!blockedMarks, title: blockedMarks ?? undefined };
    const all = (id: string) => Object.keys(stackIds).filter((h) => stackIds[h] === id);
    return [
      ...(a.make ? [{ label: `Reeks maken (${new Set(hashes).size})`, run: () => { changeStack("make", [...new Set(hashes)], null); }, ...common }] : []),
      ...(a.addTo ? [{ label: `Aan de reeks toevoegen (${a.addTo.photos.length})`, run: () => { changeStack("add", a.addTo!.photos, a.addTo!.stack); }, ...common }] : []),
      ...(a.takeOut.length ? [{ label: a.takeOut.length === 1 ? "Uit de reeks halen" : `Uit de reeks halen (${a.takeOut.length})`, run: () => { changeStack("out", a.takeOut, null); }, ...common }] : []),
      ...(a.dissolve ? [{ label: "Reeks opheffen", run: () => { changeStack("dissolve", all(a.dissolve!), null); }, ...common }] : []),
    ];
  };

  // Folding or unfolding the stack of the photo clicked (F015/S02; S03 adds the symbol).
  const foldActions = (keys: string[]) => {
    if (listId || keys.length !== 1) return [];
    const { section, index } = parseKey(keys[0]);
    const h = itemsOf(sections[section]?.month)?.[index]?.hash;
    const id = h ? stackIds[h] : undefined;
    if (!id) return [];
    const open = unfolded.has(id);
    return [{ label: open ? "Reeks dichtklappen" : "Reeks openklappen", run: () => setUnfolded((u) => toggled(u, [id], !open)) }];
  };

  // Adding to and taking out of lists, with the message and an undo step.
  const listChange = useCallback(async (op: "add" | "remove", id: string, keys: Iterable<string>) => {
    try {
      const photos = (await itemsAt(keys)).map(refOf);
      if (photos.length === 0) return;
      const done: ListDone | null = op === "add" ? await lists.addTo(id, photos) : await lists.removeFrom(id, photos);
      if (!done) return;
      const text = listText(done);
      undo.push({ kind: "list", text, done });
      showToast(text, true);
    } catch (e) {
      lists.say(String(e));
    }
  }, [itemsAt, lists.addTo, lists.removeFrom, lists.say, undo.push, showToast]);

  // Undo the newest step: write the values before back, for the photos still as it left them.
  const doUndo = useCallback(async () => {
    const step = undo.pop();
    if (!step) return;
    setToast(null);
    try {
      let skipped = "";
      if (step.kind === "color") {
        const s = await marks.reload();
        const now = stackColor(step.stack, s.stack_colors[step.stack]).name;
        if (now === step.after) marks.change({ op: "color", stack: step.stack, color: step.before });
        else skipped = "de kleur werd intussen gewijzigd, niet teruggezet";
      } else if (step.kind === "list") {
        const inList = await lists.photosOf(step.done.id);
        const p = undoList(step.done, (h) => inList.has(h));
        if (p.photos.length) {
          if (step.done.op === "add") await lists.removeFrom(step.done.id, p.photos);
          else await lists.addTo(step.done.id, p.photos);
        }
        skipped = skippedText(p.skipped, []);
      } else {
        const s = await marks.reload();
        const p =
          step.kind === "stars" ? undoStars(step, s)
          : step.kind === "hide" ? undoHide(step, s)
          : step.kind === "stack" ? undoStack(step, s)
          : undoHeart(step, s, me ?? "");
        for (const [value, hashes] of p.writes as Map<unknown, string[]>) {
          if (step.kind === "stars") marks.change({ op: "rate", photos: hashes, stars: value as number });
          else if (step.kind === "hide") marks.change({ op: "hide", photos: hashes, hidden: value as boolean });
          else if (step.kind === "stack") marks.change(value === null ? { op: "unstack", photos: hashes } : { op: "stack", photos: hashes, stack: value as string });
          else marks.change({ op: "heart", photos: hashes, on: value as boolean });
        }
        skipped = skippedText(p.skipped, p.by);
      }
      showToast(`ongedaan: ${step.text}${skipped ? ` · ${skipped}` : ""}`, false);
    } catch (e) {
      lists.say(String(e));
    }
  }, [undo.pop, lists.photosOf, lists.addTo, lists.removeFrom, lists.say, marks.reload, marks.change, me, showToast]);

  // ⌘? (Ctrl+? elsewhere) shows the shortcuts.
  const [showKeys, setShowKeys] = useState(false);
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (isShortcutsKey(e)) { e.preventDefault(); setShowKeys((v) => !v); }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // ⌘Z (Ctrl-Z on Windows) undoes, also in the lightbox; not in a text field.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key.toLowerCase() === "z") { e.preventDefault(); doUndo(); }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [doUndo]);
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
  const dialog = showSettings || showImport || confirming || welcome;
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
        listChange("remove", listId, selKeys);
      } else if ((e.key === "+" || e.key === "=") && !listId) {
        lists.say("sleep de foto's naar een lijst, of klik rechts: Toevoegen aan", false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, dialog, selected, anchor, picked, cols, counts, move, current, listId, listChange, lists.say, markKey, selKeys, allHearted]);

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
        const items = anchor ? await fetchItems(anchor.month).catch(() => undefined) : undefined;
        // the months on screen are fetched again before the old ones are dropped, so the grid
        // never shows empty tiles in between
        const onScreen = lastWanted.current.map((s) => sections[s]?.month).filter((x): x is string => !!x && x !== anchor?.month);
        const fresh = await Promise.all(
          onScreen.map((month) => fetchItems(month).then((it) => [month, it] as [string, Item[]]).catch(() => null)),
        );
        if (gen !== generation.current) return;
        const next = toSections(withRevealed(withHidden(m, hiddenPerMonth), revealed))
          .map((s) => (stacksShown.stacks.size ? { ...s, count: s.count + stackDelta(s.month, stacksShown).delta } : s));
        cache.current = new Map([
          ...(anchor && items ? [[anchor.month, items] as [string, Item[]]] : []),
          ...fresh.filter((x): x is [string, Item[]] => x !== null),
        ]);
        inFlight.current.clear();
        setError(null);
        setWaitingForIndex(false);
        setMonths(m);
        setReneed((n) => n + 1);
        setSelected(reselect(anchor, next, items && anchor ? monthTiles(anchor.month, items) : undefined));
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
  topMonthRef.current = topSection?.month ?? null;

  // After a filter change, once the counts for it are in: the same month at the top (F002/S03).
  useEffect(() => {
    const month = topOnFilter.current;
    if (!month || (filterOn && countsFor.current !== filterKey)) return;
    topOnFilter.current = null;
    const s = sections.findIndex((x) => x.month === month);
    if (s >= 0) setJump({ section: s, nonce: Date.now() });
  }, [sections, filterOn, filterKey]);

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
      {/* Header: where you are on the left, the stars and heart of the selection in the middle,
          the tile size and undo on the right (F013). */}
      <header className="grid grid-cols-[1fr_auto_1fr] items-center gap-3 px-4 h-10 border-b border-neutral-200 dark:border-neutral-800 text-sm">
        <div className="flex items-center gap-3 min-w-0 overflow-hidden whitespace-nowrap">
          {listId && lists.active ? (
            <>
              <button className="text-neutral-400 hover:text-neutral-600" title="terug naar alle foto's" onClick={() => lists.setActive(null)}>
                ← Alle foto's
              </button>
              <span className="font-semibold">{lists.active.name}</span>
              <span className="text-neutral-400 truncate">
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
        </div>
        <div className="flex items-center justify-center min-w-0">
          <FilterBar
            filter={filter}
            onChange={setFilter}
            cameras={cameras}
            people={[...new Set([...(me ? [me] : []), ...Object.values(marks.state.hearts).flatMap((h) => Object.keys(h))])].sort()}
            count={filterOn ? `${sections.reduce((n, s) => n + s.count, 0).toLocaleString("nl-BE")} van ${months.reduce((n, m) => n + m.photos + m.videos, 0).toLocaleString("nl-BE")}` : null}
          />
        </div>
        <div className="flex items-center justify-end gap-2">
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
          <button
            className="p-1 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-600 dark:text-neutral-300 disabled:opacity-30 disabled:hover:bg-transparent"
            disabled={!undo.top}
            title={undo.top ? `Ongedaan maken: ${undo.top.text} (⌘Z)` : "niets om ongedaan te maken"}
            onClick={doUndo}
            aria-label="Ongedaan maken"
          >
            <UndoIcon />
          </button>
        </div>
      </header>
      {error || waitingForIndex ? (
        <div className="flex-1 grid place-items-center p-8 text-center whitespace-pre-wrap text-neutral-500">
          {error ?? <FirstStart steps={firstStartSteps(sync.state, !!index.state?.running, nasSummary(nas.mounts, nas.check, nas.attempts, nas.busy).state)} />}
        </div>
      ) : (
        <div className="flex-1 min-h-0 flex relative">
          <div className="w-56 shrink-0 flex flex-col">
            <div className="flex-1 min-h-0">
              <MonthList
                months={shownMonths}
                selected={listId ? null : topSection?.month ?? null}
                onMenu={(kind, key, x, y) => setMonthMenu({ kind, key, x, y })}
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
                  listChange("add", id, dragKeys.current);
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
              dimHidden
              inList={!!listId}
              quietSelection={quietSelection}
              activeBar={selHashes.length ? (size) => (
                <MarksBar
                  count={selHashes.length}
                  hashes={selHashes}
                  marks={marks.state}
                  me={me}
                  blocked={blockedMarks}
                  variant={{ tile: size }}
                  onRate={(stars) => applyMarks(selKeys, { stars })}
                  onHeart={(heart) => applyMarks(selKeys, { heart })}
                />
              ) : undefined}
              unfolded={unfolded}
              onToggleStack={(id) => setUnfolded((u) => toggled(u, [id], !u.has(id)))}
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
                  count={1}
                  hashes={[current.hash]}
                  marks={marks.state}
                  me={me}
                  blocked={blockedMarks}
                  variant="lightbox"
                  onRate={(stars) => applyMarks([selKey(selected)], { stars })}
                  onHeart={(heart) => applyMarks([selKey(selected)], { heart })}
                />
              }
              onMenu={(x, y) => setMenu({ x, y, keys: [selKey(selected)] })}
              onMarkKey={(e) => markKey(e, [selKey(selected)], !!me && marks.state.hearts[current.hash]?.[me] !== undefined)}
            />
          ) : (
            <div className="absolute inset-0 z-20 bg-black/95 grid place-items-center text-neutral-400" onClick={() => setOpen(false)}>
              laden…
            </div>
          ))}
        </div>
      )}
      {/* Footer: what runs in the background and messages on the left, the NAS and the settings on
          the right. */}
      <footer className="flex items-center gap-3 px-4 h-7 border-t border-neutral-200 dark:border-neutral-800 text-xs">
        <div className="flex-1 min-w-0 flex items-center gap-3 overflow-hidden whitespace-nowrap">
          {lists.message && (
            <span className={lists.message.error ? "text-amber-600" : "text-neutral-500"}>{lists.message.text}</span>
          )}
          {index.line && (index.line.tone !== "amber" || indexLineShown) && (
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
        {messages.list.length > 0 && (
          <button
            className={"px-1.5 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800 " + (messages.unseen ? "text-amber-600" : "text-neutral-400")}
            title={messages.unseen ? `${messages.unseen} nieuwe meldingen` : "meldingen"}
            onClick={openMessages}
          >
            ⚠{messages.unseen ? ` ${messages.unseen}` : ""}
          </button>
        )}
        {hasDevice && <NasIndicator nas={nas} onOpenSettings={() => setShowSettings(true)} />}
        {hasDevice && (
          <button
            className="px-1.5 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-500"
            title="nieuwe foto's binnenhalen"
            onClick={() => setShowImport(true)}
          >
            ⇣ Importeren
          </button>
        )}
        <button
          className="px-1.5 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800 text-neutral-500"
          title="Instellingen"
          onClick={() => setShowSettings(true)}
        >
          ⚙ Instellingen
        </button>
      </footer>
      {showKeys && <ShortcutsOverlay onClose={() => setShowKeys(false)} />}
      {showMessages && <MessagesOverlay messages={messages} onClose={() => setShowMessages(false)} />}
      {toast && <Toast text={toast.text} onUndo={toast.undoable ? doUndo : null} onClose={() => setToast(null)} />}
      {monthMenu && (() => {
        const ms = shownMonths.filter((m) => (monthMenu.kind === "year" ? yearOf(m.month) === monthMenu.key : m.month === monthMenu.key));
        const hidden = ms.reduce((n, m) => n + (m.hidden ?? 0), 0);
        const shown = ms.length > 0 && ms.filter((m) => m.hidden).every((m) => m.revealed) && hidden > 0;
        const keys = ms.map((m) => m.month);
        return (
          <PhotoMenu
            x={monthMenu.x}
            y={monthMenu.y}
            what={monthMenu.kind === "year" ? monthMenu.key : `${monthLabel(monthMenu.key)} ${yearOf(monthMenu.key)}`}
            lists={[]}
            blocked={null}
            onAddTo={() => {}}
            onClose={() => setMonthMenu(null)}
            actions={[
              shown
                ? { label: "Verborgen foto's weer verbergen", run: () => setRevealed((r) => toggled(r, keys, false)) }
                : {
                    label: `Verborgen foto's tonen (${hidden})`,
                    disabled: hidden === 0,
                    title: hidden === 0 ? "hier is niets verborgen" : "tijdelijk, tot de app herstart",
                    run: () => setRevealed((r) => toggled(r, keys, true)),
                  },
            ]}
          />
        );
      })()}
      {menu && (
        <PhotoMenu
          x={menu.x}
          y={menu.y}
          what={photosText(menu.keys.length)}
          blocked={lists.blocked}
          lists={(lists.state?.lists ?? []).filter((l) => !l.archived && l.id !== listId)}
          onAddTo={(id) => listChange("add", id, menu.keys)}
          onClose={() => setMenu(null)}
          actions={[
            ...(menu.keys.length === 1 && !open ? [{ label: "Openen", run: () => { setSelected(parseKey(menu.keys[0])); setOpen(true); } }] : []),
            ...(listId && lists.active
              ? [{
                  label: `Uit ${lists.active.name} halen`,
                  disabled: !!lists.blocked,
                  title: lists.blocked ?? undefined,
                  run: () => { listChange("remove", listId, menu.keys); },
                }]
              : []),
            ...hideActions(menu.keys),
            ...foldActions(menu.keys),
            ...stackMenu(menu.keys),
          ]}
          swatches={stackSwatches(menu.keys)}
        />
      )}
      {welcome && device && (
        <DeviceSetup nas={nas} device={device.device} onDone={(d) => setDevice({ ...device, device: d })} />
      )}
      {showImport && <ImportView onClose={() => setShowImport(false)} onOpenSettings={() => { setShowImport(false); setShowSettings(true); }} />}
      {showSettings && (
        <Settings device={device} nas={nas} sync={sync} index={index} update={update} previews={previews} onShowMessages={() => { setShowSettings(false); openMessages(); }} onClose={() => { setShowSettings(false); nas.refresh(); }} />
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
