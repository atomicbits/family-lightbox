# Story S04: A list as a grid, and the other Mac's changes

**Feature:** `F003-lists`
**Status:** In progress: built and tested; the other Mac's changes to check once Lynn's Mac has the app
**Depends on:** S03

## Goal

A list opens as a grid of its photos, across months, with who added each one and when; and
what the other Mac changes shows up within about 15 seconds.

## Acceptance criteria

- **A list as a grid**: clicking a list shows its photos (by date taken), with the same
  tiles, selection, lightbox and toggle as the months; *Terug* returns to the months where they
  were. A photo whose hash isn't in the index (not synced yet) shows as a placeholder with its
  path hint.
- **Who and when**: the lightbox and the tile's tooltip say "toegevoegd door Lynn, 20 okt".
- **The other Mac**: while the window has focus, and when it gets it back, the app checks the
  sizes and times of the event files every 15 seconds and reads the lists again when one changed; the sidebar, the marks and an open
  list update without losing the selection.
- **Tests**: `cargo test` for detecting a grown file; `pnpm test` for the texts.

## Files affected

- `src-tauri/src/lists.rs`, `src-tauri/src/lib.rs`, `src/lib/lists.ts`, `src/App.tsx`,
  `src/components/`.

## Progress log

### [2026-10-29] Written

### [2026-10-29] The list as a grid built

- Built ahead, because the active list of S03 confused Peter: `list_items(id)` gives the list's
  photos from the index with their month (`Db::items_by_hash`) and how many hashes aren't in
  the index; the grid shows them by month, newest first, with the same tiles, picking,
  dragging, menu and lightbox. The header shows the list's name, its count and the month at the
  top; a tile's tooltip and the lightbox say who added the photo and when.
- One Rust test for `items_by_hash` (months, order, sidecars left out); 116 Rust tests, 65
  vitest tests, `tsc` and `pnpm build` clean.
- Still to build: noticing the other Mac's changes within 15 seconds.

### [2026-10-29] The other Mac's changes

- `lists::stamp`: a fingerprint of `devices.json` and `events/*.jsonl` (names, sizes, times),
  from the share or the local copy; the command `lists_stamp` returns it as text. Appending
  always makes a file longer, so every event changes it.
- `useLists` checks it every 15 s while the window has focus, and at once when it gets focus
  back; on a change it reads the lists again, and the sidebar, the counts and a list on screen
  follow. A list read again keeps the selection on the same photo (`reselect`, by path) and
  clears the picks, whose positions may have moved.
- Reading everything again instead of only what was appended: a few thousand events merge in
  milliseconds, so it isn't worth the bookkeeping.
- One Rust test for the stamp (no events folder, unchanged, after an append); 117 Rust tests,
  65 vitest tests, `tsc` and `pnpm build` clean.
- To check with two Macs, once Lynn's has the app (F004/S05): a change on one shows on the other
  within 15 seconds.
