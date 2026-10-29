// The lists in the UI (F003/S03): the merged state from the Rust core, the active list and its
// photos, and creating, renaming, archiving, adding and removing.
import { useCallback, useEffect, useMemo, useState } from "react";
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

export type Lists = {
  state: ListsState | null;
  active: ListInfo | null;
  /** The photos of the active list, by hash. */
  members: Map<string, Member>;
  blocked: string | null;
  message: string | null;
  refresh: () => Promise<void>;
  setActive: (id: string | null) => void;
  create: (name: string, year: number) => Promise<boolean>;
  rename: (id: string, name: string) => Promise<boolean>;
  archive: (id: string, archived: boolean) => Promise<boolean>;
  /** Puts the item in the active list, or takes it out when it is in it. */
  toggle: (item: Item) => Promise<void>;
  /** The same for several photos at once, in one write (F003/S05). */
  toggleMany: (items: Item[]) => Promise<void>;
};

export function useLists(): Lists {
  const [state, setState] = useState<ListsState | null>(null);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [members, setMembers] = useState<Map<string, Member>>(new Map());
  const [message, setMessage] = useState<string | null>(null);

  const say = useCallback((m: string) => {
    setMessage(m);
    setTimeout(() => setMessage((cur) => (cur === m ? null : cur)), 6000);
  }, []);

  const loadMembers = useCallback(async (id: string | null) => {
    if (!id) return setMembers(new Map());
    const photos = await invoke<Member[]>("list_photos", { id }).catch(() => [] as Member[]);
    setMembers(new Map(photos.map((m) => [m.photo, m])));
  }, []);

  const refresh = useCallback(async () => {
    const s = await invoke<ListsState>("lists_state").catch((e) => ({ source: "none", lists: [], ignored: 0, problem: String(e) }) as ListsState);
    setState(s);
  }, []);

  useEffect(() => {
    refresh();
    invoke<string | null>("active_list").then((id) => setActiveId(id ?? null)).catch(() => {});
  }, [refresh]);

  // the active list's photos, again whenever the lists change
  useEffect(() => { loadMembers(activeId); }, [activeId, state, loadMembers]);

  const active = useMemo(() => state?.lists.find((l) => l.id === activeId) ?? null, [state, activeId]);
  const blocked = changeBlocked(state);

  const setActive = useCallback((id: string | null) => {
    setActiveId(id);
    invoke("set_active_list", { id }).catch(() => {});
  }, []);

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

  const toggle = useCallback(async (item: Item) => {
    if (!active) return say("kies eerst een lijst in de zijbalk");
    if (blocked) return say(blocked);
    if (members.has(item.hash)) await change("list_remove", { id: active.id, photos: [item.hash] });
    else await change("list_add", { id: active.id, photos: [{ photo: item.hash, path: item.path }] });
  }, [active, blocked, members, change, say]);

  const toggleMany = useCallback(async (items: Item[]) => {
    if (!active) return say("kies eerst een lijst in de zijbalk");
    if (blocked) return say(blocked);
    const c = manyChange(items, members);
    if (c.items.length === 0) return;
    if (c.op === "remove") await change("list_remove", { id: active.id, photos: c.items.map((i) => i.hash) });
    else await change("list_add", { id: active.id, photos: c.items.map((i) => ({ photo: i.hash, path: i.path })) });
  }, [active, blocked, members, change, say]);

  return { state, active, members, blocked, message, refresh, setActive, create, rename, archive, toggle, toggleMany };
}
