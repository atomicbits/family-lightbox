# Story S03: Sources and their settings; the name proposed from the library

**Feature:** `F018-import`
**Status:** Planned
**Depends on:** —

## Goal

The app knows where new files come from on this Mac: the subfolders of a drop folder, synced
folders, and cards or devices with `DCIM/`; each with a source name and its few settings.

## Acceptance criteria

- **Settings**, kept on this Mac (`~/.config/media/`, data contract 7): drop folders, synced
  folders with the source they feed (one for photos and one for videos for the historical iPhone
  sources), per source: exclusions, "ignore", and "delete from `_imported/` once verified on the
  NAS" (off by default).
- **The layout rule** in the data contract 3.1: new sources always `<source>/<day>/`; `EOSR`'s
  split and the iPhone sources without a day folder are historical rules for those names.
- **A card**: a mounted volume with `DCIM/` is seen when it appears; the app reads the camera of
  a few files (S01) and proposes the source under which the index has most photos of that camera;
  otherwise a name from the model (`GoPro HERO13 Black` → `GoPro-HERO13-Black`), which matches the
  source rule of 3.1, to confirm or change.
- **The settings screen**: a section *Importeren* with the folders and sources.
- **Tests**: the name proposal, the source name rule, reading and writing the settings.

## Files affected

- `src-tauri/src/sources.rs` (new), `src-tauri/src/config.rs`, `src/components/Settings…`,
  `docs/data-contract.md`.

## Progress log

### [2026-10-30] Written
