# Story S02: Stars and hearts on the tiles and in the lightbox

**Feature:** `F013-ratings-and-hearts`
**Status:** Planned
**Depends on:** S01

## Goal

Every tile shows its stars bottom right and its heart top right, scaled with the tile; the
lightbox shows them too.

## Acceptance criteria

- **Stars**: only the filled ones ("★★★" for 3), small, on a dark translucent strip, bottom
  right; nothing for an unrated photo.
- **Heart**: top right; red when this Mac's owner put one on it, white when only others did; a
  small number from 2 hearts on; the tooltip names who.
- **The "NAS" badge** moves to the top left.
- **Scaling**: the marks' size follows the tile size, readable at the smallest tile and not
  overpowering at the largest.
- **The lightbox** shows the stars and the hearts with who.
- **In the library and in a list** alike; the other Mac's changes appear within 15 seconds.
- **Tests** (`pnpm test`): the texts (who hearted it), the scale per tile size.

## Files affected

- `src/components/Grid.tsx`, `src/components/Lightbox.tsx`, `src/lib/marks.ts`, `src/App.tsx`.

## Progress log

### [2026-10-30] Written
