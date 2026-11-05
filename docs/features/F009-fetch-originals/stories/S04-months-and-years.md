# Story S04: Whole months and years

**Feature:** `F009-fetch-originals`
**Status:** Built: to try in the app
**Depends on:** S03

## Goal

What `media pull` does for a month, from the app: the originals of a whole month or year that
aren't on this Mac, with the plan and its size first (Peter, 2026-11-05).

## Acceptance criteria

- Right-click on a month or a year in the left column: *Originelen ophalen (n)*, n the photos and
  videos there that aren't here; disabled when there are none or a fetch runs.
- The paths come from the index (`items` not local in those months); then the same plan, warning
  above 5 GB, job, footer and resume as S02 and S03.
- **Tests**: `cargo test` for the paths of a month and a year that aren't here.

## Files affected

- `src-tauri/src/db.rs`, `src-tauri/src/lib.rs`, `src/App.tsx`.

## Progress log

### [2026-11-05] Written

### [2026-11-05] Built

- `Db::not_local`: the photos and videos of a month (`2019/07`) or a year (`2019`) that aren't
  local; the command `not_local`.
- The left column's menu (F014/S04) has *Originelen ophalen (n)*, n from the month counts
  (photos + videos − local); disabled when nothing is away or a fetch runs. A click asks the plan
  with `FetchConfirm` as a list's fetch does, the 5 GB warning included.
- `cargo test` 224 passed (1 new), `pnpm test` 168, `tsc`, `vite build`, clippy clean.
