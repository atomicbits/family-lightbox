# Story S03: In the window

**Feature:** `F024-export`
**Status:** In progress: built; to try in the app
**Depends on:** S02

## Acceptance criteria

- A list's menu: *Exporteren…* opens a dialog with the presets (their size in pixels and what for),
  the destination (*Exports op de NAS* or *Kies een map…*), the count and size, and a note that
  Lightroom's edits aren't in it (export those from Lightroom); then the progress in the footer.
- **Tests**: `pnpm test` for the presets' texts and the destination's name.

## Progress log

### [2026-11-05] Built

- A list's menu: *Exporteren…* opens `ExportDialog`: the presets with their size and what for,
  what it makes (*602 foto's als JPEG, 33 video's ongewijzigd*), the folder (*Exports op de NAS:
  2026/2026-11-aosta-fotoboek-a4*, or *Kies een map…*), and that Lightroom's edits aren't in it.
- `exporter.ts`: the job kept by the app, resumed by itself; the footer's progress with a bar;
  the end as a message. `pnpm test` 165 passed (2 new), `tsc` and `vite build` clean.
