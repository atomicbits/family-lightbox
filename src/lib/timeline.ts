// The whole library as one grid (F001/S01): every month is a section with a header row and
// rows of tiles. Everything here is computed from the counts alone, so the layout is known
// before any month's items are loaded.
import { moveInGrid, type Move } from "./grid";

export type Section = { month: string; count: number };
export type Row = { kind: "header"; section: number } | { kind: "tiles"; section: number; row: number };
export type Layout = { rows: Row[]; headerRow: number[] };
/** A selected tile: section index and position within that section. */
export type Sel = { section: number; index: number };

export function buildLayout(sections: Section[], cols: number): Layout {
  const c = Math.max(1, cols);
  const rows: Row[] = [];
  const headerRow: number[] = [];
  sections.forEach((s, section) => {
    headerRow.push(rows.length);
    rows.push({ kind: "header", section });
    for (let row = 0; row < Math.ceil(s.count / c); row++) rows.push({ kind: "tiles", section, row });
  });
  return { rows, headerRow };
}

/** The layout row that holds a selected tile. */
export function rowOf(layout: Layout, sel: Sel, cols: number): number {
  return layout.headerRow[sel.section] + 1 + Math.floor(sel.index / Math.max(1, cols));
}

/** The sections (indices, in order, no repeats) that have a row between first and last. */
export function sectionsInRange(layout: Layout, first: number, last: number): number[] {
  const out: number[] = [];
  for (let i = Math.max(0, first); i <= Math.min(last, layout.rows.length - 1); i++) {
    const s = layout.rows[i].section;
    if (out[out.length - 1] !== s) out.push(s);
  }
  return out;
}

/**
 * A move that may cross into the month above or below. Within a section it is moveInGrid;
 * at an edge it continues in the neighbouring section, in the same column where possible.
 */
export function moveAcross(sel: Sel | null, move: Move, cols: number, counts: number[]): Sel | null {
  const last = counts.length - 1;
  if (last < 0) return null;
  if (sel === null) return { section: 0, index: 0 };
  const c = Math.max(1, cols);
  const { section: s, index: i } = sel;
  const n = counts[s];
  const inside = moveInGrid(i, move, c, n);
  if (inside !== null && inside !== i) return { section: s, index: inside };

  const col = i % c;
  switch (move) {
    case "left":
      return s > 0 ? { section: s - 1, index: counts[s - 1] - 1 } : sel;
    case "right":
      return s < last ? { section: s + 1, index: 0 } : sel;
    case "up": {
      if (s === 0) return sel;
      const p = counts[s - 1];
      const lastRowStart = Math.floor((p - 1) / c) * c;
      return { section: s - 1, index: Math.min(p - 1, lastRowStart + col) };
    }
    case "down": {
      // moveInGrid stays put on the last row; a short last row below already moved it there.
      if (Math.floor(i / c) !== Math.floor((n - 1) / c)) return sel;
      if (s === last) return sel;
      return { section: s + 1, index: Math.min(counts[s + 1] - 1, col) };
    }
  }
}
