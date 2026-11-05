# Story S02: The export job

**Feature:** `F024-export`
**Status:** Done (2026-11-05)
**Depends on:** S01, feature 9

## Acceptance criteria

- From a list's photos and a preset: the originals not here read from the NAS (not fetched to
  this Mac); each photo rendered (or, with `originelen`, copied with its sidecars) to
  `NNN-<name>.jpg`, numbered by date taken, then name; videos copied as they are, Live Photos'
  clips left out.
- Into `Exports/<year>/<year>-<month>-<list slug>[-<preset>]/` on the NAS with an `export.json`
  (data contract 6), or a folder chosen on this Mac; a file there already with that name is kept,
  so a second export adds what is missing.
- A journal like feature 9's, resumable; `export-progress`, `export-done`.
- **Tests**: `cargo test` for the names and order, the folder, `export.json`, resuming.

## Progress log

### [2026-11-05] Built

- `export::parts`: by date taken, then name, numbered (`001-`, wider for more than 999), photos to
  `.jpg`, videos and `originelen` as they are with their sidecars under the same number, Live
  Photos' clips left out. `folder_name`: `<year>/<year>-<month>-<slug>[-<preset>]`.
- `export::Job` and `run`: each part from the working copy, or from `Library` on the NAS when it
  isn't here (no fetching first: the export doesn't need the originals on this Mac), a file there
  already kept, the journal after each, `export.json` at the end (data contract 6).
- `lib.rs`: `export_presets`, `export_plan`, `export_start` (one at a time; by default
  `Exports/…` on the NAS from the layout), `export_pending`; `export-progress`, `export-done`.
- `cargo test` 220 passed (3 new), `cargo clippy --all-targets` clean.

### [2026-11-05] Done

Peter: checked in the app.
