# Story S05: Importing from a drop folder

**Feature:** `F018-import`
**Status:** Built (2026-10-31); the first real import waits for Peter
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

## How

- `import_job.rs`, one job per source, its state in `<working copy>/.import/<id>/job.json`, saved
  after every step and every few files; each file is `planned`, `staged`, `on_nas` or `done`:
  1. **plan** again (S04), so nothing stale is copied;
  2. **stage**: each file to copy to `.import/<id>/<target>`, through a temporary name, with its
     modification time, and the copy hashed again against the plan;
  3. **NAS**: per month the manifest built from the staged files with the hashes known already
     (`manifest::build` with them as `existing`), and `push::push_folder` (S02); verified files
     are `on_nas`;
  4. **local**: the merged manifest of each month into the working copy; the staged files moved
     there for the months this Mac keeps (`import.keep_months`, 18 by default), the others
     removed from the staging folder;
  5. **source**: each original that is on the NAS, and each duplicate, moved to the drop folder's
     `_imported/` at the same relative path (or deleted when the source says so); empty folders
     removed, the source folder itself kept;
  6. the index rebuilt; the previews follow through F012, which also puts them on the NAS.
- An `ImportRunner` (one job at a time) with progress events, as the sync's; a job is never
  started while a sync runs, and the previews wait for it.
- At start the app looks for `job.json` files that aren't done and offers to continue them in the
  import view.

## Files affected

- `src-tauri/src/import_job.rs` (new), `src-tauri/src/lib.rs`, `src-tauri/src/config.rs`
  (`keep_months`), `src/lib/importjob.ts` (new), `src/components/ImportView.tsx`, `src/App.tsx`,
  `docs/data-contract.md` (section 7).

## Progress log

### [2026-10-30] Written

### [2026-10-31] Built

- `import_job.rs`: `Job` (files `planned` → `staged` → `on_nas` → `done`, or `duplicate`,
  `failed`), saved to `.import/<id>/job.json` after every step; `run` with the steps `stage`
  (copy with its time, the copy hashed against the plan), `nas` (per month `manifest::build` with
  the known hashes, `push::push_folder`; a row that isn't missing or in conflict is on the NAS,
  and its sidecars with it), `local` (the merged manifests into the working copy, the files kept
  for the months from `keep_from(keep_months)`), `source` (to `_imported/` at the same relative
  path, never over a file, or deleted; empty folders pruned, the source folder kept); `pending`.
- `lib.rs`: `plan_source` shared by the plan and the import; `ImportRunner` (one import at a
  time; no sync and no previews meanwhile); `import_start`, `import_pending`, `import_resume`;
  `import-progress`, `import-done`, `import-failed`; the index rebuilt after an import, the
  previews after that (F012, onto the NAS too). A failure goes to the log and the messages.
- `ImportView.tsx`: *Importeren* per source once planned; an unfinished import with *Doorgaan*;
  the progress and the outcome. The footer shows them too (`importjob.ts`).
- Tests: `cargo test` 178 (a whole import onto a temporary "NAS" that has a file of that month
  already: the files and merged manifests there, the manifests and the kept months locally,
  everything in `_imported`; stopped after each step and continued to the same result; without
  the NAS it waits and tidies nothing; a file changed after the plan stays where it is; the
  months kept), `pnpm test` 132; clippy, tsc, build.
- Not yet: the check next to the CLI on a real source, which writes to the NAS.
