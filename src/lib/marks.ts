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

export const NO_MARKS: MarksState = { source: "none", ratings: {}, hearts: {}, ignored: 0, problem: null };

export type Marks = {
  state: MarksState;
  rate: (photos: string[], stars: number) => Promise<MarksState>;
  heart: (photos: string[], on: boolean) => Promise<MarksState>;
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

  return { state, rate, heart };
}
