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
