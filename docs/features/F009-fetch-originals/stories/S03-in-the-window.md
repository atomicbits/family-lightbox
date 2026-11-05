# Story S03: In the window

**Feature:** `F009-fetch-originals`
**Status:** In progress: built and tested; to try in the app
**Depends on:** S02

## Acceptance criteria

- *Originelen ophalen* in the grid's right-click menu (the photos picked) and on a list; first the
  plan's totals (*40 originelen, 1,2 GB; 3 staan er al*), then *Ophalen*; refused when it doesn't
  fit.
- The progress in the footer with a bar and a percentage; a click opens nothing new, the window may
  be closed and the app restarted.
- Opening or showing an original that isn't here offers to fetch it first.
- **Tests**: `pnpm test` for the texts.

## Progress log

### [2026-11-05] Built

- `fetcher.ts`: `useFetcher` keeps the fetch in the app; an interrupted one continues once the
  NAS is there; `fetchPlanText`, `fetchingText`, `fetchDoneText`.
- The grid's right-click menu: *Originelen ophalen (n)* for the photos picked that aren't here,
  and while a list is shown *Alle originelen van <lijst> ophalen (n)*; the Lightbox:
  *Origineel ophalen* on an original that isn't here. First `FetchConfirm` with the plan
  (refused when it doesn't fit), then the progress in the footer with a bar and a percentage, the
  end as a message.
- `pnpm test` 160 passed (2 new), `tsc` and `vite build` clean.

### [2026-11-05] A list's originals from the list itself

- Peter: fetching all of a list's originals belongs on the list, not on a photo in it; and
  *Archiveren* behind the list's ⋯ was too easy to click by mistake.
- The list's own menu, by right-click on its name or by ⋯ (the same menu): *Alle originelen
  ophalen* (from `list_items`, the ones not here, with the plan first), *Hernoemen…*,
  *Archiveren…*, which asks first (`ConfirmDialog`: what archiving means, *Annuleren* focused) or
  *Terugzetten*. The grid's menu keeps *Originelen ophalen (n)* for the photos picked only.
- `pnpm test` 160 passed, `tsc` and `vite build` clean.
