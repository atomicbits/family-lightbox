# Story S01: The layout file, the four roots, and everything using them

**Feature:** `F019-nas-layout`
**Status:** In progress
**Depends on:** —

## Goal

The app knows where the library, the previews, the lists and the exports are from
`familylightbox.json` on the NAS, and everything that reads or writes the NAS uses those four
roots instead of four share names.

## Acceptance criteria

- **The file** (data contract 2, "Locations", 1.4) in the root of the home share:
  `contract_version`, `name`, and `library`, `previews`, `lists`, `exports`, each
  `{ "share": …, "path": … }`; `path` relative to the share's root, `""` for the root itself, with
  `/` and without `..`. Unknown fields are allowed.
- **The home share** is a setting per Mac (`home_share`); while it is empty, `Lists` (decision 4).
- **Read** when the NAS is checked (at start, on connecting, coming back to the window) and kept
  in the config folder (`familylightbox-layout.json`), so the app knows the layout while the NAS
  is away. Missing on the NAS: the shares named in the Mac's settings apply, as today
  (decision 6). Broken: reported, and the copy read last is used.
- **The roots**: each location's share is found among the mounts as today (F006/S02) and its
  root is that mount plus the path. Everything that took a share's mount point as its root (the
  sync, the index, the previews, the lists, the marks, the import, the access check) gets the
  location's root instead.
- **Connecting** mounts each share the layout names once: four locations in one share are one
  mount and one sign-in.
- **Tests** (`cargo test`): reading the file (one share, four shares, paths, unknown fields,
  `..` refused, broken, missing), the roots against mounts, connecting a share once for several
  locations, the copy kept and used without the NAS.

## How

- `layout.rs`: `Location`, `NasLayout`, `parse`, `from_shares` (the defaults from the Mac's
  settings), `refresh(config, mounts, settings)` (read from the home share's mount, keep the
  copy), `current(config, settings)` (the copy, else the settings).
- `mounts.rs`: `find_shares` takes the layout: each location's mounts by its share, with the
  path added to the mount point; `NasMounts` stays as it is for the callers.
- `connect.rs`: what to connect is the distinct shares of the locations that have no mount.
- `config.rs`: `Settings.home_share`.

## Files affected

- `src-tauri/src/layout.rs` (new), `src-tauri/src/mounts.rs`, `src-tauri/src/connect.rs`,
  `src-tauri/src/config.rs`, `src-tauri/src/lib.rs`, `docs/data-contract.md`.

## Progress log

### [2026-10-31] Written
