# Story S02: Stars and hearts on the tiles and in the lightbox

**Feature:** `F013-ratings-and-hearts`
**Status:** Done
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

## How

- `marks.ts`: `markSize(tile)` (7.5 % of the tile, between 10 and 18 px; tiles go from 96 to
  320 px), `starsText`, `heartView` (whose hearts, whether this Mac's owner's is among them),
  `marksLines` for the tooltip.
- `Grid.tsx`: `TileMarks` draws the heart in a dark round badge top right and the stars in amber
  on a dark strip bottom right; the tooltip lists the file name, the rating and who hearted it
  (and in a list who added it).
- The owner is the device's `owner` (`device.json`); App passes `marks.state` and it to the grid
  and the lightbox.

## Files affected

- `src/components/Grid.tsx`, `src/components/Lightbox.tsx`, `src/lib/marks.ts`, `src/App.tsx`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `pnpm test`: the size at the smallest, a middle and the largest tile, the stars text, whose
  hearts, the tooltip lines; 69 tests. `tsc` and `pnpm build` clean.
- Not seen in the app yet: there are no stars or hearts on the share until S03 sets them.

### [2026-10-30] Checked

- Peter in the app: "F013 werkt".
