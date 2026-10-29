# Story S04: Undo, with a message, a button and ⌘Z

**Feature:** `F013-ratings-and-hearts`
**Status:** Planned
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

## Files affected

- `src/lib/undo.ts` (new), `src/components/Toast.tsx` (new), `src/App.tsx`, `src/lib/marks.ts`,
  `src/lib/lists.ts`.

## Progress log

### [2026-10-30] Written
