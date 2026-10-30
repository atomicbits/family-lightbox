# Story S02: Writing a month's manifest and copying originals to the NAS, verified

**Feature:** `F018-import`
**Status:** Planned
**Depends on:** S01

## Goal

The app does what `media push` does for a month, natively and over the SMB mount: copy the new
originals into `Library`, read them back, and write the month's manifest, so the files count as
"on the NAS" (data contract 3.5 and 9).

## Acceptance criteria

- Each original is copied through a temporary name `<name>.<process id>.tmp` and a rename, never
  over an existing file (section 1); sidecars (`.xmp`) may be renewed.
- After copying: the size on the share is read back; a file that differs is removed again and
  counts as failed.
- The month's manifest on the NAS is the existing one plus the verified new rows, rows of paths
  that are gone dropped (as `merge_manifest` in the CLI), with `contract_version` 1.3,
  `generated_by` this device, written atomically. The CLI and `media-manifest-nas.py` read it.
- Group read rights as section 2 asks (the share's defaults over SMB; checked on the NAS).
- Data contract section 8 gets the app's transport row.
- **Tests** (`cargo test`): copying into a temporary "share", no overwrite, a failed size, the
  merge of manifests, an interrupted copy leaves no `.tmp` that counts.

## Files affected

- `src-tauri/src/push.rs` (new), `src-tauri/src/manifest.rs` (new), `docs/data-contract.md`.

## Progress log

### [2026-10-30] Written
