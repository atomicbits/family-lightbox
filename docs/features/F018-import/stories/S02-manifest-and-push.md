# Story S02: Writing a month's manifest and copying originals to the NAS, verified

**Feature:** `F018-import`
**Status:** Done (2026-10-30)
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

## How

- `manifest.rs`, a port of the CLI's `build_manifest` / `verify_month` / `merge_manifest` and the
  NAS script's hash reuse:
  - `build(root, folder, device, existing)`: the files of a year subfolder (without the
    exclusions of section 1 and the manifest), sidecars hung on the file with the same stem and
    an orphan as its own row, dates by 3.4 and the camera (S01), the hash reused from `existing`
    when size and `mtime` are the same, rows in the CLI's field order, sorted by path.
  - `verify(local, remote, sizes)` → rows on the NAS, missing paths, conflicts (another size, or
    another hash in the NAS's manifest).
  - `merge(local, remote, ok, sizes)`: the NAS's rows whose file is there with that size, then
    the verified local rows.
  - `write(path, manifest)`: JSON with an indent of one, as Python's `json.dumps(indent=1)`, to
    `<name>.<process id>.tmp`, `fsync`, rename.
- `push.rs`: `push_month(local, share, manifest)`:
  1. every original and sidecar to the share: through `<name>.<process id>.tmp`, keeping its
     modification time, and put in place with `renamex_np(RENAME_EXCL)` on macOS, which refuses
     when the name exists, so a file that appears meanwhile is never overwritten; only `.xmp`
     replaces an older one. A target that is there already with the same size counts as there;
     with another size it is a conflict and stays untouched;
  2. the sizes on the share read back for the folder, the NAS's manifest read;
  3. `verify` and `merge`, the manifest written on the share and next to the local files.
- Not here: deciding target paths and `~<hash>` names (S04), the job around it (S05).

## Files affected

- `src-tauri/src/push.rs` (new), `src-tauri/src/manifest.rs` (new), `src-tauri/src/capture.rs`
  (`resolve_with`), `src-tauri/examples/manifest-build.rs` (new), `docs/data-contract.md`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `manifest.rs`: `is_excluded`, `folder_month`, `folder_files`, `build` (reads each photo and
  video once for date and camera through `capture::resolve_with`), `verify`, `merge`, `sizes`,
  `read`, `to_text` (Python's `indent=1` layout), `write` (temporary file, `fsync`, rename).
- `push.rs`: `copy_file` (temporary name, time kept; an original never overwritten, checked
  first and again by `renamex_np(RENAME_EXCL)` with a fallback for file systems without it; an
  `.xmp` replaced only when size or time differ, rsync's quick check) and `push_folder`. A folder
  the NAS doesn't have gets no empty manifest when nothing arrived.
- Data contract section 8: the row for the app's originals.
- Tests: 8 new (exclusions and months; `build` against the contract: order, sidecars, orphans,
  exclusions, source, camera, field order; hash reuse; `verify` and `merge`; the written text
  byte for byte; copying without overwriting, the exclusive rename, the time; a folder onto a
  "share" with a file of another Mac, a conflict, a renewed `.xmp`, a second push; a file that
  can't be read). `cargo test` 162 passed, clippy clean.
- **Read-only on real data**: the app's manifest of `2026/04` (all 351 hashed again, 14.5 s for
  3.8 GB) and of `2026/08` (775, hashes reused) equal the NAS's manifests in every field except
  `generated_*` and `mtime` (not compared). Verified against `/Volumes/Library` without writing
  (`MANIFEST_VERIFY`): 351 and 775 verified, nothing missing, no conflicts.
- Still to check with a real write: the rights of a file the app creates over SMB (section 2:
  the group must be able to read). Proposed first write: a push of `2026/04`, which copies
  nothing (all there) and only writes the manifest again; then in S05 a real import.

### [2026-10-30] The first write to the NAS

With Peter's go: `examples/push-folder` pushed `2026/04` from the local library to
`/Volumes/Library` (the NAS's manifest saved aside first). Copied 0, there 351, verified 351,
nothing missing, no conflicts; the manifest on the NAS has the same 351 rows, now with
`generated_by` `mbp-peter`, and the local one equals it; no `.tmp` left. On the NAS the manifest
written over SMB is `-rwxr--r-- peter:users`: the group can read it, as section 2 asks. The 14 s
are the stats over SMB (about 7 s per pass over 351 files); with the hashes reused, building the
manifest takes 0.04 s.

Hashing was 287 MB/s against 1.4 GB/s for Python's (OpenSSL's) SHA-256: `sha2` 0.10 only uses
the CPU's SHA instructions on aarch64 with its `asm` feature. With it: 1 489 MB/s, the same hash.
