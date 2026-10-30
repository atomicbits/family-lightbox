# Story S06: Synced folders

**Feature:** `F018-import`
**Status:** Planned
**Depends on:** S05

## Goal

Photos from a folder another app keeps (the iPhones through Google Drive) come in without the
app ever writing there, and without looking through years of files each time.

## Acceptance criteria

- **Copy only**: nothing in the synced folder is moved, renamed or deleted.
- **What was taken** (decision 7): per file its path, size, modification time and hash, in the
  app's local state; a known file is skipped without being read, a file that is gone is
  forgotten. A photo the phone uploads late still comes in.
- **The historical iPhone layout**: photos to `iPhone-photos[-<Name>]`, videos (also the clips
  of Live Photos) to `iPhone-videos[-<Name>]`, without a day folder, as `media import iphone`.
- **The first time**: files already in the library (by hash, S04) are recorded as taken, so the
  first scan of a long history copies nothing twice.
- Drive's files that are not downloaded yet are read through the mount as any file; the plan
  shows when that will take long.
- After this works on real data, `media import` says to use the app (decision 9).

## Files affected

- `src-tauri/src/import_plan.rs`, `src-tauri/src/sources.rs`.

## Progress log

### [2026-10-30] Written
