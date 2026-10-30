// Stars and hearts in the UI (F013/S01): the merged state from the Rust core by hash, read again
// when the lists' watch sees a change on the share, and writing a rating or a heart.
import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export type Rating = { stars: number; by: string; at: string };

export type MarksState = {
  source: "nas" | "local" | "none";
  ratings: Record<string, Rating>;
  /** Per photo: who put a heart on it, and when. */
  hearts: Record<string, Record<string, string>>;
  /** Hidden photos, with who hid them and when (F014). */
  hidden: Record<string, { by: string; at: string }>;
  /** Each photo in a stack, with its stack's id (F015). */
  stacks: Record<string, string>;
  /** The color chosen for a stack, by id (F015/S05). */
  stack_colors: Record<string, string>;
  ignored: number;
  problem: string | null;
};

/** The size of the marks on a tile, in CSS pixels: readable on the smallest tile, modest on the
 * largest (F013/S02). */
export function markSize(tile: number): number {
  return Math.round(Math.min(18, Math.max(10, tile * 0.075)));
}

/** "★★★★" for 4 stars; nothing for none. */
export function starsText(stars: number | undefined): string {
  return "★".repeat(Math.max(0, Math.min(5, stars ?? 0)));
}

export type HeartView = { mine: boolean; count: number; who: string[] };

/** Whose hearts are on a photo, and whether this Mac's owner's is among them. */
export function heartView(hearts: Record<string, string> | undefined, me: string | null): HeartView | null {
  const who = Object.keys(hearts ?? {}).sort();
  if (who.length === 0) return null;
  return { mine: me !== null && who.includes(me), count: who.length, who };
}

/** The tooltip lines for a photo's marks: "4 sterren (lynn)", "hartje van lynn en peter". */
export function marksLines(rating: Rating | undefined, hearts: Record<string, string> | undefined): string[] {
  const lines: string[] = [];
  if (rating) lines.push(`${rating.stars} ${rating.stars === 1 ? "ster" : "sterren"} (${rating.by})`);
  const who = Object.keys(hearts ?? {}).sort();
  if (who.length) lines.push(`hartje van ${who.length === 1 ? who[0] : `${who.slice(0, -1).join(", ")} en ${who[who.length - 1]}`}`);
  return lines;
}

/** The rating all these photos share (0: none of them has one), or null when they differ. */
export function sharedStars(hashes: string[], ratings: Record<string, Rating>): number | null {
  const values = new Set(hashes.map((h) => ratings[h]?.stars ?? 0));
  return values.size === 1 ? [...values][0] : null;
}

/** Whether this person's heart is on all, some or none of these photos. */
export function heartState(hashes: string[], hearts: Record<string, Record<string, string>>, me: string | null): "all" | "some" | "none" {
  const n = me ? hashes.filter((h) => hearts[h]?.[me] !== undefined).length : 0;
  return n === 0 ? "none" : n === hashes.length ? "all" : "some";
}

/** A click on a star: that rating for all of them, or none when they all have it already. */
export const starsAfterClick = (shared: number | null, clicked: number): number => (shared === clicked ? 0 : clicked);

/** A click on the heart: on for all of them, or off for all when it was on all. */
export const heartAfterClick = (state: "all" | "some" | "none"): boolean => state !== "all";

/** Why stars and hearts can't be changed now, or null. */
export function marksBlocked(s: MarksState, me: string | null): string | null {
  if (s.problem) return s.problem;
  if (s.source !== "nas") return "sterren en hartjes wijzigen kan alleen met de NAS";
  if (!me) return "dit toestel heeft nog geen eigenaar";
  return null;
}

/** "12 foto's: 3 sterren", "1 foto: sterren weg", "hartje op 3 foto's", "hartje weg bij 1 foto". */
export function changeText(n: number, change: { stars: number } | { heart: boolean }): string {
  const photos = n === 1 ? "1 foto" : `${n} foto's`;
  if ("stars" in change) {
    if (change.stars === 0) return `${photos}: sterren weg`;
    return `${photos}: ${change.stars} ${change.stars === 1 ? "ster" : "sterren"}`;
  }
  return change.heart ? `hartje op ${photos}` : `hartje weg bij ${photos}`;
}

export const NO_MARKS: MarksState = { source: "none", ratings: {}, hearts: {}, hidden: {}, stacks: {}, stack_colors: {}, ignored: 0, problem: null };

/** A change to stars, hearts, hiding, stacks or a stack's color. */
export type MarkChange =
  | { op: "rate"; photos: string[]; stars: number }
  | { op: "heart"; photos: string[]; on: boolean }
  | { op: "hide"; photos: string[]; hidden: boolean }
  | { op: "stack"; photos: string[]; stack: string }
  | { op: "unstack"; photos: string[] }
  | { op: "color"; stack: string; color: string };

