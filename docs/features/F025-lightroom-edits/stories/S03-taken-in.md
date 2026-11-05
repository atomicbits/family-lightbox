# Story S03: Taken in by the app

**Feature:** `F025-lightroom-edits`
**Status:** Built: to try with S02
**Depends on:** S02

## Acceptance criteria

- The app picks up `lightroom-out/` (at start, after the sync, and when the folder changes): each
  rendition checked (a JPEG, its photo in the index), written to the NAS's `Previews` at its name
  through a temporary name and a rename, the small one also to this Mac's previews; then removed
  from the folder. Without the NAS it waits.
- The sync copies the small renditions to the other Macs like previews; the index records per
  photo its variants and their time.
- A rename of a source (F022) moves the renditions with the previews.
- **Tests**: `cargo test` for naming (slug of a copy), checking, writing, waiting without the NAS.

## Progress log

### [2026-11-05] Written

### [2026-11-05] Built

- `renditions.rs`: `take_in` reads the JSONs in name order; a variant `lr` or `lr-<slug>`, the
  path inside the working copy under a year folder, both files JPEGs; then the full one and the
  small one to the NAS's previews root (from the layout), the small one to this Mac's, each
  through `<name>.<pid>.tmp` and a rename, with the edit's time; `removed` deletes them. A JSON
  that can't be taken is reported (*Lightroom* in the messages) and removed with its files, so
  it doesn't come back. Without the NAS's previews nothing is touched.
- Run at start and every 30 s when the folder has a JSON, and after each sync; the window hears
  `renditions-changed`.
- `rename::move_previews` moves a photo's renditions too (`renditions::suffixes`).
- The index doesn't record renditions: S04 finds them next to the previews when it shows a month.
- `cargo test` 227 passed (3 new for the take-in, the rename test extended), clippy clean.
