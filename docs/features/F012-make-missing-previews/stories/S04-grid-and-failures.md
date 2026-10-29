# Story S04: The grid shows new previews as they are made; failures in the settings

**Feature:** `F012-make-missing-previews`
**Status:** In progress: tiles checked during the real run; the settings section still to look at
**Depends on:** S03

## Goal

A preview the job of S03 makes replaces its placeholder in the grid right away, without a
reload. What couldn't be made is listed in the settings, with a way to try again.

## Acceptance criteria

- **In the grid**: when a preview of the month on screen is made, its tile shows it within a
  second; other months show it when they are opened. The scroll position and the selection stay.
- **The index**: the item's `preview` flag is set without a full rebuild of the index.
- **In the settings**, a section *Previews*: how many are missing, how many failed, and the list
  of failures (path and error), with *Opnieuw proberen*.
- **Tests**: `pnpm test` for the texts and the tile update; `cargo test` for setting the flag.

## How

- The job collects what it made or copied and hands it over with each progress event (every
  200 ms): `MakeSink::made(&[Done])`, with path and month. The app sets their flag with
  `index::mark_previews` (an `UPDATE` on the app's own index) and emits `previews-made`; the grid
  marks the items in its cache (`markMade`) and the month counts (`addToMonths`). This replaces
  S03's rebuild every 2 minutes; one rebuild still follows a run that made something.
- A build running at the same moment replaces the index file and may miss the previews made
  during it; the next build has them.
- `previews_overview` (missing from the index, failures from the record) and `previews_retry`
  (clears the failures and starts making) for the settings.
- Deliberately not done here: making previews in a separate process or on another Mac.

## Files affected

- `src-tauri/src/db.rs`, `src-tauri/src/index.rs`, `src-tauri/src/previews_plan.rs`,
  `src-tauri/src/previews_job.rs`, `src-tauri/src/lib.rs`, `src/lib/previews.ts` and its tests,
  `src/App.tsx`, `src/components/Settings.tsx`.

## Progress log

### [2026-10-28] Written

### [2026-10-28] Built

- `cargo test`: the flag set without a rebuild and the missing count (`index`), the failures
  listed and cleared (`previews_plan`), the batches with path and month (`previews_job`); 100 Rust
  tests, clippy clean. `pnpm test`: `markMade`, `addToMonths` and `missingText`; 57 tests.
  `pnpm build` clean.
- Still to check in the app, during Peter's first real run: tiles of the month on screen filling
  in, the counts in the settings, *Opnieuw proberen*.

### [2026-10-28] Fixed: the grid jumped back while previews were made

- Peter, during the first real run: scrolling through the months, the grid jumped back to the top
  every time previews were made. Each batch updated the month counts, which made new sections and
  a new layout, and the grid scrolled to the selected tile on every new layout.
- Now the sections stay the same object while the months and their item counts are the same
  (`sameSections`), and the grid scrolls to the selection only when the selection moves or the
  number of columns changes, not on every layout.

### [2026-10-29] Fixed: empty tiles after opening the app

- Peter: right after opening, the grid showed empty tiles until he scrolled. The index rebuild
  at start empties the month cache and loads the months again; since the sections now stay the
  same object (the fix above), the layout didn't change and the grid didn't ask for the visible
  months again, so nothing was fetched until a scroll.
- The app now remembers the sections the grid asked for last and asks for them itself after a
  rebuild.