/** The marks with a change applied here and now, as the share will have it once written: shown at
 * once, before the NAS answers. A stack left with one photo is no stack, as in the merge. */
export function applyLocal(s: MarksState, c: MarkChange, me: string, at: string): MarksState {
  const next: MarksState = { ...s };
  switch (c.op) {
    case "rate": {
      const ratings = { ...s.ratings };
      for (const h of c.photos) { if (c.stars > 0) ratings[h] = { stars: c.stars, by: me, at }; else delete ratings[h]; }
      next.ratings = ratings;
      break;
    }
    case "heart": {
      const hearts = { ...s.hearts };
      for (const h of c.photos) {
        const who = { ...(hearts[h] ?? {}) };
        if (c.on) who[me] = at; else delete who[me];
        if (Object.keys(who).length) hearts[h] = who; else delete hearts[h];
      }
      next.hearts = hearts;
      break;
    }
    case "hide": {
      const hidden = { ...s.hidden };
      for (const h of c.photos) { if (c.hidden) hidden[h] = { by: me, at }; else delete hidden[h]; }
      next.hidden = hidden;
      break;
    }
    case "stack":
    case "unstack": {
      const stacks = { ...s.stacks };
      for (const h of c.photos) { if (c.op === "stack") stacks[h] = c.stack; else delete stacks[h]; }
      const size = new Map<string, number>();
      for (const id of Object.values(stacks)) size.set(id, (size.get(id) ?? 0) + 1);
      for (const [h, id] of Object.entries(stacks)) if ((size.get(id) ?? 0) < 2) delete stacks[h];
      next.stacks = stacks;
      break;
    }
    case "color":
      next.stack_colors = { ...s.stack_colors, [c.stack]: c.color };
      break;
  }
  return next;
}

/** The command and arguments that write a change on the share. */
function command(c: MarkChange): [string, Record<string, unknown>] {
  switch (c.op) {
    case "rate": return ["marks_rate", { photos: c.photos, stars: c.stars }];
    case "heart": return ["marks_heart", { photos: c.photos, on: c.on }];
    case "hide": return ["marks_hide", { photos: c.photos, hidden: c.hidden }];
    case "stack": return ["marks_stack", { photos: c.photos, stack: c.stack }];
    case "unstack": return ["marks_unstack", { photos: c.photos }];
    case "color": return ["marks_color", { stack: c.stack, color: c.color }];
  }
}

/** A new stack's id: this device and the moment, as `marks::new_stack_id` makes it. */
export function newStackId(device: string, now = new Date()): string {
  const p = (n: number, w = 2) => String(n).padStart(w, "0");
  return `${device}-${now.getFullYear()}${p(now.getMonth() + 1)}${p(now.getDate())}T${p(now.getHours())}${p(now.getMinutes())}${p(now.getSeconds())}${p(now.getMilliseconds(), 3)}`;
}

/** How long before the one retry of a write that failed. */
export const RETRY_MS = 1500;

export type Marks = {
  state: MarksState;
  /** Read again now, for undo, which must compare with what is on the share; while writes are
   * under way, the state here, which already has them. */
  reload: () => Promise<MarksState>;
  /** Shows the change at once and writes it on the share in the background; when that fails
   * twice, the marks are read again from the share and `onError` says so. */
  change: (c: MarkChange) => void;
};

/** `version` changes when the watch of `useLists` read the share again; `me` is this Mac's owner. */
export function useMarks(version: number, me: string | null, onError: (message: string) => void): Marks {
  const [state, setState] = useState<MarksState>(NO_MARKS);
  // writes still under way: an answer from the share only replaces the state when it is the last
  const pending = useRef(0);
  const current = useRef(state);
  current.current = state;

  const reload = useCallback(async () => {
    if (pending.current > 0) return current.current;
    const s = await invoke<MarksState>("marks_state");
    setState(s);
    return s;
  }, []);

  useEffect(() => {
    if (pending.current === 0) invoke<MarksState>("marks_state").then((s) => { if (pending.current === 0) setState(s); }).catch(() => {});
  }, [version]);

  const change = useCallback((c: MarkChange) => {
    setState((s) => applyLocal(s, c, me ?? "", new Date().toISOString()));
    pending.current++;
    const [cmd, args] = command(c);
    const write = () => invoke<MarksState>(cmd, args);
    write()
      .catch(() => new Promise<MarksState>((ok, fail) => setTimeout(() => write().then(ok, fail), RETRY_MS)))
      .then(
        (s) => { pending.current--; if (pending.current === 0) setState(s); },
        (e) => {
          pending.current--;
          if (pending.current === 0) reload().catch(() => {});
          onError(`niet bewaard op de NAS, teruggezet: ${String(e)}`);
        },
      );
  }, [me, reload, onError]);

  return { state, reload, change };
}
