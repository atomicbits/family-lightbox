# Story S03: Taken in by the app

**Feature:** `F025-lightroom-edits`
**Status:** Planned
**Depends on:** S02

## Acceptance criteria

- The app picks up `lightroom-out/` (at start, after the sync, and when the folder changes): each
  rendition checked (a JPEG, its photo in the index), written to the NAS's `Previews` at its name
  through a temporary name and a rename, the small one also to this Mac's previews; then removed
  from the folder. Without the NAS it waits.
- The sync copies the small renditions to the other Macs like previews; the index records per
  photo its variants and their time.
- **Tests**: `cargo test` for naming (slug of a copy), checking, writing, waiting without the NAS.

## Progress log

### [2026-11-05] Written
