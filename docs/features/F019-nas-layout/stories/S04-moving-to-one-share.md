# Story S04: Moving an existing library to one share

**Feature:** `F019-nas-layout`
**Status:** Done (2026-11-02); the old shares go later
**Depends on:** S03; LibraryLocal's feature for the NAS's scripts, the Drive mirror and the
snapshots

## Goal

A library spread over several shares is moved into one, on the NAS, and the Macs follow by the
file alone.

## Acceptance criteria

- The steps, in `docs/moving-to-one-share.md`: a new share with the four folders; the contents
  moved on the NAS (on one volume a move is a rename, nothing is copied); the rights of the
  folders that were per share now per subfolder, and the rights of folders below them; the file in the
  new share's root; on each Mac the home share set to the new share (the only per-Mac change);
  the old shares kept read-only for a while, then removed.
- Before the move the app can say what it would find after it: a dry check that reads the new
  share's file and resolves the roots without using them.
- After the move: the sync finds nothing new, the index is the same, the lists and marks are the
  same on both Macs, an import writes to the new place.
- What outside the app reads the shares (the CLI, the NAS's scripts, the Drive copy) is moved in
  LibraryLocal's story, at the same time.

- **Older apps**: an app before 0.5 doesn't read the file and would go on writing to the old
  `Lists`. So every device keeps its app version in `devices.json` (updated on every start), and
  the check names every device that still has to update.
- **The home share moves by the file**: the file says `home`; a Mac whose home share is another
  one moves its home there once that share is mounted with a valid file. So the old home's file
  can point the Macs on, and nothing is set on any Mac.

## Files affected

- `docs/moving-to-one-share.md` (new), `src-tauri/src/layout.rs` (`home`, `next_home`,
  `dry_check`), `src-tauri/src/nas.rs` (`app` per device), `src-tauri/src/config.rs`
  (`set_home_share`), `src-tauri/src/lib.rs`, `src/components/MoveCheck.tsx` (new).

## Progress log

### [2026-10-31] Written

### [2026-10-31] Built in the app

- `nas.rs`: `Entry.app` and `Known.app`; `merge_device` brings a device's version up to date
  (and writes `devices.json` only then).
- `layout.rs`: `NasLayout.home`; `next_home` (the copy's `home`, when that share is mounted with
  a valid file); `dry_check` (per location the root and what is in it: the library's year
  folders, `devices.json` in the lists; and each device's version against 0.5.0). `check_nas`
  moves the home share when `next_home` says so (`config::set_home_share`).
- `MoveCheck.tsx` in the settings: *een verhuis naar een andere gedeelde map nakijken*.
- `docs/moving-to-one-share.md`: before (every device on 0.5 or later, a snapshot, nobody
  importing), on the NAS (the share, the move, old shares read-only, the file with `home` in the
  new share and the old home), each device (opening the app is enough), after (the old shares
  go).
- Tests: `cargo test` 186 (the version kept current, following the home, the check before a
  move), `pnpm test` 133; clippy, tsc, build.
- The NAS's side (the CLI, the scripts, Drive, snapshots, rights) is LibraryLocal F012.

### [2026-11-02] Moved

This library moved to one share by the steps of `docs/moving-to-one-share.md`, with two changes
learnt on the way: on a NAS where each share is its own file-system volume, moving between shares
is a copy anyway, so the contents were copied and the old shares left as they were until they are
removed; and a copy that sets file modes can drop the files' inherited rights, which then have
to be applied again from the top. Then the file with `home` in both places. Both Macs (0.5.0)
followed by opening the app: the new share became their home share, the layout came from the NAS,
nothing was written to the old places since. The details of this NAS are in LibraryLocal.
