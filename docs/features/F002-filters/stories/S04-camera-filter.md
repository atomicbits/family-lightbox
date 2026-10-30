# Story S04: The camera in the filter bar

**Feature:** `F002-filters`
**Status:** Planned
**Depends on:** S01, S03

## Goal

A camera menu in the filter bar lists the library's cameras with their counts; ticking one or
more shows only their photos.

## Acceptance criteria

- **The menu**: the cameras by count, most first, with "onbekend"; one or more ticked.
- **In the bar**: "Camera" when none is ticked, the camera's name for one, "3 camera's" for more.
- **Tests** (`pnpm test`): the menu's text and choice.

## Files affected

- `src/components/FilterBar.tsx`, `src/lib/filter.ts`.

## Progress log

### [2026-10-30] Written
