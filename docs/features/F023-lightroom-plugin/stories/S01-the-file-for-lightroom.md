# Story S01: The file for Lightroom

**Feature:** `F023-lightroom-plugin`
**Status:** In progress: built and tested
**Depends on:** —

## Goal

The plug-in has one file to read with everything it needs, made by the app, so the Lua side stays
small and the logic is tested in Rust.

## Acceptance criteria

- `~/.config/media/familylightbox-lightroom.json`, written atomically by the app whenever the
  lists change and when the index is rebuilt:
  `{"version": 1, "written_at": …, "lists": [{"id", "name", "year", "archived",
  "photos": [{"path": "<absolute path in the working copy>", "here": true|false}]}]}`.
- A photo's path is its current path in the library from the index (by hash), else the list's
  hint; `here` says whether the original is in the working copy.
- **Tests**: `cargo test` for the file of a few lists (archived too), paths from the index, a
  photo not here, a hash the index doesn't know.

## Progress log

### [2026-11-05] Built

`lightroom.rs`: `build` (per list its photos by their path from the index, by hash, else the
list's hint; `here` from the working copy; sorted) and `write` (atomic). `lib.rs`
`write_lightroom`: after a list's change, after the outbox is sent, after every index build (so
after fetching originals too). On Peter's Mac: *Aosta* 602 photos, all here; *Lucy* 1 456, none
here yet. `cargo test` 214 passed (1 new), `cargo clippy --all-targets` clean.
