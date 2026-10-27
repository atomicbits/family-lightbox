# Story S03: Previews appear during a long sync, folder by folder

**Feature:** `F008-native-index`
**Status:** Planned
**Depends on:** S02

## Goal

Lynn's first previews sync takes over an hour. Instead of a grid of "geen preview" tiles until it
ends, the photos appear month by month while they arrive.

## Acceptance criteria

- During the copy phase of a previews sync, the index is rebuilt quietly after every top-level
  year folder that copied something, and at most once every 2 minutes. The grid reloads as in
  S02, without moving the selection or the scroll position.
- The rebuild runs on its own thread, so the copy doesn't wait for it.
- After the sync, the final rebuild of S02 still runs.
- **Tests**: `cargo test` for when a rebuild is asked (a year finished with copies, not more than
  once per 2 minutes, not for a year without copies). Existing tests pass.
- **By hand**: with empty `MEDIA_PREVIEWS` and `MEDIA_CONFIG`, limited to two years, the first
  year's previews appear in the grid before the second year is done.

## How

- `runner.rs` notices when the copy moves on to the next year folder (from the progress of the
  sync engine) and asks the index runner of S02 for a rebuild, throttled.
- Deliberately not done: updating per file or per month, which would rebuild hundreds of times.

## Files affected

- `src-tauri/src/runner.rs`, `src-tauri/src/lib.rs`.

## Progress log

### [2026-10-27] Written
