# Story S04: Making, adding to and taking out of a stack, with undo

**Feature:** `F015-stacks`
**Status:** Planned
**Depends on:** S03

## Goal

The right-click menu makes a stack from the selection, adds photos to the one stack selected,
takes a photo out, or dissolves a stack; each with the message and undo.

## Acceptance criteria

- **Reeks maken**: two or more selected photos, none in a stack.
- **Aan de reeks toevoegen**: photos of exactly one stack selected, with photos outside any stack;
  not offered with photos of two stacks.
- **Uit de reeks halen**: on a photo (or the selected photos) of a stack.
- **Reeks opheffen**: takes all photos of the stack out.
- **The message with undo**: "reeks van 40 foto's gemaakt", "5 foto's aan de reeks toegevoegd",
  "1 foto uit de reeks gehaald", "reeks opgeheven"; undo puts each photo back in the stack it
  was in (or none), leaving alone photos changed since.
- **Tests** (`pnpm test`): the menu items for each kind of selection, the texts, undo.

## Files affected

- `src/App.tsx`, `src/lib/stacks.ts`, `src/lib/undo.ts`, `src/lib/marks.ts`.

## Progress log

### [2026-10-30] Written
