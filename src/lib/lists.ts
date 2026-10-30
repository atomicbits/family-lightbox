// The lists in the UI (F003/S03, S04): the merged state from the Rust core, the list on screen
// and its photos, and creating, renaming, archiving, adding and removing.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Item } from "./api";

export type ListInfo = {
  id: string;
  name: string;
  year: number;
  archived: boolean;
  count: number;
  created_by: string;
  created_at: string;
  updated_by: string;
  updated_at: string;
};

export type ListsState = {
  /** nas: changes are possible; local: the copy on this Mac, look only; none: no lists yet. */
  source: "nas" | "local" | "none";
  lists: ListInfo[];
  ignored: number;
  problem: string | null;
};

export type Member = { photo: string; path: string | null; added_by: string; added_at: string };

type Changed = { id: string; state: ListsState };

/** How often the event files are checked for the other Mac's changes, while the window has focus. */
export const WATCH_MS = 15_000;

/** Why the lists can't be changed now, or null. */
export function changeBlocked(s: ListsState | null): string | null {
  if (!s) return "lijsten laden…";
  if (s.problem) return s.problem;
  if (s.source !== "nas") return "lijsten wijzigen kan alleen met de NAS";
  return null;
}

export type YearGroup = { year: number; lists: ListInfo[] };

/** The lists per year, newest year first, archived ones only when asked. */
export function groupLists(lists: ListInfo[], showArchived: boolean): YearGroup[] {
  const groups: YearGroup[] = [];
  for (const l of lists) {
    if (l.archived && !showArchived) continue;
    const last = groups[groups.length - 1];
    if (last && last.year === l.year) last.lists.push(l);
    else groups.push({ year: l.year, lists: [l] });
  }
  return groups;
}

/** The year of a new list: that of the month on screen, else this year. */
export function defaultYear(month: string | null | undefined, now = new Date()): number {
  const y = Number(month?.slice(0, 4));
  return y >= 1900 && y <= 2999 ? y : now.getFullYear();
}

/** "toegevoegd door lynn, 20 okt 2026". */
export function addedText(m: Member): string {
  const d = new Date(m.added_at);
  const when = Number.isNaN(d.getTime())
    ? m.added_at
    : d.toLocaleDateString("nl-BE", { day: "numeric", month: "short", year: "numeric" });
  return `toegevoegd door ${m.added_by}, ${when}`;
}

/** For several photos: add those not in the list, or remove all when all are in it already. */
export function manyChange(items: Item[], members: Map<string, Member>): { op: "add" | "remove"; items: Item[] } {
  const unique = [...new Map(items.map((i) => [i.hash, i])).values()];
  const missing = unique.filter((i) => !members.has(i.hash));
  return missing.length === 0 ? { op: "remove", items: unique } : { op: "add", items: missing };
}

/** "1 foto" or "12 foto's"; videos count as photos here. */
export function photosText(n: number): string {
  return n === 1 ? "1 foto" : `${n} foto's`;
}

export type Lists = {
  state: ListsState | null;
  /** The list shown in the grid instead of the library, or null. */
  active: ListInfo | null;
  /** The photos of that list, by hash. */
  members: Map<string, Member>;
  blocked: string | null;
  /** What just happened, or why it couldn't; `error` for the latter. */
  message: { text: string; error: boolean } | null;
  refresh: () => Promise<void>;
  /** Goes up every time the lists were read again, so stars and hearts follow (F013/S01). */
  version: number;
  /** Show this list in the grid, or the library again with null. */
  setActive: (id: string | null) => void;
  create: (name: string, year: number) => Promise<boolean>;
  rename: (id: string, name: string) => Promise<boolean>;
  archive: (id: string, archived: boolean) => Promise<boolean>;
  /** A short message in the header; `error` for a problem. */
  say: (text: string, error?: boolean) => void;
  /** Adds the photos not in it yet to a list (dragging, the menu, undo), in one write. */
  addTo: (id: string, photos: PhotoRef[]) => Promise<ListDone | null>;
  /** The photos in a list now, by hash. */
  photosOf: (id: string) => Promise<Map<string, Member>>;
  /** Takes the photos that are in it out of a list. */
  removeFrom: (id: string, photos: PhotoRef[]) => Promise<ListDone | null>;
};

/** A photo by hash, with its path as a hint. */
export type PhotoRef = { photo: string; path: string | null };

export const refOf = (i: Item): PhotoRef => ({ photo: i.hash, path: i.path });

/** What a list change actually did. */
export type ListDone = { id: string; name: string; op: "add" | "remove"; photos: PhotoRef[] };

