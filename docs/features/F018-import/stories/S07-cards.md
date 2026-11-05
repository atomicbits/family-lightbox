# Story S07: Cards and devices

**Feature:** `F018-import`
**Status:** In progress: built and tested; to try with a card
**Depends on:** S05

## Goal

A camera's card or a device that mounts as a disk with `DCIM/` can be imported straight from the
app, on either Mac, with the same result: Lynn reads a GoPro card on hers (scenario 7).

## Acceptance criteria

- The card appears in the import view as soon as it is mounted, with the source proposed (S03).
- Nothing on the card is written, moved or deleted; files already in the library (by hash) are
  skipped, so the same card can be read again.
- Afterwards the app offers to eject the card.
- Camera-specific companions (`.LRV`, `.THM` next to a GoPro video) follow their file, as in the
  drop folder.

## Files affected

- `src-tauri/src/sources.rs`, `src-tauri/src/mounts.rs`, `src/components/ImportView.tsx`.

## Progress log

### [2026-10-30] Written

### [2026-11-05] Built

- What was there: a mounted volume with `DCIM/` is found as a card, with the source proposed from
  its camera (S03), and planned like a drop folder.
- What was missing: an import from a card would have moved its originals to `_imported/` on the
  card, as from a drop folder. `SourcePlan.copy_only` (set by `plan_source` for a card) makes the
  job copy only, as a synced folder: nothing on the card is moved, what it took is recorded.
- The import view looks for sources again every 4 s while open, so a card shows as soon as it is
  mounted; a card's row says *alleen kopiëren: op de kaart verandert niets* and has *Uitwerpen*
  (`eject_card`: only a volume under `/Volumes` with `DCIM/`, through `diskutil eject`).
- GoPro's `.LRV`, `.THM` follow as in the drop folder (they are in the plan by their kind).
- `cargo test` 221 passed (1 new: a card copied only), `pnpm test` 166 passed, `tsc`, `vite
  build` and `cargo clippy --all-targets` clean.
