# Story S04: A list as a grid, and the other Mac's changes

**Feature:** `F003-lists`
**Status:** Planned
**Depends on:** S03

## Goal

A list opens as a grid of its photos, across months, with who added each one and when; and
what the other Mac changes shows up within about 15 seconds.

## Acceptance criteria

- **A list as a grid**: double-clicking a list shows its photos (by date taken), with the same
  tiles, selection, lightbox and toggle as the months; *Terug* returns to the months where they
  were. A photo whose hash isn't in the index (not synced yet) shows as a placeholder with its
  path hint.
- **Who and when**: the lightbox and the tile's tooltip say "toegevoegd door Lynn, 20 okt".
- **The other Mac**: while the window has focus, the app checks the sizes of the event files on
  the share every 15 seconds and reads again when one grew; the sidebar, the marks and an open
  list update without losing the selection.
- **Tests**: `cargo test` for detecting a grown file; `pnpm test` for the texts.

## Files affected

- `src-tauri/src/lists.rs`, `src-tauri/src/lib.rs`, `src/lib/lists.ts`, `src/App.tsx`,
  `src/components/`.

## Progress log

### [2026-10-29] Written
