# Story S02: The Live Photo symbol on the tile and in the lightbox

**Feature:** `F017-live-photos`
**Status:** Done
**Depends on:** S01

## Goal

A Live Photo video shows three concentric circles instead of the camera, on the tile and in the
lightbox, with the photo it belongs to as the tooltip.

## Acceptance criteria

- **The symbol**: three concentric circles (a dotted outer ring, a ring and a dot, as Apple's),
  white with a soft shadow like the camera, the same size and place (top left).
- **The tooltip**: "Live Photo bij IMG_1234.HEIC".
- **The lightbox**: the symbol with "Live Photo" in the bar.
- **Tests** (`pnpm test`): the tooltip text.

## Files affected

- `src/components/LiveIcon.tsx` (new), `src/components/Grid.tsx`, `src/components/Lightbox.tsx`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `LiveIcon.tsx`, `liveTitle(item)`; the tile and the lightbox's bar show it instead of the
  camera for a video with a `live` photo. One vitest test; 104 tests, `tsc` and `pnpm build`
  clean. The icon checked as an image: it reads as Apple's Live symbol.

### [2026-10-30] Checked

- Peter in the app: "dat werkt".
