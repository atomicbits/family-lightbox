# Story S03: Hiding and unhiding from the right-click menu, with undo

**Feature:** `F014-hide-photos`
**Status:** Planned
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

## Files affected

- `src/App.tsx`, `src/components/PhotoMenu.tsx`, `src/lib/undo.ts`, `src/lib/marks.ts`.

## Progress log

### [2026-10-30] Written
