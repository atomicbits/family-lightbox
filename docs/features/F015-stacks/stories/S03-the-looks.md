# Story S03: Border, background and the symbol that folds and unfolds

**Feature:** `F015-stacks`
**Status:** Done
**Depends on:** S02

## Goal

A stack's tiles carry a colored border and a light background of that color; when a photo of
the stack is selected, the border is stronger and the stack symbol appears at the top right of
its last shown tile, to fold or unfold it.

## Acceptance criteria

- **Color**: from a palette of soft colors, chosen by the stack's id.
- **Border and background**: around each tile of the stack, so adjacent tiles read as one band,
  also across a row break.
- **Selected**: the border stronger; the symbol (two partly overlapping photo icons, in the
  stack's color) at the top right of the last shown tile; a click unfolds, another folds.
- **The menu**: *Reeks openklappen* / *dichtklappen* on a photo of the stack.
- **In a list**: a thin border of the stack's color, no folding.
- **Tests** (`pnpm test`): the color for an id, which tile carries the symbol.

## How

- `stackColor(id)`: one of eight soft colors (border and a light background), from a hash of the
  id; `lastOfStack(tiles, i, ids)` for the tile that carries the symbol.
- A stack's tile has a 2 px border of its color (3 px when a photo of the stack is selected or
  picked) and 4 px of its light background around the photo; in a list only the 2 px border.
- `StackIcon`: two overlapping photos, the back one filled when folded; a click folds or
  unfolds without selecting the tile. The heart moves left of it on that tile.
- The menu's *Reeks openklappen* / *dichtklappen* came with S02.

## Files affected

- `src/components/Grid.tsx`, `src/components/StackIcon.tsx` (new), `src/lib/stacks.ts`,
  `src/App.tsx`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `pnpm test`: the same color for the same stack and several colors for several, the symbol on
  the last shown tile; 93 tests. `tsc` and `pnpm build` clean.

### [2026-10-30] Checked

- Peter in the app: "de reeks werkt".
