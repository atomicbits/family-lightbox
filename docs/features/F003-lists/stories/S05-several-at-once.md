# Story S05: Selecting several photos and adding or removing them together

**Feature:** `F003-lists`
**Status:** Planned
**Depends on:** S03

## Goal

Shift-click selects a range, ⌘-click adds or removes one from the selection, and `+` then puts
all of them in the active list (or takes them out, when all are in it already).

## Acceptance criteria

- **Selecting**: shift-click and shift-arrows extend the selection, also across months;
  ⌘-click toggles one; Esc clears it. The header says how many are selected.
- **Adding**: `+` adds every selected photo not yet in the active list; when all are in it, it
  removes them. One event per photo, written in one go.
- **Tests**: `pnpm test` for the selection ranges across sections and the add-or-remove decision.

## Files affected

- `src/lib/timeline.ts`, `src/App.tsx`, `src/components/Grid.tsx`, `src-tauri/src/lists.rs`
  (several events in one write).

## Progress log

### [2026-10-29] Written
