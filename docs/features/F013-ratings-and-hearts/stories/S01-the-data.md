# Story S01: The data: rating and heart events, written and merged

**Feature:** `F013-ratings-and-hearts`
**Status:** Planned
**Depends on:** —

## Goal

Stars and hearts are kept as events in files of their own on the `Lists` share, one per device,
appended only and merged the same way on every Mac, as the lists are. The app reads them with the
lists, notices the other Mac's changes with the same 15-second watch, and can write them. Nothing
shows them yet (S02).

## Acceptance criteria

- **Data contract**, a new section 5.5 *Marks*: `Lists/marks/<device>.jsonl`, written only by that
  device, only by appending, one event per line, with the writing rules of 5.2:
  - `{"v":1,"ts":…,"dev":…,"who":…,"op":"rate","photo":"sha256:…","stars":4}`; `stars` 0–5,
    0 removes the rating;
  - `{"v":1,…,"op":"heart","photo":"sha256:…"}` and `"op":"unheart"`;
  - an optional `path` hint, as with `add`.
- **Merging**, as 5.3: sorted by the moment of `ts`, then `dev`, then line; for each photo the
  last `rate` gives its stars (with who and when), and for each photo and person (`who`) the
  last `heart` or `unheart` decides. Only registered devices; bad, foreign and half-written
  lines are ignored and counted.
- **Writing**: `rate` (several photos, one number of stars) and `heart` / `unheart` (several
  photos), all lines in one `write` with `fsync`; refused without the share, for an unregistered
  device, with no photos, or with stars outside 0–5.
- **Reading**: the state from the share, or the local copy without it; the sync of feature 7
  copies `marks/*.jsonl` too.
- **The watch**: the 15-second fingerprint covers `marks/*.jsonl`, so a change on the other Mac
  reads both the lists and the marks again.
- **Tests** (`cargo test`): the merge (last rating wins across devices and offsets, a rating
  removed, hearts per person, heart then unheart, equal times), bad lines, writing and reading
  back, every refusal, the sync filter, the fingerprint.

## How

- A module `src-tauri/src/marks.rs`, using the line handling and device check of `lists.rs`.
- Commands `marks_state` (stars and hearts by hash, and where they were read from),
  `marks_rate(photos, stars)`, `marks_heart(photos, on)`; `lists_stamp` covers the marks.
- `src/lib/marks.ts`: `useMarks`, read again when the watch of `useLists` sees a change.

## Files affected

- `src-tauri/src/marks.rs` (new), `src-tauri/src/lists.rs`, `src-tauri/src/lib.rs`,
  `src/lib/marks.ts` (new), `src/lib/lists.ts`, `src/App.tsx`, `docs/data-contract.md`.

## Progress log

### [2026-10-30] Written
