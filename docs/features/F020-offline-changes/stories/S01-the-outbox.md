# Story S01: The outbox: changes kept on this Mac, and read with the lists and marks

**Feature:** `F020-offline-changes`
**Status:** In progress: built and tested; to check in the app
**Depends on:** —

## Goal

A change to a list, a star, a heart, a hidden photo or a stack is kept on this Mac when it can't be
written to the NAS, and shows as if it had been, also after a restart.

## Acceptance criteria

- **Where**: `~/.config/media/outbox/events.jsonl` for list events and `outbox/marks.jsonl` for
  marks (decision 2), in exactly the format of data contract 5.2 and 5.5, this device's lines,
  each change in one `write` followed by `fsync`.
- **When**: when the `Lists` share isn't connected, and also when writing to it fails
  (decision 1). With the share there and the outbox empty, a change goes to the share as now.
- **Reading**: the lists and the marks are merged from the share, or the local copy without it,
  with the outbox's lines read as if they stood at the end of this device's own file there, so the
  merge of 5.3 stays as it is.
- **Refused offline**: only what is refused online too, and a change on a Mac that was never
  registered (no `devices.json` in the local copy).
- **Tests**: `cargo test` for a change without the share (kept, merged, after a restart), a
  failed write going to the outbox, and the order of lines from the file and the outbox.

## Files affected

- `src-tauri/src/outbox.rs` (new), `lists.rs`, `marks.rs`, `lib.rs`.

## Progress log

### [2026-11-04] Written

With Peter's decisions on the plan's open points: also on a failed write; the outbox in the app's
own state; no extra warning about the other Mac.

### [2026-11-04] Built

- `outbox.rs`: `outbox/events.jsonl` and `outbox/marks.jsonl` in the app's state, complete lines
  only, `add` in one `write` and `fsync`, `count`.
- `lists::read_with` and `marks::read_with`: the outbox's lines read after this device's own file,
  so 5.3's order by moment, device and line holds; `load` takes them.
- `lib.rs` `write_change`: to the share when it is there, after sending what waits; to the outbox
  when the share isn't there or refuses; a Mac the local copy doesn't list as registered is
  refused. `change_list` and `change_marks` use it, with the lists as this Mac knows them
  (share or copy, with the outbox) for the refusals and new ids.
- `cargo test` 206 passed (5 new: kept and read after the device's file, sending, a send that
  stopped, an id taken meanwhile, an unregistered device; and a change without the NAS through the
  app's own function).
