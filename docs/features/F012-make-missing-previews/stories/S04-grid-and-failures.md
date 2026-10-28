# Story S04: The grid shows new previews as they are made; failures in the settings

**Feature:** `F012-make-missing-previews`
**Status:** Planned
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

- An event per finished preview (path), batched to a few per second; the grid updates the tiles
  it shows.
- Deliberately not done here: making previews in a separate process or on another Mac.

## Files affected

- `src-tauri/src/db.rs`, `src-tauri/src/previews_job.rs`, `src/App.tsx`, the grid and
  `src/components/Settings.tsx`.

## Progress log

### [2026-10-28] Written
