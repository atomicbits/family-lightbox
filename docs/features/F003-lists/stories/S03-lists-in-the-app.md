# Story S03: Lists in the app: the sidebar, the active list, adding and removing

**Feature:** `F003-lists`
**Status:** In progress: built and tested; to try in the app
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

- `src/lib/lists.ts`: `useLists` (the state, the active list and its photos by hash, create,
  rename, archive, toggle), `groupLists`, `changeBlocked`, `defaultYear`, `addedText`.
- `src/components/ListsPanel.tsx` below the months in the sidebar (at most 45 % of its height):
  the lists per year, *+* opens a name and year field (Enter creates, Esc cancels), ⋯ opens
  *Hernoemen* and *Archiveren* / *Terugzetten*; a click makes a list active, a second click
  makes none active.
- The active list is kept in `familylightbox-state.json` (`active_list`), through the commands
  `active_list` and `set_active_list`.
- The grid marks tiles in the active list with a blue ✓ and says who added them in the tooltip;
  the lightbox has a button "+ <list>" / "✓ in <list>" and the same key. Messages (no active
  list, without the NAS, a refusal) show in the header for a few seconds.

## Files affected

- `src/lib/lists.ts` and `lists.test.ts` (new), `src/components/ListsPanel.tsx` (new),
  `src/App.tsx`, `src/components/Grid.tsx`, `src/components/Lightbox.tsx`,
  `src-tauri/src/config.rs`, `src-tauri/src/lib.rs`.

## Progress log

### [2026-10-29] Written

### [2026-10-29] Built

- `pnpm test`: the grouping per year with and without archived lists, when changes are blocked,
  the default year (also for a named folder and `0000`), the "toegevoegd door" text; 62 tests.
  `tsc` and `pnpm build` clean; 115 Rust tests, clippy clean.
- To try in the app: create a list, make it active, `+` on a few photos in the grid and the
  lightbox, the marks, rename, archive and show archived; the events on the NAS and the summary.

### [2026-10-29] The active list replaced by showing the list

- Peter, trying it: with a list clicked, the grid still showed all photos with a blue ✓ on the
  ones in the list, the month list kept its highlighted month, and right-click on a photo
  without ✓ offered to take it out of the list. Confusing.
- Now clicking a list shows only its photos in the grid (S04), without marks, and no month is
  highlighted; clicking it again, *← Alle foto's* or a month returns to the library. Adding is
  done by dragging to a list or *Toevoegen aan* in the right-click menu; in a list, ⌫ or *Uit
  <list> halen* takes photos out. The `+` key, the ✓ marks, the lightbox button and the stored
  active list are gone; `+` in the library says how to add instead.
