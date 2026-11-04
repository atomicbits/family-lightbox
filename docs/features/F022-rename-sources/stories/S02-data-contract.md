# Story S02: The data contract: renaming a source, and its record

**Feature:** `F022-rename-sources`
**Status:** Done (2026-11-04)
**Depends on:** —

## Progress log

### [2026-11-04] Written and done

Data contract 1.6: section 3.6 describes a rename on the NAS (only the source's folder changes;
per month originals and sidecars, then previews, then the manifest, checked; the old layout never
touched; a clash gets `~<hash>`, a duplicate stays; Drive keeps the old paths); section 5.6 the
record `Lists/renames/<id>.json`, written once after the NAS is done, which every device applies
after its next sync; section 1: a rename moves, it deletes nothing, and a Mac applying it may drop
a local twin the sync already brought to the new place. The sync copies `renames/` with `Lists`.
