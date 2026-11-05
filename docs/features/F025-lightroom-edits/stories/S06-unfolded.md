# Story S06: Variants unfolded in the grid

**Feature:** `F025-lightroom-edits`
**Status:** Planned
**Depends on:** S04

## Goal

Decision 1: a photo's versions are shown like the members of a stack, folded to the chosen one,
unfolded with a click, so they can be compared side by side in the grid.

## Acceptance criteria

- A tile with versions has the stack's fold button; unfolded, a tile per version next to it, the
  chosen one marked; a click on a version's tile and *Deze versie tonen* (right-click) choose it.
- In a list the same.
- **Tests**: `pnpm test` for the tiles of an unfolded photo and how they combine with a stack.

## Progress log

### [2026-11-05] Written

Split from S04: tiles are by hash, and the versions share theirs; to judge with Peter once
renditions are in the app.
