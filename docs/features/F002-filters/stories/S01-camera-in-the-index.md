# Story S01: The camera in the app's index

**Feature:** `F002-filters`
**Status:** Done (2026-10-30)
**Depends on:** LibraryLocal F011/S01 (the field `camera` in the manifest, contract 1.3)

## Goal

The app's index reads each file's `camera` from the manifests and keeps it, so the core can
filter on it and list the cameras with their counts.

## Acceptance criteria

- A table `cameras(path, camera)` in the app's index (the `items` table keeps the CLI's columns),
  filled from the manifests' `camera` field; files without it have no row.
- A query for the cameras in the library with their counts, and "unknown".
- **Tests** (`cargo test`): manifests with and without `camera`, the counts.

## Files affected

- `src-tauri/src/index.rs`, `src-tauri/src/db.rs`, `src-tauri/src/lib.rs` (the `cameras` command),
  `src/lib/api.ts` (`Item.camera`).

## Progress log

### [2026-10-30] Written

### [2026-10-30] Done

- `index.rs`: `Entry.camera`; a row in `cameras` for every photo or video with a non-empty camera.
- `db.rs`: `has_cameras` next to `has_live` (an older index has no `cameras` table), `joins()` for
  both left joins, `Item.camera`, and the camera now goes into the filter's `Candidate` in
  `filtered_months` and `filtered_items`. `camera_counts()` gives the cameras of photos and videos
  with their counts, most first, `""` for unknown.
- `lib.rs`: the command `cameras` for the filter bar (S04).
- Test: `the_camera_reaches_the_items_the_counts_and_the_filter`. `cargo test` 144 passed,
  clippy clean, `pnpm test` 112 passed.
- The cameras show up after the next index build, once the manifests on the NAS are at 1.3
  (LibraryLocal F011/S02) and the sync has copied them.
