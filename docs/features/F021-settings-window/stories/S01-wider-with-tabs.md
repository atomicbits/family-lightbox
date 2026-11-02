# Story S01: A wider settings window with the topics on tabs

**Feature:** `F021-settings-window`
**Status:** In progress: built; to check in the app
**Depends on:** —

## Goal

The settings in a window that fits, with the topics on tabs instead of one long column.

## Acceptance criteria

- The window is 48rem wide, or the app's width less a margin when that is smaller, and has a
  fixed height (scrolling inside a tab when needed), so it doesn't jump between tabs.
- Nothing scrolls sideways: long paths and share names break or are cut off with the whole text
  on hover.
- The tabs and what is on them:
  - **Algemeen**: this device, the local copies, the version with *Controleren op updates*, and
    *Meldingen*;
  - **NAS**: its name, the home share, the layout, the move check, the shares found with
    *Verbinden* and *Synchroniseren*;
  - **Importeren**: the import settings of feature 18;
  - **Index en previews**: *Index bijwerken*, the missing and failed previews with *Opnieuw
    proberen*.
- A tab with unsaved changes shows a dot; *Bewaar* and *Sluiten* stay at the bottom on every tab.
- The tab last open is remembered on this Mac (in the window's local storage, like the tile size)
  and opens again; an unknown value gives *Algemeen*.
- **Tests**: `pnpm test` for the tab of a change (the NAS fields, the import) and reading the
  remembered tab safely.

## How

- `src/lib/settingsTabs.ts`: the tabs, `changedTabs(draft, saved)`, `loadTab` and `saveTab`.
- `Settings.tsx`: the tab bar, each tab's sections as they are now, the wider window.
- `ImportSettings.tsx`: without its own heading, which is now the tab's name.

## Files affected

- `src/lib/settingsTabs.ts` (new) with its test, `src/components/Settings.tsx`,
  `src/components/ImportSettings.tsx`.

## Progress log

### [2026-11-02] Written

From Peter: the settings are too narrow (scrolling sideways) and have become hard to oversee.

### [2026-11-02] Built

- `settingsTabs.ts`: the four tabs, `changedTabs` (the NAS fields, the import, the local folders)
  and the tab remembered in local storage, read safely.
- `Settings.tsx`: 48rem wide (at most the window), 40rem high (at most the window), the tab bar at
  the top, the tab's content scrolling on its own, *Bewaar* and *Sluiten* at the bottom of every
  tab. Long paths and share names break instead of widening the window; the share rows wrap.
- `ImportSettings.tsx`: its heading left out, the tab names it.
- `pnpm test` 138 passed (2 new), `tsc` and `vite build` clean.
