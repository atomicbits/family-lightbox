# Feature 25: Lightroom's edits visible in FamilyLightbox

**Status:** Planned (an idea, to discuss)
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

## Open points

1. Instead of the preview, or next to it?
2. Only the photos of lists, or every edited photo (Lightroom can find those by their develop
   history)?
3. Renditions at preview size (2 048 px) only, or also at export size, so feature 24 can make
   edited prints without Lightroom open?

## Stories

To be written when the idea is accepted.
