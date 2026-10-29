# Story S03: Lists in the app: the sidebar, the active list, adding and removing

**Feature:** `F003-lists`
**Status:** Planned
**Depends on:** S02

## Goal

The lists show in the sidebar, per year, and Lynn and Peter create, rename and archive them
there. One list is active; one key or a button puts the selected photo in it or takes it out,
and every photo in it carries a mark in the grid.

## Acceptance criteria

- **The sidebar**: below the months, a section *Lijsten* with the lists per year (newest year
  first), each with its count; archived lists behind *Gearchiveerd tonen*. A *+* creates one:
  name and year (the year of the month on screen, changeable).
- **Rename and archive** from a list's menu (right-click or a small ⋯).
- **The active list**: clicking a list makes it active (highlighted); it stays active after a
  restart (kept per Mac in the app's state).
- **Adding and removing**: `+` (or `=`) on the grid or in the lightbox toggles the selected
  photo in the active list; a button in the lightbox does the same. Without an active list the
  key asks to choose one.
- **The mark**: a tile whose photo is in the active list shows a small mark; the lightbox says
  "in <list>".
- **Without the NAS**: the lists show (from the local copy), the buttons and the key are disabled,
  with the reason as a tooltip.
- **Tests** (`pnpm test`): the grouping per year, the texts, the toggle decision.

## How

- `src/lib/lists.ts` (state, hooks, texts), `src/components/ListsPanel.tsx`, changes to
  `MonthList`, `Grid`, `Lightbox` and `App`.

## Files affected

- The files above, and `src-tauri/src/config.rs` (the active list in the app's state).

## Progress log

### [2026-10-29] Written
