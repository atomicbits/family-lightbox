# Story S06: Synced folders

**Feature:** `F018-import`
**Status:** In progress: built and tested; the first plan on real data still to make
**Depends on:** S05

## Goal

Photos from a folder another app keeps (the iPhones through Google Drive) come in without the
app ever writing there, and without reading years of files each time.

## Background

What the two folders look like on Peter's Mac, and what `media import iphone` does with them:

- A sync app on each phone writes `<year>/<month>/<file>` into a Drive folder. Peter's has a
  second copy of recent photos in `iPhone/Recents/`, which the CLI never looks at; Lynn's is a
  Drive shortcut (a symbolic link) to a folder she shares.
- Peter's folder has about 20 000 files from 2003 on, Lynn's about 38 000 from 2011 on. Files
  that aren't downloaded are fetched by Drive when they are read: reading them all once would
  download years of photos.
- The CLI takes the month from the folder (`2026/10`), puts photos in the photos source and
  videos, also the clips of Live Photos, in the videos source, without a day folder, keeps the
  file name, and calls a file new unless that month of the library has the same name with the
  same size in either source. It looks only at the last 6 months unless told otherwise.

## Acceptance criteria

- **Copy only**: nothing in the synced folder is moved, renamed or deleted; there is no
  `_imported/` step.
- **The folders**: with `<year>/<month>/` folders in the synced folder, only the files directly
  in them count (not `iPhone/Recents/`); otherwise every file below it. A synced folder that is a
  symbolic link is followed.
- **Where a file goes**: the month of its folder (or of its date, without month folders); photos
  and sidecars to the photos source, videos to the videos source, each with the layout it has in
  the library (S03): the iPhone sources without a day folder.
- **From which month**: a synced folder takes the months from its *Vanaf* setting (`YYYY/MM`), or
  when that is empty from the first month its photos source has in the library (`iPhone-photos`
  from 2019/11); older files are counted in the plan as *ouder dan …* and never read. A new source
  without *Vanaf* takes every month.
- **What was taken** (decision 7): every file an import took, or found already in the library, is
  recorded with its path, size and modification time in the app's local state
  (`familylightbox-synced.sqlite`). A recorded file is skipped without being read; a file whose
  size or time changed counts again; a file that is gone no longer matters. A photo the phone
  uploads late still comes in.
- **Without reading**: a file whose month in the library has the same name with the same size in
  the photos or videos source is already there (as the CLI); only the other files are read and
  hashed, and they get the checks of S04 (duplicates by hash, `~<hash>` for a conflict).
- **Without Live Photos' clips**, when the synced folder's setting says so: a video with the name
  of a photo in its folder is left out, never read and not recorded, so turning the setting off
  later brings them in.
- **The plan** says how many files were taken before, how many are older than the first month,
  and how many it had to read; *Importeren* is possible when it found only files already in the
  library, so that they are recorded and not looked at again.
- **Tests**: `cargo test` for the folders (month folders, `Recents` left out, a linked root),
  the targets (photo, video, Live Photo clip, sidecar), the first month, the record (skipped,
  changed, recorded after an import), the check without reading, and a whole import from a synced
  folder that leaves it unchanged.

## Out of scope

- Other synced folders than `<year>/<month>/` ones or flat ones.
- Stopping a plan halfway (a long first read); the plan shows its progress.
- `media import` pointing to the app (decision 9): after this works on real data.

## Files affected

- `src-tauri/src/import_plan.rs`, `src-tauri/src/import_job.rs`, `src-tauri/src/lib.rs`,
  `src-tauri/src/db.rs`, `src/components/ImportView.tsx`, `src/lib/importplan.ts`.

## Progress log

### [2026-10-30] Written

### [2026-11-02] Rewritten before building

After looking at both Drive folders and at `media import iphone`: the month from the folder, the
check by name and size, `Recents` left out, the first month, and recording what was found already.

### [2026-11-02] Built

- `import_plan.rs`: `synced_files` (the `<year>/<month>/` folders, or every file without them; a
  linked root followed), `TakenRecord` (`familylightbox-synced.sqlite`: path, size, time) and
  `plan_synced`: before the first month → *older*; recorded → *taken before*; the same name and
  size in that month of either source → already in the library, without reading; the rest read,
  hashed and checked as in S04, with the month of its folder. `SourcePlan` says `synced`,
  `taken_before`, `older`, `from_month`, `read`.
- `import_job.rs`: `Job.synced`; for a synced folder the last step records the files that reached
  the NAS or were there already (`taken`), and moves nothing.
- `db.rs`: `path_sizes`, `first_month`; `lib.rs`: `plan_source` for a synced folder, the record
  in the job's `Env`.
- `config.rs`, `sources.rs`, `ImportSettings.tsx`: *Vanaf* per synced folder, checked as a month.
  Added after counting what a first plan would read on Peter's Mac, by names and sizes only:
  - `iPhoneSync`: 3 718 older than 2019/11, 16 165 already in the library, 499 to read (2.8 GB),
    mostly August to October 2026;
  - Lynn's folder: from 2008/08, 24 354 already in the library, 14 287 to read (72 GB), mostly
    2016–2020, which are in the library under other names or sizes. Drive would download all of
    that, so Lynn's folder wants a *Vanaf*.
- `ImportView.tsx`: a synced folder can be planned and imported; its plan says what was taken
  before, what is older, and how many files were read; the header names both sources.
- `cargo test` 196 passed (4 new: the folders, the plan, the first month, a whole import from a
  synced folder that leaves it as it was and skips everything the next time), `cargo clippy
  --all-targets` clean; `pnpm test` 144 passed (1 new), `tsc` clean.

### [2026-11-03] Leaving out Live Photos' clips

- Peter: maybe the Live Photos' clips shouldn't come in any more. In `iPhoneSync/2026/09` 96 of
  the 99 videos are such clips (0.42 GB); the library has 11 229 of them (49 GB).
- `Synced.skip_live` (*Live Photo-clipjes overslaan* in the settings, off by default):
  `plan_synced` leaves out a video whose name, without extension, is that of a photo in the same
  folder, and counts it as `live_skipped`; the plan says *… Live Photo-clipjes overgeslagen*.
- `cargo test` 196 passed (the plan's test extended), `pnpm test` 144 passed.

### [2026-11-03] The plan's progress moves from the start

- Peter: the plan stayed at *0 / 1 224 bestanden · 0,0 / 11,8 GB* for a long time. Before hashing,
  `plan_synced` read every file's date, which makes Drive download it, without reporting progress.
  Now it hashes first, reporting every file, and reads the dates afterwards from the files Drive
  has downloaded by then.
