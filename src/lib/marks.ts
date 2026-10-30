// Stars and hearts in the UI (F013/S01): the merged state from the Rust core by hash, read again
// when the lists' watch sees a change on the share, and writing a rating or a heart.
import { useCallback, useEffect, useState } from "react";
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

export type Marks = {
  state: MarksState;
  /** Read again now, for undo, which must compare with what is on the share. */
  reload: () => Promise<MarksState>;
  rate: (photos: string[], stars: number) => Promise<MarksState>;
  heart: (photos: string[], on: boolean) => Promise<MarksState>;
  hide: (photos: string[], hidden: boolean) => Promise<MarksState>;
  /** Into this stack, or a new one when `stack` is null. */
  stack: (photos: string[], stack: string | null) => Promise<MarksState>;
  unstack: (photos: string[]) => Promise<MarksState>;
  color: (stack: string, color: string) => Promise<MarksState>;
};

/** `version` changes when the watch of `useLists` read the share again. */
export function useMarks(version: number): Marks {
  const [state, setState] = useState<MarksState>(NO_MARKS);

  useEffect(() => {
    invoke<MarksState>("marks_state").then(setState).catch(() => {});
  }, [version]);

  const apply = useCallback(async (cmd: string, args: Record<string, unknown>) => {
    const s = await invoke<MarksState>(cmd, args);
    setState(s);
    return s;
  }, []);
  const rate = useCallback((photos: string[], stars: number) => apply("marks_rate", { photos, stars }), [apply]);
  const heart = useCallback((photos: string[], on: boolean) => apply("marks_heart", { photos, on }), [apply]);
  const hide = useCallback((photos: string[], hidden: boolean) => apply("marks_hide", { photos, hidden }), [apply]);
  const stack = useCallback((photos: string[], stack: string | null) => apply("marks_stack", { photos, stack }), [apply]);
  const unstack = useCallback((photos: string[]) => apply("marks_unstack", { photos }), [apply]);
  const color = useCallback((stack: string, color: string) => apply("marks_color", { stack, color }), [apply]);

  const reload = useCallback(async () => {
    const s = await invoke<MarksState>("marks_state");
    setState(s);
    return s;
  }, []);

  return { state, reload, rate, heart, hide, stack, unstack, color };
}
