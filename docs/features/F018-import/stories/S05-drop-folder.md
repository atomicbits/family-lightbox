# Story S05: Importing from a drop folder

**Feature:** `F018-import`
**Status:** Planned
**Depends on:** S02, S04

## Goal

The first real replacement of `media import offload` and `media push`: one job takes a drop
folder's new files onto the NAS, verified, with previews, and tidies up after itself.

## Acceptance criteria

- **The steps**, as a background job with progress in the footer (like the previews): copy into
  the staging area `.import/<job>/` in the local library, hash, previews (feature 12), copy to
  the NAS and write the manifest (S02), then move each original to `_imported/` (or delete it,
  by the source's setting), keep locally only the months this Mac keeps, rebuild the index.
- **Resumable**: the job's state is a file in its staging folder; at start an unfinished job is
  found and offered to continue; an original is never moved or deleted before it is verified on
  the NAS.
- **Which months stay local**: a setting, the last 18 months by default, as the CLI's `pull`.
- On Peter's Mac, the same paths and manifests as `media import offload` and `media push` on a
  real source (checked once next to the CLI).
- **Tests**: the steps against temporary folders, interruption after each step, the setting.

## Files affected

- `src-tauri/src/import_job.rs` (new), `src-tauri/src/lib.rs`, `src/App.tsx`.

## Progress log

### [2026-10-30] Written
