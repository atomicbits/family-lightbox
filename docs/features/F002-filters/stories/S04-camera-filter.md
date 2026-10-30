# Story S04: The camera in the filter bar

**Feature:** `F002-filters`
**Status:** Planned
**Depends on:** S01, S03

## Goal

A camera menu in the filter bar lists the library's cameras with their counts; ticking one or
more shows only their photos.

## Acceptance criteria

- **Light in the bar** (Peter, 2026-10-30: the camera filter will be used less often, the bar
  mustn't get heavy): only a small button "Camera ▾"; the list opens below it on a click, and
  closes on a click outside or Esc.
- **The list**: "Alle camera's" at the top, which clears the choice (no choice means all
  cameras too); below it the cameras by count, most first, with "onbekend", each with a tick box,
  as many ticked as wanted. The list stays open while ticking. Should a drop-down turn out too
  cramped, the list goes in a filter panel that folds open below the bar instead.
- **The button**: "Camera" when none is ticked, the camera's name for one, "3 camera's" for more,
  in the bar's blue when set.
- **Tests** (`pnpm test`): the menu's text and choice.

## Files affected

- `src/components/FilterBar.tsx`, `src/lib/filter.ts`.

## Progress log

### [2026-10-30] Written
