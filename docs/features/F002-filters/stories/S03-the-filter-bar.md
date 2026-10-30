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
  starts over at every change), a list is filtered in the frontend with
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

### [2026-10-30] Stacks stay folded with a filter

- Peter: unfolding every stack when a filter is set confused the grid. With a filter a stack now
  keeps only its photos that match, folded by the same rule among those (hearts or 4–5 stars, else
  the most stars, else the first), standing at its first matching photo; unfolding shows its
  matching photos; a stack with none is left out. Done by giving the stacks' view a `isHidden`
  that also hides what doesn't match (`stacksShown`), so `arrange` and `stackDelta` work as they
  are. One vitest test; 109 tests.

### [2026-10-30] The grid stays where it was when the filter changes

- Peter: with a photo selected in one month and another month on screen, changing the filter
  jumped the grid back to the selected photo. Two causes: the app put the selection back on the
  same photo after the new layout, and the grid scrolls to every new selection; and the layout
  itself changes with the counts.
- A selection the app moves itself (photos hidden, a filter changed) now sets
  `quietSelection`, and the grid doesn't scroll to it. A filter change remembers the month at the
  top and jumps back to it once the counts for that filter are in (or forgets it after 3 s when
  nothing changed).
