# Story S01: The plan of a rename or merge, as a dry run

**Feature:** `F022-rename-sources`
**Status:** Done (2026-11-04)
**Depends on:** —

## Goal

Before anything moves, Peter sees what a rename or merge would do, read from the NAS itself.

## Acceptance criteria

- **Input**: one or more sources (`Hero7`, `GoProH7`) and the target name (`GoPro-Hero7`), which
  must be a valid source name (data contract 3.1) and not one of the sources.
- **Read from the NAS**: every month's manifest in `Library` (only `YYYY/MM` folders; the old
  layout is never touched), and in `Previews` whether a preview and thumbnail exist.
- **Per month**: every original of those sources with its sidecars, from its path to the same path
  with the source's folder renamed; whether its preview and thumbnail move along.
- **Clashes**: a target path the month has already with other content gets `~<hash>` (contract
  3.3); the same content there already is a duplicate, listed and left where it is.
- **Totals**: files, sidecars, previews, bytes, and the bytes the Drive copy will upload again
  (without `lrv` and `thm`, which don't go to Drive).
- Nothing is written anywhere.
- **Tests**: `cargo test` for the targets (with and without day folders, split), sidecars,
  previews, clashes, duplicates, the old layout left out, and a refused target name.
- **By hand**: the dry run of `Hero7` + `GoProH7` → `GoPro-Hero7` on the NAS, read only.

## Files affected

- `src-tauri/src/rename.rs` (new), `src-tauri/examples/rename-plan.rs`.

## Progress log

### [2026-11-04] Written

### [2026-11-04] Built and run on the NAS, read only

- `rename.rs`: `plan` reads every ordinary month's manifest on the NAS (the old layout left out),
  and per original of the sources its new path (only the source's folder renamed), its sidecars,
  whether its preview is there; a clash gets `~<hash>`, the same content under the target is a
  duplicate left alone. Totals with the bytes Drive uploads again (without `lrv`, `thm`).
- `examples/rename-plan.rs` prints the plan.
- `cargo test` 208 passed (2 new), `cargo clippy --all-targets` clean.
- **`Hero7` + `GoProH7` → `GoPro-Hero7`** on the NAS: 2019/02, 2019/03 and 2019/05 one or two
  films each, without day folders; 2020/08 291 moves into four day folders and one
  `Time Lapse 1` folder below one of them. Together 209 originals, 86 sidecars, 183 previews,
  75.0 GB, of which 74.3 GB go to Drive again; no duplicates, no clashes.
