# Story S01: The camera in the app's index

**Feature:** `F002-filters`
**Status:** Planned
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

- `src-tauri/src/index.rs`, `src-tauri/src/db.rs`.

## Progress log

### [2026-10-30] Written
