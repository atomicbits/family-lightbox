# Story S04: Shown and chosen

**Feature:** `F025-lightroom-edits`
**Status:** Planned
**Depends on:** S03

## Acceptance criteria

- A photo with renditions shows its chosen variant in the grid, the lists and the Lightbox, with a
  small mark *bewerkt in Lightroom*; folded like a stack, a click unfolds the variants (the plain
  original and each rendition).
- In the Lightbox and the right-click menu: *Deze versie tonen* chooses the variant; a `variant`
  event in `Lists/marks/`, so the other Mac shows the same.
- **Tests**: `cargo test` for the chosen variant (mark, default the latest); `pnpm test` for the
  texts and folding.

## Progress log

### [2026-11-05] Written
