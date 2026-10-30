# Feature 17: Live Photos

**Status:** In progress (plan accepted 2026-10-30)
**Depends on:** Feature 8 (the app's own index)
**Folder:** `F017-live-photos`

## Goal

An iPhone keeps a Live Photo as a photo and a short video with the same number (`IMG_1234.HEIC`
and `IMG_1234.MOV`); the import puts them in different folders, so the grid shows thousands of
short videos that are mostly the moving part of a photo that is there too. The app recognizes
them and marks them with a subtle symbol of their own, so they can be told apart from real
videos, and later filtered, stacked with their photo or hidden.

## Background

- **Measured on 2026-10-30** in the app's index: 16 608 `.MOV` files; 11 672 have a photo with the
  same IMG number in the same month; 6 418 of those were taken within 3 seconds of it, almost all
  from the iPhones, with a median size of 4.3 MB (90 % under 5.6 MB). The number alone isn't
  enough: an iPhone starts again after IMG_9999.
- **The duration** of a video isn't known: the manifests don't have it and the previews job
  doesn't keep it. It isn't needed for this.
- **The index** (feature 8) has every item's path, month, kind and `taken`; it is built by the app
  alone, so a table of its own doesn't touch the CLI.

## Scope

### In scope

- **Recognizing**: a `.MOV` video is a Live Photo when the same month has a photo with the same
  IMG number (`IMG_1234`, `IMG_E1234`, also with the import's date prefix) taken within 3 seconds
  of it. Done when the index is built, kept in a table of the app's index.
- **Showing**: on the tile and in the lightbox, a subtle symbol of its own instead of the camera
  (three concentric circles, as Apple's), with "Live Photo bij IMG_1234.HEIC" as its tooltip.

### Out of scope, for later

- A filter for Live Photos (with the filters feature).
- Stacking a Live Photo with its photo, or hiding them by default.
- Playing the video on the photo.

## Acceptance criteria

- About 6 400 of the `.MOV` files show the Live Photo symbol; a video without such a photo keeps
  the camera.
- `cargo test` covers the rule (the number in its forms, the 3 seconds, another month, a `.MP4`,
  a number repeated after IMG_9999 but far apart in time); `pnpm test` the symbol's text.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-recognizing.md) | Recognizing Live Photos when the index is built; the photo each belongs to | Planned | — |
| [S02](stories/S02-the-symbol.md) | The symbol on the tile and in the lightbox | Planned | S01 |
