# Story S03: Setting stars and hearts for the selection

**Feature:** `F013-ratings-and-hearts`
**Status:** Planned
**Depends on:** S02

## Goal

With one or more photos selected, the header shows five stars and a heart, saying how many
photos they apply to; a click sets them for all. Keys and the lightbox do the same.

## Acceptance criteria

- **The header**, when at least one photo is selected: "12 foto's" (or the file name for one),
  five stars and a heart.
  - The stars show the rating all selected photos share, or a dimmed "–" when they differ;
    clicking a star gives all of them that rating; clicking the rating they all have removes it.
  - The heart is filled when this person's heart is on all of them, half when on some; a click
    puts it on all, or takes it off all when it was on all.
- **Keys**, in the grid and the lightbox: `1`–`5` set the stars, `0` removes them, `H` toggles
  the heart.
- **The lightbox**: the same stars and heart for the photo shown.
- **Without the NAS**: disabled, with the reason as a tooltip.
- **Tests** (`pnpm test`): the shared rating and the heart state for a selection, what a click
  does.

## Files affected

- `src/App.tsx`, `src/components/MarksBar.tsx` (new), `src/components/Lightbox.tsx`,
  `src/lib/marks.ts`.

## Progress log

### [2026-10-30] Written
