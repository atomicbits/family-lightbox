# Story S05: Selecting several photos and adding or removing them together

**Feature:** `F003-lists`
**Status:** In progress: built and tested; to try in the app
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

## How

- `timeline.ts`: `rangeBetween` (every tile between two, across sections), `pickAfterClick`
  (shift: the range from the anchor; ⌘ or Ctrl: add or remove one, keeping the tile that was
  selected; a plain click: start over), `selKey` / `parseKey`.
- `App.tsx`: the picked tiles as `section:index` and the anchor; shift-arrows extend the range,
  plain arrows clear it, Esc clears it; the header says "N geselecteerd". After an index rebuild
  the picks are cleared, since positions may have moved.
- `+` with more than one picked: the items are gathered (months not loaded yet are fetched), and
  `manyChange` adds those not in the active list, or removes all when all are in it, as one
  `list_add` or `list_remove`, so one `write` on the NAS.
- `Grid.tsx`: picked tiles get a light blue ring and tint.
- **Dragging** (added after Peter's first try): a tile is draggable; dragging one of the picked
  tiles drags all of them, another tile only itself. A list in the sidebar lights up under the
  drag and takes the photos not in it yet on drop, active or not. The window's native drop
  handler is off (`dragDropEnabled: false`): it swallows HTML5 drag and drop, and the app takes
  no dropped files.
- **Right-click** on a tile (or on the picked ones): the app's own menu, with *Openen* for one
  photo, *Uit <actieve lijst> halen*, and *Toevoegen aan* every list that isn't archived. The
  webview's own menu (Open Image in New Window, …) is switched off outside text fields.
- A message in the header says what happened: "12 foto's toegevoegd aan Ski", "3 foto's al in
  Ski", or why it couldn't.

## Files affected

- `src/lib/timeline.ts` and its tests, `src/lib/lists.ts` and its tests, `src/App.tsx`,
  `src/components/Grid.tsx`. Several events in one write were already in S02.

## Progress log

### [2026-10-29] Written

### [2026-10-29] Built

- Moved ahead of S04: Peter tried shift-click and ⌘-click in the grid while trying S03.
- `pnpm test`: ranges across sections in both directions, the three kinds of click, adding the
  missing ones or removing all; 65 tests. `tsc` and `pnpm build` clean.

### [2026-10-29] Dragging and the right-click menu

- Peter picked several photos and then tried to drag them to a list and to right-click them;
  neither did anything, and `+` wasn't discoverable. Both added as above; the hint on "N
  geselecteerd" names all three ways.
