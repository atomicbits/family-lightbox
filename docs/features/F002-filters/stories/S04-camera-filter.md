# Story S04: The camera in the filter bar

**Feature:** `F002-filters`
**Status:** Built (2026-10-30), to check in the app
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

- `src/components/FilterBar.tsx`, `src/lib/filter.ts`, `src/App.tsx` (loading the cameras).

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `filter.ts`: `cameraButton`, `cameraLabel` (`""` is "onbekend"), `toggleCamera` (`null` is
  "Alle camera's"); `matches` now also checks the camera, for a list's photos.
- `FilterBar.tsx`: `CameraMenu`, a "Camera ▾" chip after the kinds; the list opens below it, at
  most 24 rem high and scrolling, with a tick box per camera and its count; it closes on a click
  outside or Esc and stays open while ticking. A drop-down turned out to be enough: 47 cameras so
  far, which scroll fine.
- `App.tsx`: the cameras come from the `cameras` command (S01), again after every index build;
  a failure goes to the log.
- Tests: `camera (S04)` in `filter.test.ts`. `pnpm test` 115 passed, tsc and lint clean.
- Some camera names are long ("EASTMAN KODAK COMPANY KODAK LS633 ZOOM DIGITAL CAMERA"): the
  button truncates, the list is as wide as the longest. Shortening the make belongs to the
  manifest's `camera_name` (LibraryLocal F011), if Peter wants it.

### [2026-10-30] Most recently used first

Peter asked to keep each camera's last use and sort recent to old by default. `camera_counts`
now also gives the latest `taken` per camera and sorts on it, "onbekend" always last. The menu
shows the month and year of the last use next to the count, with a toggle "recent" / "aantal"
(recent is the default, not kept). `sortCameras` and `lastUsedLabel` in `filter.ts`, with
tests; `pnpm test` 117 and `cargo test` 144 passed.
