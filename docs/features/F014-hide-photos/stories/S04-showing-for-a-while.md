# Story S04: Showing hidden photos per month or year, until a restart

**Feature:** `F014-hide-photos`
**Status:** In progress: built and tested; to try in the app
**Depends on:** S03

## Goal

Right-click on a month or a year in the month list shows its hidden photos, dimmed, in their
place, and hides them again; only on this Mac and only until the app restarts.

## Acceptance criteria

- **The menu** on a month or a year: *Verborgen foto's tonen (20)*, disabled when there are none;
  *Verborgen foto's weer verbergen* when shown.
- **Shown**: the hidden photos appear in their place, dimmed, with a small mark; the counts
  include them again; the month (or the months of the year) carry a small mark in the month list.
- **Not kept**: nothing is written; after a restart everything hidden is hidden again.
- **Tests** (`pnpm test`): showing a month and a year, the counts, hiding them again.

## How

- `revealed`, a set of months in App's state, never written anywhere. `withRevealed` marks those
  months, `shownCount` counts their hidden photos again, and `itemsOf` / `itemsAt` keep them in
  for those months, so the tiles match the counts.
- The grid dims every hidden photo it shows (only in a list or a shown month do they appear) with
  "verborgen" in the middle; *Niet meer verbergen* from S03 works on them.
- `MonthList` reports a right-click on a year or a month; App opens the same menu as for photos,
  with one item: *Verborgen foto's tonen (N)* (disabled at 0) or *Verborgen foto's weer verbergen*
  when all of them are shown. A year acts on all its months. A shown month, and its year, carry a
  small amber ◉.

## Files affected

- `src/components/MonthList.tsx`, `src/App.tsx`, `src/lib/hidden.ts` and its tests,
  `src/lib/api.ts`, `src/lib/months.ts`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `pnpm test`: a shown month counting its hidden photos again, showing and hiding months a year
  at a time; 83 tests. `tsc` and `pnpm build` clean.
