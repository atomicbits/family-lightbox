# Story S01: What to fetch

**Feature:** `F009-fetch-originals`
**Status:** Done (2026-11-05)
**Depends on:** —

## Acceptance criteria

- **Input**: library paths of originals (a selection, a list's photos).
- **From the month's manifest** in the working copy (the sync keeps it current): each original's
  size, hash and sidecars; a path in no manifest is reported, not fetched.
- **Already here**: a local file with the manifest's size is skipped (its hash is checked when it
  is fetched or verified later, not now); a local file of another size is a conflict, reported and
  never written over.
- **Sidecars** come along (`.xmp`, `.AAE`, …), but not `lrv` and `thm` (derived; data contract 8).
- **Totals**: files, bytes, already here, conflicts; and whether it fits on this Mac with the 20 GB
  reserve.
- **Tests**: `cargo test` for here/not here/conflict, sidecars, a path in no manifest, the space.

## Progress log

### [2026-11-05] Built

`fetch::plan`: per path the row of its year subfolder's manifest in the working copy (also the
old `MM.<name>` folders), here with the manifest's size → skipped, another size → a conflict,
missing → fetched with its sidecars but `lrv` and `thm`; a path in no manifest reported; the
bytes, and whether they fit with the 20 GB reserve. Test: here, not here, a conflict, the THM
left out, a path in no manifest, the reserve.

### [2026-11-05] Done

Peter: checked in the app.
