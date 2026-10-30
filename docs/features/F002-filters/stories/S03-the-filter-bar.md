# Story S03: The filter bar in the toolbar

**Feature:** `F002-filters`
**Status:** In progress: built and tested; to try in the app
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

## How

- `filter.ts`: `FilterState` (stars with op and n, hearts anyone or people, kinds, cameras),
  `marksSet` (the stars and hearts as the set of photos that pass; "zonder sterren" alone as the
  rated ones to exclude; "≤ n" means 1 to n stars, the unrated have "zonder sterren"), `toCore`,
  `matches` (the core's test, for a list's photos), `filterText`, `readFilter` (a kept filter
  read back safely).
- `FilterBar.tsx` in the middle of the toolbar: ≥ = ≤, five stars (lighting up to the hovered
  one; a click on the chosen star clears it), *geen*; ♥ *iemand* and the people who put hearts;
  foto, video, Live. Set: a light blue background, "1 234 van 124 837" and ✕.
- App: kept with `get_filter` / `set_filter` in the app's state file. With a filter set, the
  months' counts come from `months_filtered`, the items from `items_filtered` (the month cache
  starts over at every change), stacks aren't folded, a list is filtered in the frontend with
  `matches`; the month list and the year rows count the matches and dim the months with none.

## Files affected

- `src/components/FilterBar.tsx` (new), `src/lib/filter.ts` (new), `src/App.tsx`,
  `src/components/MonthList.tsx`, `src-tauri/src/config.rs`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `pnpm test`: stars by ≥ = ≤ and without, hearts of anyone or of people, stars and hearts
  together, a list's photos matched as in the core, the text, reading a kept filter back; 108
  tests. `tsc`, `pnpm build`, clippy clean.
