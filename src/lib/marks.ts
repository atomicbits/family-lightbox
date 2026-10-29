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
