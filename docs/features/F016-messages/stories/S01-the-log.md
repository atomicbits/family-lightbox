# Story S01: The log file and the messages from the Rust core

**Feature:** `F016-messages`
**Status:** Planned
**Depends on:** —

## Goal

The Rust core writes every warning and error to a log file and sends it to the window as a
message, with time, level and source.

## Acceptance criteria

- **A module** `log.rs`: `message(level, source, text)` appends a line to
  `~/.config/media/familylightbox.log` (`2026-10-30T21:15:02+01:00 WARN sync …`) and emits a
  `message` event; at 1 MB the file becomes `.log.1` (the old one is dropped) and a new one starts.
- **Sources**: the sync (skipped files, a failed sync), the index (unreadable manifests, a failed
  build), the previews job (failures, writing to the NAS), the lists and marks (failed writes),
  the NAS check, updates.
- **The UI's own messages** (a failed write after its retry) are logged too, through a command.
- **Tests** (`cargo test`): the line format, appending, the rotation at the limit.

## Files affected

- `src-tauri/src/log.rs` (new), `src-tauri/src/lib.rs`, `runner.rs`, `index.rs`,
  `previews_job.rs`, `docs/data-contract.md`.

## Progress log

### [2026-10-30] Written
