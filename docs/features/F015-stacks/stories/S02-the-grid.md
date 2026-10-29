# Story S02: The grid with stacks together, folded or unfolded

**Feature:** `F015-stacks`
**Status:** Planned
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

## Files affected

- `src/lib/stacks.ts` (new), `src/App.tsx`.

## Progress log

### [2026-10-30] Written
