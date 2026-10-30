# Story S01: The data: rating and heart events, written and merged

**Feature:** `F013-ratings-and-hearts`
**Status:** Done
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
- `src/lib/marks.ts`: `useMarks(version)`, read again when `useLists` reads the share again
  (its new `version`); App uses it from S02 on.
- `lists.rs` lends `complete_lines`, `moment` and `append_lines` (the device check and the one
  `write` with `fsync`); its `stamp` and sync filter cover `marks/` too.

## Files affected

- `src-tauri/src/marks.rs` (new), `src-tauri/src/lists.rs`, `src-tauri/src/lib.rs`,
  `src/lib/marks.ts` (new), `src/lib/lists.ts`, `src/App.tsx`, `docs/data-contract.md`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- 7 tests in `marks.rs`: the last rating wins across devices and offsets, a rating removed with
  0, hearts per person (and an `unheart` without a heart), equal times by device and line, seven
  kinds of bad lines, writing and reading back (a stranger's file not read), every refusal with
  nothing written, the watch and the sync filter. 124 Rust tests, clippy clean; 65 vitest tests,
  `tsc` and `pnpm build` clean.
- Data contract 5.5 written; the app's sync copies `marks/*.jsonl`.
- Nothing written to the real share yet: the first stars come from the app in S03.

### [2026-10-30] Fixed: changes refused with the NAS connected

- Peter: making a stack was refused ("alleen met de NAS") while the footer said "NAS verbonden".
  At start the app read the lists and marks before it had connected the `Lists` share, so from
  the local copy (look only); the 15-second watch compared only the files, which were the same
  on the share, so it never read them again from there.
- The fingerprint (`lists_stamp`) now names its source, and the app reads the lists and marks
  again as soon as the NAS is connected at start.
