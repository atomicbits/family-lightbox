# Feature 24: Exporting a list from its originals

**Status:** Done (2026-11-05)
**Depends on:** Feature 3 (lists), feature 9 (fetching originals)
**Folder:** `F024-export`

## Goal

Lynn and Peter make the files for a photo book, prints or a calendar from a list, without
Lightroom: from the originals, never the previews, at the size the product asks for. Lightroom's
edits are not in these files (they are in Lightroom's catalog): for edited photos the export from
Lightroom (feature 23) is the way, and the window says so.

## Scope

### In scope

- **From a list** (or the photos picked): *Exporteren…* with a preset; the originals not here are
  fetched first (feature 9).
- **Presets**, JPEG in sRGB unless said otherwise, never enlarged, the long side in pixels:

  | Preset | Long side | Quality | For |
  |---|---|---|---|
  | *Delen* | 2 048 px | 85 | mail, messages, a screen |
  | *Afdruk 10×15* | 1 800 px | 92 | 10 × 15 cm at 300 dpi |
  | *Afdruk 13×18* | 2 126 px | 92 | 13 × 18 cm at 300 dpi |
  | *Afdruk 20×30* | 3 543 px | 92 | 20 × 30 cm at 300 dpi |
  | *Fotoboek A4* | 3 508 px | 92 | a page of an A4 photo book at 300 dpi |
  | *Kalender A3* | 4 961 px | 92 | an A3 calendar page at 300 dpi |
  | *Volle resolutie* | as the original | 95 | anything else; RAW developed by macOS |
  | *Originelen* | as they are | — | the files themselves, RAW and video too, with their sidecars |

  The DPI field of each JPEG says 300 for the print presets, so a print service reads its size.
- **Videos**: copied as they are in every preset (no conversion); Live Photos' clips left out.
- **Where**: by default `Exports/<year>/<year>-<month>-<list>/` on the NAS with its `export.json`
  (data contract 6), or a folder chosen on this Mac (a book service's upload folder).
- **Names**: `001-<original name>.jpg`, numbered by date taken, then name (the lists' order), so a
  service reads them in that order; the date taken and the camera kept in the JPEG's metadata.
- **Background** job with its progress in the footer, resumable; a second export of the same list
  and preset into the same folder adds only what is missing.

### Out of scope

- Cropping to a page's ratio, layouts, ordering by hand.
- Lightroom's edits (feature 23 exports those from Lightroom; feature 25 may bring them back).

## Acceptance criteria

- A list exported as *Fotoboek A4* gives one JPEG per photo, 3 508 px on its long side (or the
  original's size when smaller), sRGB, 300 dpi, numbered by date, the videos copied, in
  `Exports/…` with an `export.json` naming the list.
- RAW originals (CR3, NEF, DNG) and HEIC come out as JPEG; an original that can't be read is
  reported, the rest goes on.
- `cargo test` for the presets' sizes, names and order, the resumed export; by hand on a list.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-rendering.md) | Rendering one original at a preset (sizes, sRGB, DPI, metadata), RAW and HEIC | Done | — |
| [S02](stories/S02-the-export-job.md) | The export job: fetch, render, name, `export.json`, resumable, in the footer | Done | S01, feature 9 |
| [S03](stories/S03-in-the-window.md) | In the window: *Exporteren…* on a list, the presets, the folder, the note about edits | Done | S02 |
