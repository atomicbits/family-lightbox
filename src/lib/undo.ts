// Undo (F013/S04): every change to stars, hearts or what is in a list, made in this window, as a
// step that remembers the values before; undo writes those back as new events, step by step,
// newest first, leaving alone photos someone changed since.
import { useCallback, useRef, useState } from "react";
import type { ListDone, PhotoRef } from "./lists";
import type { MarksState } from "./marks";

export type Step =
  | { kind: "stars"; text: string; after: number; before: Record<string, number> }
  | { kind: "heart"; text: string; after: boolean; before: Record<string, boolean> }
  | { kind: "hide"; text: string; after: boolean; before: Record<string, boolean> }
  | { kind: "list"; text: string; done: ListDone };

/** What undoing a step writes: per value, the photos to give it; and how many were left alone. */
export type Plan<V> = { writes: Map<V, string[]>; skipped: number; by: string[] };

function group<V>(entries: [string, V][]): Map<V, string[]> {
  const m = new Map<V, string[]>();
  for (const [h, v] of entries) m.set(v, [...(m.get(v) ?? []), h]);
  return m;
}

/** Stars back to what they were, for the photos that still have the rating this step gave. */
export function undoStars(step: Extract<Step, { kind: "stars" }>, s: MarksState): Plan<number> {
  const back: [string, number][] = [];
  const by = new Set<string>();
  let skipped = 0;
  for (const [h, before] of Object.entries(step.before)) {
    const now = s.ratings[h]?.stars ?? 0;
    if (now === step.after) { if (before !== now) back.push([h, before]); }
    else { skipped++; if (s.ratings[h]) by.add(s.ratings[h].by); }
  }
  return { writes: group(back), skipped, by: [...by].sort() };
}

/** My heart back to what it was, for the photos where it is still as this step left it. */
export function undoHeart(step: Extract<Step, { kind: "heart" }>, s: MarksState, me: string): Plan<boolean> {
  const back: [string, boolean][] = [];
  let skipped = 0;
  for (const [h, before] of Object.entries(step.before)) {
    const now = s.hearts[h]?.[me] !== undefined;
    if (now === step.after) { if (before !== now) back.push([h, before]); }
    else skipped++;
  }
  return { writes: group(back), skipped, by: [] };
}

/** Hidden or shown back as it was, for the photos still as this step left them (F014/S03). */
export function undoHide(step: Extract<Step, { kind: "hide" }>, s: MarksState): Plan<boolean> {
  const back: [string, boolean][] = [];
  const by = new Set<string>();
  let skipped = 0;
  for (const [h, before] of Object.entries(step.before)) {
    const now = h in s.hidden;
    if (now === step.after) { if (before !== now) back.push([h, before]); }
    else { skipped++; if (s.hidden[h]) by.add(s.hidden[h].by); }
  }
  return { writes: group(back), skipped, by: [...by].sort() };
}

/** "20 foto's verborgen", "1 foto niet meer verborgen". */
export function hideText(n: number, hidden: boolean): string {
  const photos = n === 1 ? "1 foto" : `${n} foto's`;
  return hidden ? `${photos} verborgen` : `${photos} niet meer verborgen`;
}

/** The photos to take out again after an add, or put back after a remove: those still as left. */
export function undoList(done: ListDone, inList: (photo: string) => boolean): { photos: PhotoRef[]; skipped: number } {
  const still = done.photos.filter((p) => (done.op === "add" ? inList(p.photo) : !inList(p.photo)));
  return { photos: still, skipped: done.photos.length - still.length };
}

/** "1 foto intussen door lynn gewijzigd, niet teruggezet", or "". */
export function skippedText(skipped: number, by: string[]): string {
  if (skipped === 0) return "";
  const photos = skipped === 1 ? "1 foto" : `${skipped} foto's`;
  return `${photos} intussen ${by.length ? `door ${by.join(" en ")} ` : ""}gewijzigd, niet teruggezet`;
}

/** "3 foto's toegevoegd aan Ski", "1 foto uit Ski gehaald". */
export function listText(d: ListDone): string {
  const photos = d.photos.length === 1 ? "1 foto" : `${d.photos.length} foto's`;
  return d.op === "add" ? `${photos} toegevoegd aan ${d.name}` : `${photos} uit ${d.name} gehaald`;
}

export type Undo = {
  top: Step | null;
  push: (s: Step) => void;
  /** Takes the newest step off; the caller writes it back. */
  pop: () => Step | null;
};

export function useUndo(): Undo {
  const steps = useRef<Step[]>([]);
  const [top, setTop] = useState<Step | null>(null);
  const push = useCallback((s: Step) => {
    steps.current.push(s);
    setTop(s);
  }, []);
  const pop = useCallback(() => {
    const out = steps.current.pop() ?? null;
    setTop(steps.current[steps.current.length - 1] ?? null);
    return out;
  }, []);
  return { top, push, pop };
}
