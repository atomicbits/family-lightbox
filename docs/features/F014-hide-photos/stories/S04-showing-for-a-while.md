# Story S04: Showing hidden photos per month or year, until a restart

**Feature:** `F014-hide-photos`
**Status:** Planned
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

## Files affected

- `src/components/MonthList.tsx`, `src/App.tsx`, `src/lib/hidden.ts`, `src/components/Grid.tsx`.

## Progress log

### [2026-10-30] Written
