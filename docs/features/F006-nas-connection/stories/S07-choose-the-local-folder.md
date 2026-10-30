# Story S07: Choosing the local folder at first start

**Feature:** `F006-nas-connection`
**Status:** Built (2026-10-30); to check on Lynn's Mac
**Depends on:** S06

## Goal

Where the app keeps its local copies on this Mac (the working copy of `Library`, the previews,
the copy of `Lists`) is the user's choice, with a proposal (Peter, 2026-10-30). Until now they
were fixed at `~/Pictures/LibraryLocal`, `~/Pictures/Previews` and `~/Pictures/Lists`, the CLI's
places, which mean nothing to another family and may be on a disk without room.

## Acceptance criteria

- **One folder** is chosen; the app keeps `Library`, `Previews` and `Lists` in it. The proposal for
  a new Mac is `~/Pictures/FamilyLightbox`.
- **In the welcome** (S06), step 2 shows the folder with *Kies…* (the macOS folder dialog) and the
  free space on its disk. *Bewaar* checks that the folder can be created and written, saves it in
  the app's settings, creates the device, and restarts the app once so everything uses the new
  folder. Nothing is written in the old places.
- **Kept in the settings** (`familylightbox.json`, `local`: the three paths). The CLI's
  environment variables (`MEDIA_LIBRARY`, `MEDIA_PREVIEWS`, `MEDIA_LISTS`) still win.
- **An existing installation keeps its folders**: with no `local` in the settings and the CLI's
  working copy `~/Pictures/LibraryLocal` present, the three old places are written to the settings
  once, as they are. Adopting the NAS host (S06) and the folders both work when the other wrote
  the settings file first.
- **The settings panel** shows the three folders. Moving them later, with what is in them, is not
  part of this story.
- Data contract section 7: the local places are the app's settings, with the CLI's as defaults.
- **Tests**: `cargo test` for adopting the folders, the three paths from a chosen folder, a folder
  that can't be written, the free space; `pnpm test` for the free-space text.

## Files affected

- `src-tauri/src/config.rs`, `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml` and
  `capabilities/default.json` (`tauri-plugin-dialog`), `package.json`,
  `src/components/DeviceSetup.tsx`, `src/components/Settings.tsx`, `src/lib/api.ts`,
  `docs/data-contract.md`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `config.rs`: `Settings.local` (`Local`: library, previews, lists), `Local::inside(folder)`,
  `proposed_folder`, `cli_local`, `adopt_local` (the CLI's places when its working copy exists),
  `choose_local` (creates the folder, writes a test file, keeps the three paths). Writing the
  settings without the NAS check is `write_settings`, for what is adopted or chosen before a NAS
  is set; `adopt_host` now looks at an empty host instead of a missing file, so the two adoptions
  work in either order.
- `lib.rs`: `paths()` reads the local folders from the settings (adopting first), else the
  proposal; the `MEDIA_*` variables still win. Commands `local_folders`, `free_space`
  (`statvfs` on the nearest existing folder), `choose_local`, `relaunch`. The dialog plugin with
  only `dialog:allow-open`.
- `DeviceSetup.tsx`: the folder with *Kies…* and the free space in step 2; *Bewaar* keeps the
  folder, creates the device and restarts the app. A Mac that has a device but no folder yet
  (one set up with 0.4.0) gets only that choice. `App.tsx` starts the NAS, the sync and the rest
  only once both are there. The welcome saves the whole settings object, so the folder stays.
- `Settings.tsx`: a section with the three local folders.
- Tests: `cargo test` 168, `pnpm test` 120, tsc, clippy and `pnpm build` clean.
- On Peter's Mac the restarted dev app wrote `raven` and the CLI's three places to its settings,
  as they were; nothing moved.
