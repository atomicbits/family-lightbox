# Story S02: The grid with stacks together, folded or unfolded

**Feature:** `F015-stacks`
**Status:** Done
**Depends on:** S01

## Goal

A month's tiles put each stack's photos together at the place of its first photo; folded, only
the photos that matter are shown, unfolded all of them.

## Acceptance criteria

- **Order**: a month's items by date taken, except that a stack's photos stand together, in the
  order taken, at the place of the stack's first photo; a stack whose first photo is in another
  month is left out here.
- **Folded** (the default): the photos with a heart or 4–5 stars; without those, the photos with
  the most stars; without any stars or hearts, the first photo.
- **Unfolded**: all photos of the stack; which stacks are unfolded is kept only in this window.
- **Hidden photos** stay out, as in feature 14.
- **Counts**: the sections count the tiles shown, so the layout has no holes; the month list
  keeps counting photos, with the stacks in the tooltip.
- **A list** shows every photo on its own.
- **Selection**: folding or unfolding keeps the selection on the same photo.
- **Tests** (`pnpm test`): the order with interleaved devices, folded with hearts, with 4–5 stars,
  with only lower stars (the highest win), with nothing, unfolded, a stack starting in another month, hidden photos.

## How

- `stacks.ts`: `buildStacks` (members by date taken, then path), `folded` (the rule above),
  `home` (the month of a stack's first visible photo), `tilesOf`, `arrange(month, items, ids,
  view)` and `stackDelta(month, view)` (tiles minus visible stacked photos, and how many stacks
  stand there).
- The members come from the index for every stacked hash (`items_by_hash`), since a stack may
  hold photos of another month; they are read again when the stacks or the index change.
- App's `monthTiles` applies hidden photos (F014) and then the stacks to a month's items; the
  sections add `stackDelta` to the counts, so the layout matches the tiles; the month list keeps
  counting photos and names the stacks in the tooltip.
- Folding is a set of stack ids in App's state. The menu already offers *Reeks openklappen* /
  *dichtklappen* on a photo of a stack (S03 adds the symbol). When a photo is folded away, the
  selection moves to its stack's first shown tile.

## Files affected

- `src/lib/stacks.ts` and its tests (new), `src/App.tsx`, `src/lib/hidden.ts`, `src/lib/api.ts`,
  `src/components/MonthList.tsx`, `src-tauri/src/lib.rs` (`items_by_hash`).

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `pnpm test`: folded to the first photo with the other device's photos left in place, unfolded
  together in the order taken, the folding rule (hearts and 4–5 stars; the highest stars; the
  first), a stack across midnight standing in its first month, a hidden first photo moving the
  stack, the tile counts; the tooltip with stacks. 91 tests; `tsc`, `pnpm build`, clippy clean.
- No stacks exist yet; S04 makes them from the menu.

### [2026-10-30] Checked

- Peter in the app: "de reeks werkt".
