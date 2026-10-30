# Story S02: The Live Photo symbol on the tile and in the lightbox

**Feature:** `F017-live-photos`
**Status:** Planned
**Depends on:** S01

## Goal

A Live Photo video shows three concentric circles instead of the camera, on the tile and in the
lightbox, with the photo it belongs to as the tooltip.

## Acceptance criteria

- **The symbol**: three concentric circles, white with a soft shadow like the camera, the same
  size and place (top left).
- **The tooltip**: "Live Photo bij IMG_1234.HEIC".
- **The lightbox**: the symbol with "Live Photo" in the bar.
- **Tests** (`pnpm test`): the tooltip text.

## Files affected

- `src/components/LiveIcon.tsx` (new), `src/components/Grid.tsx`, `src/components/Lightbox.tsx`.

## Progress log

### [2026-10-30] Written
