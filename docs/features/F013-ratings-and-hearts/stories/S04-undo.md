# Story S04: Undo, with a message, a button and ⌘Z

**Feature:** `F013-ratings-and-hearts`
**Status:** Done
**Depends on:** S03

## Goal

Every change to stars, hearts or what is in a list can be undone, step by step, newest first:
from the message that says what happened, from a button that is always in the header, and with
⌘Z (Ctrl-Z on Windows).

## Acceptance criteria

- **What is undoable**: stars, hearts, adding to and removing from lists, made in this window
  since it opened. Not creating, renaming or archiving lists. The history is gone after a
  restart.
- **Each step** remembers, per photo, the value before and the value set; undo writes the values
  before as new events (the event files are never rewritten).
- **The other Mac**: a photo whose value changed since (by someone else) is left alone; the
  message says how many and by whom.
- **The message** at the bottom after every change: what happened ("12 foto's: 3 sterren",
  "hartje op 1 foto", "3 foto's toegevoegd aan Ski") with *Ongedaan maken*; it disappears after
  about 6 seconds, and a new change replaces it.
- **The button**: ↶ at the right of the header, next to ⚙, greyed out with nothing to undo; its
  tooltip says what it would undo.
- **⌘Z** (Ctrl-Z on Windows) undoes the newest step, except in a text field.
- **Tests** (`pnpm test`): the stack, the inverse of each kind of change, skipping photos changed
  meanwhile, the texts.

## How

- `undo.ts`: a step is `stars` (the rating set, and per photo the rating before), `heart` (on or
  off, and per photo whether this person's heart was on) or `list` (what `addTo` or
  `removeFrom` actually changed). `undoStars`, `undoHeart` and `undoList` work out what to write
  back: only the photos still as the step left them, grouped per value, so one write per value;
  the others are counted, with who rated them since. `useUndo` keeps the steps in a ref.
- Before undoing stars or hearts the marks are read again from the share, so a change the 15-second
  watch hasn't seen yet still counts as "changed meanwhile".
- `lists.addTo` and `removeFrom` now answer with what actually changed (and leave the success
  message to the caller); `photosOf(id)` gives a list's photos now.
- `Toast.tsx`: the message at the bottom with *Ongedaan maken*, 6 seconds; after an undo it says
  "ongedaan: …" without a button. Errors stay in the header.
- ↶ left of ⚙, greyed out with nothing to undo, with the step in its tooltip; ⌘Z or Ctrl-Z
  anywhere except in a text field.

## Files affected

- `src/lib/undo.ts` (new), `src/components/Toast.tsx` (new), `src/App.tsx`, `src/lib/marks.ts`,
  `src/lib/lists.ts`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `pnpm test`: stars back per photo grouped by value with a photo rated by someone else left
  alone, hearts back, list adds and removes back, the texts; 77 tests. `tsc` and `pnpm build`
  clean.

### [2026-10-30] Checked

- Peter in the app: "F013 werkt".
