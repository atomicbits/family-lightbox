# Story S04: Stars and hearts to Lightroom

**Feature:** `F023-lightroom-plugin`
**Status:** Done (2026-11-05)
**Depends on:** S01, S02

## Goal

The stars and hearts given in FamilyLightbox show in Lightroom as its stars, so Peter sees there
what was chosen together.

## Acceptance criteria

- **Which photos**: every photo with stars or a heart in FamilyLightbox (not only those in lists)
  whose original is in the working copy and in the catalog (added to it by S02 for the lists'
  photos; others aren't added for this).
- **The rating**: a heart, anyone's, is 5 stars (Peter, 2026-11-05: simpler than a keyword per
  person); otherwise the photo's stars in FamilyLightbox.
- **Who wins**: FamilyLightbox, but only where it has stars or a heart; a photo without either
  keeps its stars in Lightroom. One way: stars given in Lightroom don't go back.
- **Hidden photos** are not passed on.
- *Lijsten bijwerken* does it too, and its message says how many ratings changed.
- **Tests**: `cargo test` for the file's ratings (stars, a heart, both, a photo not here, a hash
  the index doesn't know); by hand in Lightroom.

## Files affected

- `src-tauri/src/lightroom.rs`, `src-tauri/src/lib.rs`, `lightroom/FamilyLightbox.lrplugin/UpdateLists.lua`.

## Progress log

### [2026-11-05] Written

With Peter's choices: every photo with stars, FamilyLightbox wins where it has stars, a heart is
5 stars, hidden photos not passed on.

### [2026-11-05] Built

- `lightroom.rs` `ratings`: every hash with stars or a heart, by its path from the index, if the
  original is here; a heart (anyone's) is 5, else the stars; 0 or no path: left out. The file has
  them as `ratings`, written again after every change of stars or hearts too.
- `UpdateLists.lua`: outside a write access the photos whose Lightroom rating differs, then in the
  second write access `setRawMetadata("rating", …)`; the message counts them.
- On Peter's Mac: 15 ratings in the file (5 × 1, 3 × 3, 6 × 4, 1 × 5).
- `cargo test` 215 passed (1 new), `cargo clippy --all-targets` clean.

### [2026-11-05] Checked

Peter: the stars work in Lightroom; the integration is good as it is.
