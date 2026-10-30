# Story S01: The log file and the messages from the Rust core

**Feature:** `F016-messages`
**Status:** Done
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

### [2026-10-30] Built

- `log.rs`: `Message` (time, level, source, text) and its one-line form, `write` with the
  rotation at `MAX_BYTES`; `report(app, level, source, text)` in `lib.rs` writes and emits
  `message`. Commands `log_message` (the UI's own: a mark write that failed after its retry, a
  list change refused) and `log_path`.
- Reported so far: the sync's skipped files (and how many more than the 50 kept) and a failed
  sync, the index's error and unreadable manifests, the previews job's failures and NAS write
  error. The NAS check and updates show their state in the UI already; they follow in S02 where
  they fade.
- 2 Rust tests (the line, appending and rotating); 135 Rust tests, clippy clean. Data contract 7
  lists the log file.
