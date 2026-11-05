# Story S01: Rendering one original at a preset

**Feature:** `F024-export`
**Status:** In progress: built and tested
**Depends on:** —

## Acceptance criteria

- `preview::render`: an original (JPEG, HEIC, PNG, TIFF, RAW such as CR3, NEF, DNG) read by
  macOS, turned upright, at most the preset's long side and never enlarged (or its full size),
  in sRGB, as a JPEG at the preset's quality, with its DPI when the preset has one (300), and with
  the date taken (with its offset) and the camera (TIFF `Model`) in its metadata.
- The presets of the plan, as data: `delen`, `10x15`, `13x18`, `20x30`, `fotoboek-a4`,
  `kalender-a3`, `vol`, `originelen`.
- **Tests**: `cargo test` for the sizes (smaller, larger than the preset, full), the DPI, the
  metadata, a rotated original.

## Progress log

### [2026-11-05] Built

`preview::render` (and `photo_at` with a limit or none): upright, at most the long side, never
enlarged, redrawn in sRGB, JPEG at the preset's quality, `DPIWidth`/`DPIHeight`, the date taken
with its offset, the camera as TIFF `Model`; through a temporary name. `export::PRESETS`: the eight
presets of the plan. Test: a rotated original, 300 px with 300 dpi, date and camera, never enlarged,
full size. By hand on Peter's files: a CR3 and a JPG at *Fotoboek A4*, 3 508 px, 300 dpi, about
half a second each. `examples/render-one.rs` for that.
