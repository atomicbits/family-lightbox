# Story S04: A quiet start: the grid loads once

**Feature:** `F008-native-index`
**Status:** Done (2026-11-04)
**Depends on:** S02

## Goal

At start the grid shows the index there is, from the local disk, without waiting for the NAS
(S02). On Peter's Mac it still looked as if the photos only came once the NAS was connected: the
grid loaded its tiles again two or three times in the first seconds, the last time about when the
NAS was there. The grid should load once, and only again when something it shows changed.

## Background

What reloaded the grid at start:

1. **The rebuild at start** (S02) always counts as a new build, also when the manifests and the
   local files are as they were. Every build bumps `builds`, and the grid fetches its months and
   tiles again.
2. **The stars, hearts and hidden photos** are read at start from the local copy, and again from
   the share once it is connected (feature 3). They are the same, but the core keeps them in hash
   maps, so they come in another order each time. With a filter on, the filter sent to the core
   lists the hidden photos in that order, its key changes, and the grid throws away the tiles it
   has and fetches them again.

## Acceptance criteria

- A build whose index comes out the same as the one there is (the same rows, flags and Live
  Photos) leaves the index file as it is and doesn't count as a new build: the grid doesn't
  reload. A build that changes something reloads as before.
- Marks read again with the same content don't change the state in the window; with a filter on,
  the same marks give the same filter, in any order the core sends them.
- The rebuild at start still runs, so a preview or original that came or went outside the app (the
  CLI's `previews`, `thin`) is still noticed at start.
- **Tests**: `cargo test` for a rebuild without changes (unchanged, file kept, not counted) and
  one with a change (counted); `pnpm test` for the filter's key not depending on the order of the
  marks. Existing tests pass.
- **By hand**: with a filter on, the app starts with the NAS connected and the grid's tiles show
  once, without going blank again when the NAS check and the lists come in.

## How

- `index.rs`: `write` also computes a digest of what it wrote (every row with its flags, the
  cameras, the Live Photos) and keeps it in a `meta` table. `build` compares it with the digest in
  the index there is; when they are equal it removes the temporary file and reports
  `changed: false`. `IndexRunner` only counts builds that changed the index.
- `marks.rs`: the marks in sorted maps (`BTreeMap`), so they always come in the same order.
- `filter.ts`: `toCore` sorts the hashes it sends, so the key doesn't depend on order either.
- `marks.ts`: a reload whose marks are the same as the state keeps the state as it is.

## Files affected

- `src-tauri/src/index.rs`, `src-tauri/src/marks.rs`, `src/lib/filter.ts`, `src/lib/marks.ts`, and
  their tests.

## Progress log

### [2026-11-02] Written

From Peter's observation that the previews seemed to wait for the NAS at start.

### [2026-11-02] Built

- `index.rs`: `digest` reads an index's tables in the order of their keys and hashes every value.
  `build` compares the new index with the one there is; an equal one is removed and reported with
  `changed: false`, the old file stays. `IndexRunner` counts only builds that changed the index,
  so `builds`, which the grid watches, doesn't move for a build that found nothing. On Peter's
  index (about 250 000 rows) reading both indexes takes well under a second, on the build's own
  thread.
- `marks.rs`: the five maps of `Marks` are `BTreeMap`s, so the core sends them sorted.
- `filter.ts`: `toCore` sorts the hashes of the marks, the hidden photos and the shown months.
- `marks.ts`: `sameMarks`; a reload or a write's answer with the same content keeps the state.
  From the local copy to the share the source changes, so the state does too, but the filter's key
  stays the same.
- `cargo test` 187 passed (1 new: a rebuild without changes keeps the file and isn't counted, a
  preview that came outside the app is), `cargo clippy --all-targets` clean; `pnpm test` 136
  passed (2 new: the filter in any order, marks read again).
- Still to check in the app: with a filter on, a start with the NAS connected loads the tiles once.

### [2026-11-02] Fix: the grid stayed on *foto's laden…*

- At start the months were fetched once, and the answer was dropped when the filter's first key
  bumped the grid's generation meanwhile. Until now the rebuild at start always fetched them
  again; since it no longer counts when nothing changed, they never came. The months at start now
  have a generation of their own (`monthsGen`), bumped only when a rebuild reloads the months.

### [2026-11-04] Checked in the app

Peter checked it in the app: OK.
