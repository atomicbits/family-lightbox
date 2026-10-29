# Story S02: The grid and the counts without hidden photos

**Feature:** `F014-hide-photos`
**Status:** Planned
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

- A command `hidden_per_month(hashes)` counting, per month, the index items with those hashes;
  the frontend subtracts them from `months()`.

## Files affected

- `src-tauri/src/db.rs`, `src-tauri/src/lib.rs`, `src/App.tsx`, `src/components/MonthList.tsx`,
  `src/lib/hidden.ts` (new).

## Progress log

### [2026-10-30] Written
