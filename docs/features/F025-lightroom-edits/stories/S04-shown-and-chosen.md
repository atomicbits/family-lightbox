# Story S04: Shown and chosen

**Feature:** `F025-lightroom-edits`
**Status:** Built: to try with renditions
**Depends on:** S03

## Acceptance criteria

- A photo with renditions shows its chosen variant in the grid, the lists and the Lightbox, with a
  small mark *bewerkt in Lightroom*. Unfolding the variants into tiles like a stack is S06.
- In the Lightbox: the versions side by side to look at, and *Deze versie tonen* chooses the variant; a `variant`
  event in `Lists/marks/`, so the other Mac shows the same.
- **Tests**: `cargo test` for the chosen variant (mark, default the latest); `pnpm test` for the
  texts and folding.

## Progress log

### [2026-11-05] Written

### [2026-11-05] Built

- Split: the variants share their photo's hash, and the grid's stacks work by hash, so unfolding
  them into tiles is its own piece of work (S06), best judged with real renditions.
- `Item.variants`: the small renditions next to the photo's preview on this Mac, latest edit
  first (`renditions::fill`, one look per folder), in `items`, `items_filtered`, `list_items`
  and `items_by_hash`; not in the index.
- The mark `variant` (data contract 5.5) in `marks.rs` (read, merged, written; an unknown variant
  ignored) and `marks_variant`; in the window `MarkChange` `variant`, shown at once.
- `variants.ts`: `chosenVariant` (the merge rule), `variantLabel` (*Origineel*, *Bewerkt*, a
  copy's name from its slug). The grid's tile shows the chosen version with *LR* top left (its
  tooltip names the version and how many there are); the Lightbox has a button per version to
  look at and *Deze versie tonen* to choose it, for both Macs.
- New renditions (`renditions-changed`) reload the items as after an index build.
- `cargo test` 229, `pnpm test` 170 (2 new), `tsc`, `vite build`, clippy clean.
