# Feature 25: Lightroom's edits visible in FamilyLightbox

**Status:** In design (idea accepted 2026-11-04)
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

## Open points

1. Every edited photo (Lightroom can find those by their develop history), or only those of lists?
2. Renditions at preview size (2 048 px) only, or also at export size, so feature 24 can make
   edited prints without Lightroom open?

## Stories

To be written when the open points are decided.
