# Story S01: Recognizing Live Photos when the index is built

**Feature:** `F017-live-photos`
**Status:** Done
**Depends on:** —

## Goal

The index builder pairs each Live Photo video with its photo, and the items the grid reads carry
the photo's path for those videos.

## Acceptance criteria

- **The rule**: a video whose extension is `.MOV` and whose file name ends in `IMG_` or `IMG_E`
  followed by four digits (with any prefix, such as the import's date) is the Live Photo of the
  photo in the same month (`coalesce(month, folder)`) with the same number, taken within 3
  seconds; with several such photos, the nearest in time.
- **Kept**: in a table `live_photos(video, photo)` of the app's index, rebuilt with it.
- **Read**: `items(month)` and `items_by_hash` give each video its `live` photo path, or none.
- **Tests** (`cargo test`): the forms of the number, 3 seconds in and 4 out, another month, an
  `.MP4`, the nearest of two, the table after a build.

## Files affected

- `src-tauri/src/index.rs`, `src-tauri/src/db.rs`, `src/lib/api.ts`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built and checked

- `index.rs`: `img_number` (the four digits after `IMG_` or `IMG_E` at the end of the stem),
  `live_pairs` (per month and number, the nearest photo within `LIVE_SECONDS` = 3, comparing the
  phone's local times), the table `live_photos` filled after the items in the same transaction.
- `db.rs`: `Item.live`, read through a `LEFT JOIN` when the index has the table (an index built
  before F017 doesn't, and gives none); `items` and `items_by_hash` share the columns.
- An example program, `cargo run --release --example index-build`, builds the index into a file
  of choice.
- 3 Rust tests (the forms of the number, the pairing rules, a built index); 138 Rust tests,
  clippy clean.
- On Peter's working copy: 6 406 Live Photos found; the build still takes 1.4 s for 124 837
  items.
