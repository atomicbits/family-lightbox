# Story S05: Exported as edited

**Feature:** `F025-lightroom-edits`
**Status:** Built: to try with renditions
**Depends on:** S04

## Acceptance criteria

- An export (feature 24) of a photo with a chosen rendition renders from its `.lr.full.jpg` on the
  NAS instead of the original; the plain original when that is chosen. `export.json` says which.
- **Tests**: `cargo test` for the source an export takes per variant.

## Progress log

### [2026-11-05] Written

### [2026-11-05] Built

- `Part.variant` (kept in the job, so a resumed export takes the same): the chosen version of each
  rendered photo (`renditions::chosen`, the marks and the renditions here); the `originelen` preset
  and videos never take one.
- `export::run` renders from `<src>.<variant>.full.jpg` in the NAS's `Previews`; missing, the
  original with a problem *de bewerkte versie staat niet op de NAS, het origineel genomen*.
- `export.json` lists those files in `lightroom_edits` (data contract 6, 1.7).
- `cargo test` 229 passed (the export test extended), clippy clean.
