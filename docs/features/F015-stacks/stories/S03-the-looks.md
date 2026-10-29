# Story S03: Border, background and the symbol that folds and unfolds

**Feature:** `F015-stacks`
**Status:** Planned
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

## Files affected

- `src/components/Grid.tsx`, `src/components/StackIcon.tsx` (new), `src/lib/stacks.ts`,
  `src/App.tsx`.

## Progress log

### [2026-10-30] Written
