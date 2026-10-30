# Story S02: Filtering in the Rust core

**Feature:** `F002-filters`
**Status:** Done
**Depends on:** —

## Goal

The core answers, for a filter, which photos of a month match and how many match per month, from
the index and the marks, fast enough for every change in the bar.

## Acceptance criteria

- **A filter**: stars (≥, =, ≤ n, or none), hearts (anyone's, or a set of people), kinds (photo,
  video, Live Photo), cameras (a set, with "unknown"); an empty part doesn't filter.
- **Commands**: `months(filter)` with each month's count of matches, and `items(month, filter)`;
  without a filter they answer as today.
- **The marks**: the frontend sends the set of photos that pass the stars and hearts (or, for
  "without stars", the set that doesn't), since it shows a change before the NAS has it; so the
  grid and the counts always agree with what is on screen.
- **Hidden photos** (feature 14) are left out of the filtered counts and items too, except in
  months where they are shown; the frontend sends both sets.
- **Fast**: a filter over 124 837 items in about 0.2 s.
- **Tests** (`cargo test`): each part, AND across, OR within, the counts.

## Files affected

- `src-tauri/src/filter.rs` (new), `src-tauri/src/db.rs`, `src-tauri/src/lib.rs`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `filter.rs`: `Filter` (marks set, kinds, cameras, hidden, revealed) and `matches`; a Live
  Photo's video counts as kind `live`, not `video`. `Db::filtered_months` (one pass over the
  index, every month kept with its count, 0 included) and `Db::filtered_items`; commands
  `months_filtered` and `items_filtered`. The camera is `None` until S01 puts it in the index.
- With a filter set, the frontend won't fold stacks (S03): the grid shows exactly the photos
  that match, each with its stack's border.
- 5 Rust tests (an empty filter, marks in and out, kinds with Live Photos, cameras with unknown and
  everything together, the months and items from an index); 143 Rust tests, clippy clean.
- On the real index: the Live Photo filter over 124 837 items in 65 ms (254 months, 6 406 found),
  with an example program `cargo run --release --example filter-speed`.
