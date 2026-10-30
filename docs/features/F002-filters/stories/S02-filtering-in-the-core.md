# Story S02: Filtering in the Rust core

**Feature:** `F002-filters`
**Status:** Planned
**Depends on:** —

## Goal

The core answers, for a filter, which photos of a month match and how many match per month, from
the index and the marks, fast enough for every change in the bar.

## Acceptance criteria

- **A filter**: stars (≥, =, ≤ n, or none), hearts (anyone's, or a set of people), kinds (photo,
  video, Live Photo), cameras (a set, with "unknown"); an empty part doesn't filter.
- **Commands**: `months(filter)` with each month's count of matches, and `items(month, filter)`;
  without a filter they answer as today.
- **The marks** come from the share or the local copy, as `marks_state`; the frontend's view and
  the core's agree within the 15-second watch.
- **Fast**: a filter over 124 837 items in about 0.2 s.
- **Tests** (`cargo test`): each part, AND across, OR within, the counts.

## Files affected

- `src-tauri/src/filter.rs` (new), `src-tauri/src/db.rs`, `src-tauri/src/lib.rs`.

## Progress log

### [2026-10-30] Written
