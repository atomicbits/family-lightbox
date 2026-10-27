# Story S03: Previews appear during a long sync, folder by folder

**Feature:** `F008-native-index`
**Status:** In progress: built and tested, checked end to end on real previews; the grid in the app still to check
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

### [2026-10-27] Built

- `runner.rs`:
  - `YearWatch` notices during the copy when it moves on to the next year folder. Only copies
    are reported, so a year that shows up copied something. It asks for a rebuild at most once
    every 2 minutes (`REBUILD_EVERY`);
  - `Sink::year_copied`, with a default that does nothing.
- `lib.rs`: `EventSink::year_copied` asks the index runner of S02 for a rebuild. That runs on its
  own thread, so the copy doesn't wait, and a request during a build is queued as in S02. The
  grid reloads on `index-done` and keeps its selection and the month at the top (S02).
- `cargo test` 70 passed (2 new: the year watch with its 2-minute limit, and a sync of three years
  that asks once), `cargo clippy --all-targets` clean.
- **End to end on real previews**, with a throwaway test (removed afterwards): Peter's local
  previews of 2022/07 and 2023/07 (3 606 files, 1.2 GB) as the remote, an empty local folder, and
  the index built from the real working copy. When the copy moved on to 2023, the index already
  had all 319 previews of 2022/07; after the sync it had 1 803. That test waited for the build
  on purpose, so it could look; the app doesn't wait.
- Still to check by hand in the app: with empty `MEDIA_PREVIEWS` and `MEDIA_CONFIG`, previews
  appearing in the grid while a sync of several years runs. On Peter's Mac a full first sync is
  29 GB, so that check fits best on Lynn's Mac (feature 4).
