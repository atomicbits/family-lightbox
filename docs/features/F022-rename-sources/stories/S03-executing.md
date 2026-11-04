# Story S03: Executing on the NAS

**Feature:** `F022-rename-sources`
**Status:** Done (2026-11-04)
**Depends on:** S01, S02

## Progress log

### [2026-11-04] Written and built

- `rename::Job` (the record and the months done, in the app's state under `renames/`) and
  `execute`: per month the originals and sidecars, then previews and thumbnails (never over a
  file; one already at its new place counts as done), the manifest rewritten and read back, every
  move checked (at its new place, not at its old one, the manifest not naming the old path), the
  empty folders of the old sources removed; the journal saved after every month; at the end the
  record written to `Lists/renames/` through a temporary name.
- `lib.rs` `rename_start`: only with a snapshot of today confirmed, not during a sync or an import
  (it shares the import's lock, so the sync and the previews wait); a pending rename is continued,
  another one refused until it is done; afterwards applied here at once (S04), the months'
  manifests copied from the NAS, the index rebuilt; `rename-progress` and `rename-done`.
- Tests: a whole rename resumed after an earlier run moved a file, the old folders gone, an empty
  folder of another name kept, the manifest's paths, sidecars and source, the record; a file in
  the way stops the month and keeps the journal.

### [2026-11-04] The first rename, and closing the window meanwhile

- Peter merged `Hero7` and `GoProH7` into `GoPro-Hero7` in the app, after a snapshot. Checked on
  the NAS: 2019/02, 2019/03, 2019/05 and 2020/08 have all 209 originals under `GoPro-Hero7` at the
  planned paths, no file or sidecar missing, no manifest naming the old sources, the old folders
  gone, 201 previews moved on the NAS and on this Mac, the record of 295 moves in
  `Lists/renames/`, applied here, the journal removed; the index has 209 under `GoPro-Hero7`. The
  old-layout folders (`2019/02.Ski/GoProH7`) are untouched.
- Peter asked whether closing the settings would interrupt it. It doesn't: the rename runs in the
  core. But its progress was the tab's state, so it was lost on closing, and an interrupted one
  waited for *Doorgaan*. Now `useRenamer` (`renamer.ts`) keeps it in the app, the footer shows
  *hernoemen: maand 2 van 4* with a spinner, the tab says the window may be closed and the app
  restarted, and an interrupted rename continues by itself once the NAS is there, without asking
  for the snapshot again (it was confirmed when it started).
- What an interruption leaves: at most one month whose files moved partly while its manifest
  still has the old paths, until it continues; nothing is lost or written over.
