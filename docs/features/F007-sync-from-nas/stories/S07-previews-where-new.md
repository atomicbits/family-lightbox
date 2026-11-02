# Story S07: The previews pass at start looks only where something can be new

**Feature:** `F007-sync-from-nas`
**Status:** In progress: built and tested; to check at the next start
**Depends on:** S04, S05

## Goal

The sync at start counts the previews only in the years where one can be new, instead of the whole
`Previews` share. On Peter's Mac a full count walks about 249 000 files over SMB at every start,
showing *previews tellen…* for old folders where nothing changes; usually only a handful of years
can have something new.

## Acceptance criteria

- At start, the previews pass counts only the years that:
  - had a manifest copied or changed in this sync's manifests pass, or
  - have photos or videos without a preview on this Mac, by the index, apart from the ones this
    Mac failed to make (the previews record of feature 12): the NAS won't have those either.
- With no such year, the previews pass is skipped: no *previews tellen…*.
- A full count, of every year, still runs:
  - with *Synchroniseren* in the settings;
  - at start when the last full count on this Mac is more than 7 days ago, or there was none, or
    the index can't be read (a new Mac before its first index);
  - and the moment of a full count that finished is kept in the app's state.
- Nothing else changes: the confirmation of a large first pass, the 20 GB reserve, the rebuilds
  during and after the copy.
- **Tests**: `cargo test` for which years a start counts (changed manifests, missing previews,
  both, none), the 7 days, the full count from the settings, and that a full count is recorded.

## How

- `db.rs`: `missing_preview_years`, the years of photos and videos without a preview.
- `runner.rs`: `Jobs` gets the index and whether the pass is full. At start, after the manifests
  pass, a pass that isn't full sets the job's `only` (the engine's top-level folders, S02) to those
  years, or ends without a previews pass when there are none. A full count that finished saves
  `previews_full_at` in `familylightbox-state.json`.
- `lib.rs`: `sync_start` asks for a full pass unless it is the sync at start with a full count in
  the last 7 days.

## Files affected

- `src-tauri/src/db.rs`, `src-tauri/src/runner.rs`, `src-tauri/src/config.rs`,
  `src-tauri/src/lib.rs`, and their tests.

## Progress log

### [2026-11-02] Written

From Peter: *previews tellen…* shows very often, for old folders where nothing changes.

### [2026-11-02] Built

- `runner.rs`: `Scope` (`Full`, or `WhereNew` with the index and the previews record), `full_due`
  (no full count yet, or the last more than 7 days ago) and `years_to_count`. At start the
  previews pass sets the engine's `only` to the years of the changed manifests and of the missing
  previews, or ends without counting when there are none; a full count that finished keeps
  `previews_full_at` in the app's state.
- `db.rs`: `missing_preview_years`, leaving out the previews this Mac failed to make. On Peter's
  Mac all 107 missing previews are such failures (the four `.360` videos among them), so a start
  counts nothing until a manifest changes. Without that, 7 years with about 97 000 previews would
  have been counted at every start.
- `lib.rs`: `sync_start` uses `WhereNew` only for the sync at start with a full count in the last
  7 days; *Synchroniseren* in the settings counts every year.
- `cargo test` 192 passed (3 new: the years to count, the week, and a start that counts nothing,
  then a changed year, then every year by hand), `cargo clippy --all-targets` clean.
- The first start after this still counts in full, since no full count was recorded yet.
