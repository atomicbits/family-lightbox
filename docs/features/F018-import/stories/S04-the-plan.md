# Story S04: The plan and the import screen

**Feature:** `F018-import`
**Status:** Built (2026-10-31); to check in the app
**Depends on:** S01, S03

## Goal

Before anything is copied, the app shows per source what an import would do, and why.

## Acceptance criteria

- Per source: the new files with their target path (3.1), in which months, how many dates do not
  come from the file itself (`partner`, `name`, `mtime`), duplicates and conflicts.
- **Duplicates by hash** against the whole index: a file whose hash the library has already, by
  any path, is a duplicate and is not copied. To spare hashing, a file whose name and size match
  the target path is hashed first; others only when the plan is made.
- **Conflicts** (3.3): the same target path with another hash gets `~` and eight hexadecimal
  characters of its hash.
- **The screen**: an *Importeren* view from the toolbar with the sources, their counts and
  warnings, one button per source and one for all; nothing happens without that click.
- **Tests**: the target paths for new and historical sources, a day outside the month never
  occurs, duplicates by hash, the conflict name; `pnpm test` for the texts.

## How

- `import_plan.rs`: for one source, the files (without the exclusions of section 1, the CLI's
  `.DS_Store`/`*.sh`/`Icon\r`/`.localized` and the source's own patterns), their dates by 3.4
  (`capture::resolve_with`, each file read once), their hash, and per file what happens:
  - the target path: `<year>/<month>/<source>/[photos|videos/][<day>/]<file>` with the layout of
    the source (S03), the month and day from `taken` as the manifest writes it; a sidecar goes
    where its file goes;
  - **duplicate** when the index has that hash anywhere, or an earlier file of this plan has it;
  - **conflict** when the index has another file at that path: the name gets `~` and the first
    eight hexadecimal characters of the hash;
  - **new** otherwise.
- Hashes are kept per file (path, size, modification time) in `familylightbox-import.sqlite`, so
  planning the same folder again reads only what changed.
- In this story the plan covers drop folders and cards; a synced folder is planned in S06, which
  remembers what was taken instead of hashing years of a phone's history.
- A command `import_plan(source)` on a blocking thread, with progress events (files, bytes).
- `ImportView.tsx`: from the toolbar; the sources found (S03) with *Plan maken*; per source the
  counts (new with GB, already in the library, conflicts), the months, the dates that don't come
  from the file itself by why, and the first files with their target. *Importeren* is shown but
  off until S05.

## Files affected

- `src-tauri/src/import_plan.rs` (new), `src-tauri/src/db.rs` (`path_hashes`), `src-tauri/src/lib.rs`
  (`import_plan`), `src-tauri/examples/plan-check.rs` (new), `src/components/ImportView.tsx` (new),
  `src/lib/importplan.ts` (new), `src/App.tsx`.

## Progress log

### [2026-10-30] Written

### [2026-10-31] Built

- `import_plan.rs`: `glob`, `files` (the exclusions), `HashCache` (`familylightbox-import.sqlite`:
  path, size, modification time, hash), `target`, `conflict_name`, `plan`. A sidecar goes to its
  file's folder; a duplicate is found by hash in the whole library or earlier in the same plan; a
  name taken by other content gets `~<8 hex>` (3.3).
- `lib.rs`: `import_plan(path)` finds the source again (S03), refuses a synced folder (S06), plans
  a card from its `DCIM`, and sends `import-plan-progress` at most five times a second.
- `ImportView.tsx`, opened with *Importeren* in the footer: the sources, *Plan maken*, the
  progress, the summary, the months, the dates that don't come from the file, and the files
  with their target (up to 500, selectable). *Importeren* is shown off until S05.
- Tests: `cargo test` 173 (patterns and targets; new, duplicate in the library and in the plan,
  conflict, a sidecar, exclusions, the counts, the cache), `pnpm test` 130; clippy, tsc, build.
- **On Peter's offload folder** (read only, `examples/plan-check`, a temporary cache): `Hero13` 82
  to copy (93 GB) and 71 already in the library, in 70 s (1.3 GB/s); `Max2` 4 to copy. The same
  counts and months as `media import offload -n`, now by content. Where the CLI took the GoPro
  files' dates from their modification time (exiftool found none), the app reads them from the
  files; the months and days come out the same.
