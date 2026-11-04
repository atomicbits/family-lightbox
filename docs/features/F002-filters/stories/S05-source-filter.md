# Story S05: The source in the filter bar

**Feature:** `F002-filters`
**Status:** Done (2026-11-04)
**Depends on:** S03, S04

## Goal

Show the photos and videos of one or more sources: the way they came in (`EOSR`, `iPhone-photos`,
a one-off source such as the photos a trip's organisers sent). Peter imported a one-off source
whose photos are interleaved by time with his own of the same days, and wanted to see them apart.
The camera filter (S04) does that only when the source's camera is nowhere else.

## Acceptance criteria

- A *Bron* button in the filter bar, after *Camera*, with a list like the camera's: *Alle
  bronnen*, then every source with its number of photos and videos and when it was last used,
  sorted by *recent* (default) or *aantal*; the files without a source (the folders from before
  the source level, contract 3.1) as *zonder bron*, always last. More than one can be ticked.
- The source filters with the rest (stars, hearts, kind, camera), in the core for the library and
  in the frontend for a list, as the camera does; the button reads *Bron*, the source, or
  *3 bronnen*; the bar's tooltip and the toolbar name it.
- The choice is kept after a restart with the rest of the filter; a filter kept before this story
  reads back without sources.
- **Tests**: `cargo test` for the source in `Filter::matches` (one, several, none, with the rest)
  and the counts per source; `pnpm test` for the button's text, the filter text, reading a kept
  filter and a list's photos.

## How

- `filter.rs`: `Filter.sources`, `Candidate.source`; `db.rs`: the source in the filtered months
  and items, `source_counts` like `camera_counts`; the command `sources`.
- `filter.ts`: `sources` in `FilterState` and `CoreFilter`, `sourceButton`, `matches` with the
  item's source; `FilterBar.tsx`: the camera's menu made general and used for both.

## Files affected

- `src-tauri/src/filter.rs`, `src-tauri/src/db.rs`, `src-tauri/src/lib.rs`, `src/lib/filter.ts`,
  `src/components/FilterBar.tsx`, `src/App.tsx`, and their tests.

## Progress log

### [2026-11-02] Written

At Peter's request, after the first one-off source.

### [2026-11-02] Built

- `filter.rs`: `Filter.sources` and `Candidate.source`; `db.rs`: the source in the filtered months
  and items, and `source_counts` (photos and videos, latest `taken`, "" last); `lib.rs`: the
  command `sources`.
- `filter.ts`: `sources` in the filter and what the core gets, `sourceButton` and `sourceLabel`
  (*zonder bron* for ""), in `filterText`, `readFilter` and `matches` (for a list's photos).
- `FilterBar.tsx`: the camera's menu is `ChoiceMenu`, with its words per use; *Bron* after
  *Camera*, sorted by *recent* by default, as Peter asked, or by *aantal*. ✕ clears the sources
  too. `App.tsx` reads the sources with the cameras after every index build.
- On Peter's index: 23 sources and about 45 000 files without one (the folders before 2022).
- `cargo test` 189 passed (2 new), `cargo clippy --all-targets` clean; `pnpm test` 143 passed (3
  new), `tsc` and `vite build` clean.

### [2026-11-04] Checked in the app

Peter checked it in the app: OK.
