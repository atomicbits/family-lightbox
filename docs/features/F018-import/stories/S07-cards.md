# Story S07: Cards and devices

**Feature:** `F018-import`
**Status:** Planned
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
