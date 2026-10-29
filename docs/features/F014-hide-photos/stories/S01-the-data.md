# Story S01: The data: hide and unhide in the marks files

**Feature:** `F014-hide-photos`
**Status:** Planned
**Depends on:** —

## Goal

Hiding is kept as `hide` and `unhide` events in the marks files of feature 13, merged on every
Mac the same way: the last one decides for a photo, whoever wrote it.

## Acceptance criteria

- **Data contract 5.5**: the ops `hide` and `unhide`, with `photo` and the optional `path`; a
  photo is hidden when its last `hide` or `unhide` is a `hide`.
- **Merging**: in `marks.rs`, by moment, device and line, as stars; `MarksState` gets the hidden
  photos with who hid them and when.
- **Writing**: `marks_hide(photos, hidden)`, all lines in one `write`, refused like stars.
- **Tests** (`cargo test`): hide, unhide, the last one wins across devices, bad lines, writing and
  reading back.

## Files affected

- `src-tauri/src/marks.rs`, `src-tauri/src/lib.rs`, `src/lib/marks.ts`, `docs/data-contract.md`.

## Progress log

### [2026-10-30] Written
