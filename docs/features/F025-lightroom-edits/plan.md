# Feature 25: Lightroom's edits visible in FamilyLightbox

**Status:** In progress (stories written 2026-11-05)
**Depends on:** Feature 23 (the plug-in)
**Folder:** `F025-lightroom-edits`

## Goal

A photo Peter edited in Lightroom Classic (cropped, straightened, developed) shows in the app as
he edited it, next to or instead of the plain original's preview, and a virtual copy in Lightroom
shows as a variant of its photo; so a list chosen in the app can be judged on the edited versions.

## Idea

- Only Lightroom can render its edits. The plug-in of feature 23 gets a second part: a
  **publish service** *FamilyLightbox*. Lightroom publishes the photos put in it (or every photo
  in a list's collection) as JPEGs at the size of a preview, and publishes them again by itself
  when they are edited afterwards (Lightroom's *modified photos to re-publish*).
- The renditions go next to the previews, as derived files: `Previews/<original path>.lr.jpg`, and
  a virtual copy `…​.lr-<copy name>.jpg` (a new rule in data contract 4), and to the NAS with the
  rest of `Previews`.
- The app shows the edited rendition when there is one, with a mark *bewerkt in Lightroom*, and
  the plain preview a click away; virtual copies as the photo's variants, like the members of a
  stack (feature 15). Exports (feature 24) can use the rendition at its size where it is large
  enough, or ask Lightroom for one.

## Decisions

Taken with Peter on 2026-11-04:

1. **Shown in the grid and in lists**: an edited photo shows its edited rendition, and its
   variants (the plain original, virtual copies) are shown like the members of a stack
   (feature 15): folded to one, unfolded with a click.
2. **One variant is chosen**: per photo, the variant that shows when folded and that an export
   (feature 24) takes, chosen by hand (shared, like a stack's choice); by default the latest edited
   one. The choice is a mark of the photo (data contract 5.5), kept for both Macs.

Taken with Peter on 2026-11-05:

3. **Only the photos of lists**: the edited photos and virtual copies in the collections of
   FamilyLightbox's lists (feature 23), not every edited photo in the catalog.
4. **Two sizes**: a rendition at preview size (2 048 px) to show, and one at full size for
   exports (feature 24), so an edited print needs no Lightroom open.

Proposed with the stories (2026-11-05):

5. **No publish service**: *Lijsten bijwerken* (feature 23) renders, after updating the lists, the
   edited photos of the lists whose edit is newer than their rendition. One click as now; no
   second place in Lightroom to look after.
6. **Through a folder on the Mac**: the plug-in renders into `~/.config/media/lightroom-out/` with
   a small JSON per rendition; the app puts them on the NAS (temporary name, rename) as it does
   previews. Writing the contract's files stays in the app.
7. **The full size stays on the NAS**: `Previews/<path>.lr.full.jpg` isn't copied to the Macs'
   previews; an export reads it from the NAS, as it reads an original that isn't here.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-the-contract.md) | The contract (1.7): renditions next to the previews, the chosen variant as a mark | Done | — |
| [S02](stories/S02-rendering-in-lightroom.md) | The plug-in renders the edited photos and virtual copies of the lists, at two sizes, only what changed | Planned | S01 |
| [S03](stories/S03-taken-in.md) | The app takes them in: to the NAS's `Previews`, the small one to the Mac, known to the index | Planned | S02 |
| [S04](stories/S04-shown-and-chosen.md) | Shown in the grid and lists, variants folded like a stack, one chosen and kept for both Macs | Planned | S03 |
| [S05](stories/S05-exported.md) | An export takes the chosen variant at full size | Planned | S04 |
