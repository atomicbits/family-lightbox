// Keyboard movement in the grid (F001): left/right by one, up/down by one row.

export type Move = "left" | "right" | "up" | "down";

/**
 * The index after a move, given the column count and the number of items.
 * - Nothing selected yet: any move selects the first item.
 * - Up on the first row and down on the last row stay where they are.
 * - Down into a shorter last row lands on its last item, as in the Finder.
 */
export function moveInGrid(selected: number | null, move: Move, cols: number, count: number): number | null {
  if (count === 0) return null;
  if (selected === null) return 0;
  const c = Math.max(1, cols);
  switch (move) {
    case "left":
      return Math.max(0, selected - 1);
    case "right":
      return Math.min(count - 1, selected + 1);
    case "up":
      return selected - c >= 0 ? selected - c : selected;
    case "down": {
      const lastRow = Math.floor((count - 1) / c);
      if (Math.floor(selected / c) === lastRow) return selected;
      return Math.min(count - 1, selected + c);
    }
  }
}

export const ARROWS: Record<string, Move> = {
  ArrowLeft: "left",
  ArrowRight: "right",
  ArrowUp: "up",
  ArrowDown: "down",
};

/**
 * How many tiles fit next to each other in a scroll box of `clientWidth` (which includes the
 * box's padding of `gap` on both sides, and excludes the scrollbar): n tiles take
 * n * tile + (n - 1) * gap. At least 1.
 */
export function columnsFor(clientWidth: number, tile: number, gap: number): number {
  const inner = clientWidth - 2 * gap;
  return Math.max(1, Math.floor((inner + gap) / (tile + gap)));
}

/** Tile sizes for the size slider, in CSS pixels. Thumbnails are 400 px on their long side, so
 * they stay sharp up to 200 on a Retina screen and soften a little above that. */
export const TILE_MIN = 96;
export const TILE_MAX = 320;
export const TILE_DEFAULT = 176;
const TILE_KEY = "familylightbox.tile";

export function clampTile(n: number): number {
  if (!Number.isFinite(n)) return TILE_DEFAULT;
  return Math.min(TILE_MAX, Math.max(TILE_MIN, Math.round(n)));
}

/** The size chosen last time on this Mac; a convenience only, so any storage problem gives the default. */
export function loadTile(): number {
  try {
    const v = localStorage.getItem(TILE_KEY);
    return v === null ? TILE_DEFAULT : clampTile(Number(v));
  } catch {
    return TILE_DEFAULT;
  }
}

export function saveTile(n: number): void {
  try {
    localStorage.setItem(TILE_KEY, String(clampTile(n)));
  } catch {
    // not remembered; the slider still works
  }
}

/** Width of a full row of `cols` tiles, so the rows can be centred in the panel. */
export function rowWidth(cols: number, tile: number, gap: number): number {
  return cols * tile + (cols - 1) * gap;
}
