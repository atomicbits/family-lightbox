# Story S04: The plan and the import screen

**Feature:** `F018-import`
**Status:** Planned
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

## Files affected

- `src-tauri/src/import_plan.rs` (new), `src/components/ImportView.tsx` (new), `src/App.tsx`.

## Progress log

### [2026-10-30] Written
