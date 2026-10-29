# Story S01: The data: stack and unstack in the marks files

**Feature:** `F015-stacks`
**Status:** Planned
**Depends on:** —

## Goal

A photo's stack is kept as `stack` and `unstack` events in the marks files of feature 13, merged
the same way on every Mac: per photo the last one decides.

## Acceptance criteria

- **Data contract 5.5**: `{"op":"stack","photo":…,"stack":"<id>"}` and
  `{"op":"unstack","photo":…}`; the id is made by the device that makes the stack: its name and
  the moment (`mbp-peter-20261030T211502`), unique without coordination.
- **Merging**: per photo the last `stack` or `unstack` decides; a stack is the photos whose last
  event names it; a stack with fewer than two photos counts as none.
- **Writing**: `marks_stack(photos, stack)` and `marks_unstack(photos)`, one `write` each.
- **Tests** (`cargo test`): making, adding, taking out, the last event wins across devices, a
  stack of one, bad lines.

## Files affected

- `src-tauri/src/marks.rs`, `src-tauri/src/lib.rs`, `src/lib/marks.ts`, `docs/data-contract.md`.

## Progress log

### [2026-10-30] Written
