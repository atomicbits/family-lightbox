# Story S03: Sources and their settings; the name proposed from the library

**Feature:** `F018-import`
**Status:** Built (2026-10-31); to check in the app
**Depends on:** —

## Goal

The app knows where new files come from on this Mac: the subfolders of a drop folder, synced
folders, and cards or devices with `DCIM/`; each with a source name and its few settings.

## Acceptance criteria

- **Settings**, kept on this Mac (`~/.config/media/`, data contract 7): drop folders, synced
  folders with the source they feed (one for photos and one for videos for the historical iPhone
  sources), per source: exclusions, "ignore", and "delete from `_imported/` once verified on the
  NAS" (off by default).
- **The layout of a source comes from the library**, like its name (decision 6), so no source
  names are in the code: from the index, per source, whether its files are under
  `photos/`/`videos/` (split) and whether under a day folder `YYYY-MM-DD/`, by what most of its
  files do. A source the library doesn't have yet gets `<source>/<day>/`. Data contract 3.1 says
  so instead of naming this library's exceptions.
- **A card**: a mounted volume with `DCIM/` is seen when it appears; the app reads the camera of
  a few files (S01) and proposes the source under which the index has most photos of that camera;
  otherwise a name from the model (`GoPro HERO13 Black` → `GoPro-HERO13-Black`), which matches the
  source rule of 3.1, to confirm or change.
- **The settings screen**: a section *Importeren* with the folders and sources.
- **What the app finds**, as a list for the import view (S04): each subfolder of a drop folder
  (not `_imported` or hidden ones) with its source name (the folder's name, or the name the user
  set for it), each synced folder, each card; with the layout and, for a card, the camera.
- **Tests**: the layout from the index (split, day, flat, majority), the name proposal, the source
  name rule, the discovery of drop folders and cards in temporary folders, reading and writing the
  settings.

## How

- `sources.rs`: `Layout { split, day }`, `layouts(db)` over `items` (the parts of the path
  between the source and the file name), `propose_for_camera(db, camera)` (the source with most
  items of that camera among sources with day folders, as a card's files are), `name_from_model`,
  `source_name_problem`, `discover(settings, mounts)`.
- `config.rs`: `Settings.import` with the drop folders, the synced folders (a path, the source
  for photos and the one for videos) and per source `name` (another name in the library),
  `exclude`, `ignore`, `delete_imported`.
- Commands `import_sources` (what is found) and the settings through `save_settings`.
- `Settings.tsx`: a section *Importeren*: drop folders and synced folders added with the folder
  dialog and removed; per source found its name in the library, *negeren*, *na import uit
  `_imported` verwijderen*, exclusions.

## Files affected

- `src-tauri/src/sources.rs` (new), `src-tauri/src/config.rs`, `src/components/Settings…`,
  `docs/data-contract.md`.

## Progress log

### [2026-10-30] Written

### [2026-10-31] Built

- `sources.rs`: `Layout`, `layouts` (from the paths of each source's photos and videos in
  ordinary month folders), `source_name_problem`, `name_from_model`, `propose` (for a card's
  camera: the source with most of its files among sources with day folders, else a name from the
  model), `discover` (drop folders' subfolders without `_imported` and hidden ones, synced folders,
  folders under `/Volumes` with `DCIM`, the camera from the first of at most 20 files on a card).
- `db.rs`: `source_paths`, `sources_of_camera`. `config.rs`: `Settings.import` (`Import`,
  `Synced`, `SourceSettings`). Command `import_sources`.
- `ImportSettings.tsx` in the settings: drop and synced folders added with the folder dialog and
  removed; per subfolder found its name in the library, *negeren*, *_imported leegmaken*,
  exclusions; the cards and synced folders with where they go. `sources.ts` for the texts.
- Data contract 3.1: a new source gets day folders, an existing one keeps the layout read from
  the library; no source names of this library in the contract's rule.
- Tests: `cargo test` 171 (layouts with a majority, names, proposals, discovery in temporary
  folders with a card built from a JPEG), `pnpm test` 128; clippy, tsc and the build clean.
- On Peter's index (read only, `examples/sources-check`): `EOSR` split with day folders, the phone
  streams flat; a GoPro HERO13 card proposes `Hero13`, a Canon EOS R `EOSR`, a camera the library
  doesn't know a name from its model.
