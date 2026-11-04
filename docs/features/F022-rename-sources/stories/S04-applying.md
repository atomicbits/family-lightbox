# Story S04: Applying a rename on each Mac

**Feature:** `F022-rename-sources`
**Status:** In progress: built and tested; Lynn's Mac applies it from the next release on
**Depends on:** S02, S03

## Progress log

### [2026-11-04] Written and built

- `rename::records` and `apply`: after every sync, the records in the local copy of `Lists` this
  Mac hasn't applied yet; what it has under the old path moves, a twin the sync already brought to
  the new path (the same size) is dropped; the applied ids are kept in the app's state
  (`renames_applied`); the index is rebuilt when something moved. Applying again changes nothing.
- A Mac with an older app (v0.6.1) doesn't apply it: it keeps its old previews at the old paths
  next to the new ones the sync brings, harmless, until it updates.
