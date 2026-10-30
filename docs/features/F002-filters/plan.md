# Feature 2: Filters

**Status:** In progress (plan accepted 2026-10-30; S02 done, S03 built)
**Depends on:** Feature 13 (stars and hearts), feature 17 (Live Photos), and the CLI's feature 11 in LibraryLocal (the camera in the manifest)
**Folder:** `F002-filters`

## Goal

A simple filter in the toolbar, centered above the grid, after Lightroom Classic's attribute
filter: stars, hearts, the kind of photo and the camera. The grid, the month list and a list show
only what matches; the filter stays after a restart, and the toolbar shows clearly that one is
set. The date needs no filter: the month list is there for that.

(This feature was "Filter by source"; Peter widened it on 2026-10-30.)

## Background

- **Lightroom Classic** filters with a bar above the grid: text, attributes (flag, rating with
  ≥ = ≤, color label, kind) and metadata columns. Peter chose the attributes only, without flag,
  color labels and virtual copies.
- **Stars and hearts** (feature 13) are in the marks, by hash; **Live Photos** (feature 17) are
  in the app's index; **hidden photos** (feature 14) stay hidden as they are.
- **The camera** isn't known today: the app has only the import folder (`source`), which mixes
  device, owner and photo/video, and is missing for 45 086 older photos. Peter chose to do it
  properly: the camera's make and model in the manifest, from the originals' metadata, for all
  years (LibraryLocal feature 11), before the camera filter.
- **The toolbar** has the place, the tile-size slider and undo; its middle is free since the stars
  moved onto the selected tile.

## Scope

### In scope

- **Stars**: ≥, = or ≤ a number of stars (1–5), or "without stars".
- **Hearts**: "anyone's", or per person (tick one or more people).
- **Kind**: photo, video, Live Photo; one or more.
- **Camera**: one or more of the cameras in the library, with "unknown".
- **Combined** with AND across these four; within kind and camera, OR.
- **Where**: the library and a list; the month list counts what matches and dims months with
  nothing.
- **Kept**: on this Mac, after a restart; a filter that is set shows clearly in the toolbar, with
  a way to clear it at once.
- **The toolbar**: the filter centered above the grid; the slider and undo stay at the right.

### Out of scope

- Flag, color labels, virtual copies, lens, date, keywords, text search, local or NAS, in a list
  or not, who rated (perhaps later), presets.

## Acceptance criteria

- "★ ≥ 4" and "iPhone 13 Pro" show only those photos, in the grid and with matching counts in
  the month list, within about 0.2 s of a change.
- "Live Photo" alone shows the 6 400 Live Photo videos; adding "foto" shows the photos too.
- After a restart the filter is still set, and the toolbar says so.
- `cargo test` covers the filter in the index (each attribute, AND and OR); `pnpm test` the bar's
  state, its text and keeping it.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-camera-in-the-index.md) | The camera in the app's index, from the manifests | Planned | LibraryLocal F011/S01 |
| [S02](stories/S02-filtering-in-the-core.md) | Filtering in the Rust core: the months with their counts and a month's items for a filter | Done | — |
| [S03](stories/S03-the-filter-bar.md) | The filter bar in the toolbar: stars, hearts, kind, camera; kept after a restart | In progress: built, to try in the app | S02 |
| [S04](stories/S04-camera-filter.md) | The camera in the filter bar | Planned | S01, S03 |
