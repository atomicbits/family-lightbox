# Story S02: The welcome: the home share, the file, a new library

**Feature:** `F019-nas-layout`
**Status:** Planned
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
