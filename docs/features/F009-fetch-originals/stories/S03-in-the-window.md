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
