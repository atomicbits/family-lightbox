# Story S03: The filter bar in the toolbar

**Feature:** `F002-filters`
**Status:** Planned
**Depends on:** S02

## Goal

The toolbar's middle holds the filter: stars, hearts and kind, compact, centered above the grid;
a set filter is clear at a glance and cleared with one click; it is kept after a restart.

## Acceptance criteria

- **Stars**: a choice of ≥, = or ≤ and five stars to click (a click on the chosen star clears
  it), and "zonder sterren".
- **Hearts**: ♥ "iemand", or the people to tick.
- **Kind**: foto, video, Live Photo, each a toggle.
- **Set**: the bar's background is tinted and a ✕ clears everything; the grid, the month list
  and a list show only what matches.
- **Kept**: in the app's state file per Mac; back after a restart.
- **Tests** (`pnpm test`): the filter's text, what counts as set, keeping and reading it back.

## Files affected

- `src/components/FilterBar.tsx` (new), `src/lib/filter.ts` (new), `src/App.tsx`,
  `src/components/MonthList.tsx`, `src-tauri/src/config.rs`.

## Progress log

### [2026-10-30] Written
