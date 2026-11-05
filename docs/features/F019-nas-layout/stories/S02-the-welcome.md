# Story S02: The welcome: the home share, the file, a new library

**Feature:** `F019-nas-layout`
**Status:** Built: to try with an empty share
**Depends on:** S01

## Goal

A new user names their NAS and one shared folder; the app finds the library described there, or
sets up a new one in it.

## Acceptance criteria

- Step 1 of the welcome (F006/S06) asks the NAS and the home share (one field instead of four
  share names; the four names go).
- After connecting: `familylightbox.json` found → the shares it names are connected too, and the
  welcome goes on to step 2.
- Not found → *Hier een nieuwe bibliotheek inrichten*: the subfolders `Library`, `Previews`,
  `Lists`, `Exports` and the file in that share, with a name for the library; nothing is
  overwritten that exists. Or *Bestaande mappen aanwijzen*, for a library that is elsewhere: per
  location a share and a path, written to the file.
- The home share's file must be writable to set up; otherwise the welcome says so.
- **Tests**: setting up in an empty share, in a share that has some of the folders, refused when
  the file exists; the texts.

## Files affected

- `src-tauri/src/layout.rs`, `src-tauri/src/lib.rs`, `src/components/DeviceSetup.tsx`.

## Progress log

### [2026-10-31] Written

### [2026-11-05] Built

- Step 1 asks the NAS and *Gedeelde map van de bibliotheek* (saved as `home_share`); the four
  share names are gone from the welcome. Connecting mounts the home share first, as before.
- `layout_home` says what is there: connected, a valid file (and its name), a broken one, and
  which of `Library`, `Previews`, `Lists`, `Exports` exist. A file found: on to step 2.
- None: a step *library* with a name for it and *Hier een nieuwe bibliotheek inrichten*; the text
  says which folders are made and which are used as they are (`setUpText`). Or *bestaande mappen
  aanwijzen*: per location a share and a folder, prefilled with the old four shares.
- `layout_set_up` (`layout::set_up`): the folders the locations in the home share name, then the
  file with `create_new`, so an existing one is never overwritten; checked like a read file;
  no write access is said as such. The copy is read at once and the welcome connects again, so
  the shares the file names are mounted before step 2.
- Tests: `cargo test` 223 (2 new: an empty share, a share with some folders, pointing elsewhere,
  refused when the file is there, a bad path writes nothing); `pnpm test` 168 (the set-up text,
  paths inside a share).
