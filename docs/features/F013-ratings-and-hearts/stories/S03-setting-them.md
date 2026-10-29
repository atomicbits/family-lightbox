# Story S03: Setting stars and hearts for the selection

**Feature:** `F013-ratings-and-hearts`
**Status:** In progress: built and tested; to try in the app
**Depends on:** S02

## Goal

With one or more photos selected, the header shows five stars and a heart, saying how many
photos they apply to; a click sets them for all. Keys and the lightbox do the same.

## Acceptance criteria

- **The header**, when at least one photo is selected: "12 foto's" for several (nothing for one,
  on Peter's request: the file name was too busy there), five stars and a heart.
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

## How

- `marks.ts`: `sharedStars`, `heartState`, `starsAfterClick`, `heartAfterClick`, `marksBlocked`
  (no NAS, a problem, or no owner) and `changeText` for the message.
- `MarksBar.tsx`: the label, a dimmed "–" when the ratings differ, five star buttons and the heart
  (♡ none, ♥ all, a lighter ♥ some); in the header after the month, and in the lightbox's bar.
- `App.tsx`: the selection is the picked tiles or the selected one; a change gathers their items
  (fetching months not loaded yet), writes one `marks_rate` or `marks_heart` for the unique
  hashes, and says what happened in the header (S04 turns that into the message with undo).
  The keys `0`–`5` and `H` work in the grid and the lightbox, without ⌘, Ctrl or ⌥.

## Files affected

- `src/App.tsx`, `src/components/MarksBar.tsx` (new), `src/components/Lightbox.tsx`,
  `src/lib/marks.ts`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `pnpm test`: the shared rating (same, different, none, partly none), the heart state, what a
  click does, when it is blocked, the texts; 73 tests. `tsc` and `pnpm build` clean.
- To try in the app, which also shows S02 for the first time.
