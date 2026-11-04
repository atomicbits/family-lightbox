# Feature 21: The settings window

**Status:** Done (2026-11-04)
**Depends on:** Feature 6 (the NAS settings), feature 18 (the import settings), feature 19 (the
layout)
**Folder:** `F021-settings-window`

## Goal

The settings fit in their window and are easy to find. The panel grew with every feature (the
device, the local copies, the index, the previews, the NAS, the move check, the import, updates)
into one narrow column that Peter has to scroll through, also sideways.

## Scope

### In scope

- A wider window, as wide as the app's window allows, without scrolling sideways.
- The topics on tabs, with one *Bewaar* and *Sluiten* for all of them.

### Out of scope

- New settings, or moving settings to other places in the app.
- The English texts (feature 11).

## Acceptance criteria

- At the app's smallest and at a usual window size nothing in the settings needs scrolling
  sideways; long paths wrap or are cut off with the whole path on hover.
- Four tabs: *Algemeen*, *NAS*, *Importeren*, *Index en previews*; the one last open comes back
  the next time the settings open.
- A tab with changes not yet saved says so; *Bewaar* saves the changes of all tabs.
- `pnpm test` covers which tab a change belongs to and the remembered tab.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-wider-with-tabs.md) | A wider settings window with the topics on tabs | Done | — |