export function useLists(): Lists {
  const [state, setState] = useState<ListsState | null>(null);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [members, setMembers] = useState<Map<string, Member>>(new Map());
  const [message, setMessage] = useState<{ text: string; error: boolean } | null>(null);

  const say = useCallback((text: string, error = true) => {
    if (error) invoke("log_message", { level: "warn", source: "lijsten", text }).catch(() => {});
    const m = { text, error };
    setMessage(m);
    setTimeout(() => setMessage((cur) => (cur === m ? null : cur)), 6000);
  }, []);

  const loadMembers = useCallback(async (id: string | null) => {
    if (!id) return setMembers(new Map());
    const photos = await invoke<Member[]>("list_photos", { id }).catch(() => [] as Member[]);
    setMembers(new Map(photos.map((m) => [m.photo, m])));
  }, []);

  const [version, setVersion] = useState(0);
  const refresh = useCallback(async () => {
    setVersion((v) => v + 1);
    const s = await invoke<ListsState>("lists_state").catch((e) => ({ source: "none", lists: [], ignored: 0, problem: String(e) }) as ListsState);
    setState(s);
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  // a problem reading the lists goes to the log and the messages once (F016)
  const problem = state?.problem ?? null;
  useEffect(() => {
    if (problem) invoke("log_message", { level: "warn", source: "lijsten", text: problem }).catch(() => {});
  }, [problem]);

  // The other Mac's changes (F003/S04): while the window has focus, and when it gets it back,
  // look whether an event file changed, and read the lists again when one did.
  const lastStamp = useRef<string | null>(null);
  useEffect(() => {
    const check = () => {
      if (!document.hasFocus()) return;
      invoke<string | null>("lists_stamp")
        .then((s) => {
          if (lastStamp.current !== null && s !== lastStamp.current) refresh();
          lastStamp.current = s;
        })
        .catch(() => {});
    };
    check();
    const timer = setInterval(check, WATCH_MS);
    window.addEventListener("focus", check);
    return () => { clearInterval(timer); window.removeEventListener("focus", check); };
  }, [refresh]);

  // the active list's photos, again whenever the lists change
  useEffect(() => { loadMembers(activeId); }, [activeId, state, loadMembers]);

  const active = useMemo(() => state?.lists.find((l) => l.id === activeId) ?? null, [state, activeId]);
  const blocked = changeBlocked(state);

  const setActive = useCallback((id: string | null) => setActiveId(id), []);

  const change = useCallback(async (cmd: string, args: Record<string, unknown>): Promise<Changed | null> => {
    try {
      const r = await invoke<Changed>(cmd, args);
      setState(r.state);
      return r;
    } catch (e) {
      say(String(e));
      return null;
    }
  }, [say]);

  const create = useCallback(async (name: string, year: number) => {
    const r = await change("list_create", { name, year });
    if (r) setActive(r.id);
    return r !== null;
  }, [change, setActive]);
  const rename = useCallback(async (id: string, name: string) => (await change("list_rename", { id, name })) !== null, [change]);
  const archive = useCallback(async (id: string, archived: boolean) => {
    const r = await change("list_archive", { id, archived });
    if (r && archived && id === activeId) setActive(null);
    return r !== null;
  }, [change, activeId, setActive]);

  const nameOf = useCallback((id: string) => state?.lists.find((l) => l.id === id)?.name ?? id, [state]);

  const membersOf = useCallback(async (id: string) => id === active?.id
    ? members
    : new Map((await invoke<Member[]>("list_photos", { id }).catch(() => [] as Member[])).map((m) => [m.photo, m])),
  [active, members]);

  // Both answer with what actually changed, for the message and undo (F013/S04), or null.
  const addTo = useCallback(async (id: string, photos: PhotoRef[]): Promise<ListDone | null> => {
    if (blocked) { say(blocked); return null; }
    const inList = await membersOf(id);
    const unique = [...new Map(photos.map((p) => [p.photo, p])).values()];
    const missing = unique.filter((p) => !inList.has(p.photo));
    if (missing.length === 0) { say(`${photosText(unique.length)} al in ${nameOf(id)}`, false); return null; }
    if (!(await change("list_add", { id, photos: missing }))) return null;
    return { id, name: nameOf(id), op: "add", photos: missing };
  }, [blocked, membersOf, change, say, nameOf]);

  const removeFrom = useCallback(async (id: string, photos: PhotoRef[]): Promise<ListDone | null> => {
    if (blocked) { say(blocked); return null; }
    const inList = await membersOf(id);
    const there = [...new Map(photos.map((p) => [p.photo, { photo: p.photo, path: p.path ?? inList.get(p.photo)?.path ?? null }])).values()]
      .filter((p) => inList.has(p.photo));
    if (there.length === 0) { say(`niet in ${nameOf(id)}`, false); return null; }
    if (!(await change("list_remove", { id, photos: there.map((p) => p.photo) }))) return null;
    return { id, name: nameOf(id), op: "remove", photos: there };
  }, [blocked, membersOf, change, say, nameOf]);

  return { state, active, members, blocked, message, refresh, version, setActive, create, rename, archive, say, addTo, removeFrom, photosOf: membersOf };
}
