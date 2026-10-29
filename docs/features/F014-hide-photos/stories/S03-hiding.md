# Story S03: Hiding and unhiding from the right-click menu, with undo

**Feature:** `F014-hide-photos`
**Status:** In progress: built and tested; to try in the app
**Depends on:** S02

## Goal

*Verbergen* in the right-click menu hides one photo or the selection; *Niet meer verbergen* on a
hidden photo that is shown undoes it. Both get the message with undo, like stars and hearts.

## Acceptance criteria

- **The menu**: *Verbergen* on photos that aren't hidden; *Niet meer verbergen* on shown hidden
  ones (from S04, or in a list); with a selection of both, both items, each for its own photos.
- **The message**: "20 foto's verborgen", "1 foto niet meer verborgen", with *Ongedaan maken*;
  ↶ and ⌘Z work the same way, step by step with stars, hearts and lists.
- **Undo** puts back what each photo was, leaving alone photos the other Mac changed since.
- **After hiding**, the picks are cleared and the selection moves to the next shown photo.
- **Without the NAS**: the items are disabled with the reason.
- **Tests** (`pnpm test`): the menu items for a selection, the texts, the undo step.

## How

- The menu counts how many of the clicked photos are hidden: *Verbergen* for the others,
  *Niet meer verbergen* for those, with the number when the selection has both.
- `applyHide` writes one `marks_hide` for the photos whose state changes, remembers per photo
  whether it was hidden, and pushes a `hide` step; `undoHide` puts back the photos still as the
  step left them (with who changed the others).
- After hiding, S02's effect clears the picks and puts the selection on the photo now at the same
  place, the next shown one.

## Files affected

- `src/App.tsx`, `src/components/PhotoMenu.tsx`, `src/lib/undo.ts`, `src/lib/marks.ts`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `pnpm test`: undoing a hide (one shown again meanwhile left alone), the texts; 81 tests. `tsc`
  and `pnpm build` clean.
