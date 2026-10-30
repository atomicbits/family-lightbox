# Story S06: One file that fails doesn't stop the sync

**Feature:** `F007-sync-from-nas`
**Status:** Planned
**Depends on:** S05

## Goal

A file the NAS doesn't deliver in time (Peter saw `Operation timed out (os error 60)` on one
preview) is tried once more and otherwise skipped; the sync goes on with the rest, and the next
sync tries it again. Only when the NAS stops answering altogether does the sync stop.

## Acceptance criteria

- **One retry**: a copy that fails is tried again after a short pause (1 second).
- **Skipped**: when the retry fails too, the file is skipped (nothing half-written stays
  behind), counted, and the copy goes on.
- **The NAS gone**: after 10 failed files in a row the sync stops with the last error, as today.
- **The outcome**: the number of skipped files and, for at most 50 of them, the path and the
  error; the sync ends as *done*, and the line says "bijgewerkt: 120 previews · 1 overgeslagen"
  and fades like any other done line. The details go to the messages of feature 16.
- **Planning** (listing folders) keeps stopping on an error, as today: without a complete plan the
  previews count would be wrong.
- **Tests** (`cargo test`): a file failing once and then copied, a file failing twice and skipped
  with the rest copied, ten in a row stopping the sync; `pnpm test` for the line.

## Files affected

- `src-tauri/src/sync.rs`, `src-tauri/src/pull.rs`, `src-tauri/src/runner.rs`, `src/lib/sync.ts`.

## Progress log

### [2026-10-30] Written
