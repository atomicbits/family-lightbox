# Story S01: The data: hide and unhide in the marks files

**Feature:** `F014-hide-photos`
**Status:** Done
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

### [2026-10-30] Built

- `marks.rs`: `Op::Hide` / `Op::Unhide`, `Marks.hidden` (by hash, with who and when),
  `Change::Hide`; the command `marks_hide(photos, hidden)`; `useMarks().hide` and
  `MarksState.hidden` in the UI. Data contract 5.5 extended.
- 2 tests: hiding shared with the last one deciding (an earlier unhide doesn't undo a later
  hide), writing hide and unhide and reading back, an empty selection refused. 126 Rust tests,
  clippy clean.
