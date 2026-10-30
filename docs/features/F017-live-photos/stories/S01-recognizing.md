# Story S01: Recognizing Live Photos when the index is built

**Feature:** `F017-live-photos`
**Status:** Planned
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
