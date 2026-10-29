# Story S02: The grid and the counts without hidden photos

**Feature:** `F014-hide-photos`
**Status:** In progress: built and tested; to see in the app once S03 can hide photos
**Depends on:** S01

## Goal

Hidden photos don't appear: not as tiles, not in the counts of the month list and the year rows.

## Acceptance criteria

- **The tiles**: a month's items without the hidden ones; the sections count only what is shown,
  so the layout has no holes.
- **The counts**: per month and per year without the hidden photos; the tooltip says how many are
  hidden. A month where all are hidden stays in the list with 0.
- **A list** (open point 2): shows its hidden photos, dimmed, with a small "verborgen" mark.
- **The other Mac**: hiding or unhiding there changes the grid here within 15 seconds; the
  selection stays on the same photo where it can and the picks are cleared.
- **Tests**: `cargo test` for the hidden counts per month from the index; `pnpm test` for
  filtering a month's items, the sections and the counts.

## How

- A command `count_per_month(hashes)` (`Db::count_per_month`, one pass over the index) counts
  the hidden photos per month; the frontend keeps them as `hidden` on each month
  (`withHidden`), and the sections, the month list and the year rows count `shownCount`.
- `itemsOf` and `itemsAt` leave hidden items out in the library (`visibleItems`), so the tiles
  match the counts; a list keeps them, and the grid dims them with "verborgen" in the middle.
- When the hidden photos or the sections change, the selection moves back to the photo it was on
  (remembered by path when the selection itself moved), and the picks are cleared.

## Files affected

- `src-tauri/src/db.rs`, `src-tauri/src/lib.rs`, `src/App.tsx`, `src/components/MonthList.tsx`,
  `src/components/Grid.tsx`, `src/lib/hidden.ts` and its tests (new), `src/lib/api.ts`,
  `src/lib/months.ts`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `cargo test`: the hidden count per month (photos and videos, not sidecars, a named folder
  under its folder); 127 Rust tests, clippy clean. `pnpm test`: the counts, leaving items out
  (the same array when nothing is hidden), the tooltip text; 80 tests. `tsc` and `pnpm build`
  clean.
- Nothing is hidden on the share yet; S03 adds hiding from the right-click menu.
