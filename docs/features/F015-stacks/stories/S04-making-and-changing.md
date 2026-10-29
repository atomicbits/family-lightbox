# Story S04: Making, adding to and taking out of a stack, with undo

**Feature:** `F015-stacks`
**Status:** In progress: built and tested; to try in the app
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

## How

- `stackActions(hashes, ids)` decides what the menu offers for the clicked photos: *Reeks maken
  (N)*, *Aan de reeks toevoegen (N)* (only the photos outside any stack, into the one stack
  selected), *Uit de reeks halen*, *Reeks opheffen* (all photos of the stack, also those folded
  away). Nothing of this in a list.
- `changeStack` writes one `marks_stack` (with a new id for a new stack) or `marks_unstack`,
  remembers each photo's stack before, and pushes a `stack` step; `undoStack` puts each photo
  back in the stack it was in, or none, leaving alone photos changed since.

## Files affected

- `src/App.tsx`, `src/lib/stacks.ts`, `src/lib/undo.ts`, `src/lib/marks.ts`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `pnpm test`: the menu for photos outside any stack, one stack with others, two stacks; undo
  back into a stack and out of one with a photo changed meanwhile left alone; the texts. 96
  tests; `tsc` and `pnpm build` clean.
