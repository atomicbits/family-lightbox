# Story S04: Moving an existing library to one share

**Feature:** `F019-nas-layout`
**Status:** Planned
**Depends on:** S03; LibraryLocal's feature for the NAS's scripts, the Drive mirror and the
snapshots

## Goal

A library spread over several shares is moved into one, on the NAS, and the Macs follow by the
file alone.

## Acceptance criteria

- The steps, in `docs/moving-to-one-share.md`: a new share with the four folders; the contents
  moved on the NAS (on one volume a move is a rename, nothing is copied); the rights of the
  folders that were per share now per subfolder, including denied year folders; the file in the
  new share's root; on each Mac the home share set to the new share (the only per-Mac change);
  the old shares kept read-only for a while, then removed.
- Before the move the app can say what it would find after it: a dry check that reads the new
  share's file and resolves the roots without using them.
- After the move: the sync finds nothing new, the index is the same, the lists and marks are the
  same on both Macs, an import writes to the new place.
- What outside the app reads the shares (the CLI, the NAS's scripts, the Drive copy) is moved in
  LibraryLocal's story, at the same time.

## Files affected

- `docs/moving-to-one-share.md` (new), `src-tauri/src/lib.rs` (the dry check).

## Progress log

### [2026-10-31] Written
