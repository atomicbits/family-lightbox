# Story S01: The layout file, the four roots, and everything using them

**Feature:** `F019-nas-layout`
**Status:** Built (2026-10-31); checked with a real file in S03
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

### [2026-10-31] Built

- `layout.rs`: `Location`, `NasLayout` (`from_shares`, `in_one_share`, `locations`), `parse` (a
  share and a path inside it, no `..`, no absolute path, no `\`), `read_copy`, `refresh` (from the
  home share's mount; kept as `familylightbox-layout.json`; the copy removed when the file is gone
  from the NAS, kept when it is broken), `Source` (`nas`, `copy`, `settings`).
- `config.rs`: `Settings.home_share`; `Settings.layout`, read from the copy by `load_settings`
  and never saved; `effective_layout` (the file, else the share names) and `home`.
- `mounts.rs`: `find_shares` takes the layout: each location's mounts by its share, the mount
  point plus the location's path. All ten callers are unchanged and get the roots.
- `connect.rs`: `to_connect` gives each share once, the lists' first. `lib.rs`: `connect_nas`
  connects the home share first, reads the file, then connects what it names; `check_nas` reads
  the file first and reports `layout_source` and `layout_problem`. The sync's messages name the
  location's share.
- Data contract section 2: the locations and the file, version 1.4.
- Tests: `cargo test` 182 (reading the file, refusals, broken; the copy kept, used without the
  NAS, removed when the file goes; the roots of one share and of another share's subfolder; one
  share connected once for four locations), `pnpm test` 132; clippy and tsc clean.
- On Peter's Mac: no file on the NAS yet, so the share names of the settings apply; the dev app
  checks the NAS as before.
