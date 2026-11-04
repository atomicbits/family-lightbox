# Story S03: Executing on the NAS

**Feature:** `F022-rename-sources`
**Status:** In progress: built and tested; the first rename on the NAS waits for Peter
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
