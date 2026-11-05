# Story S02: Sending the outbox once the NAS is there

**Feature:** `F020-offline-changes`
**Status:** Done (2026-11-05)
**Depends on:** S01

## Goal

What waits in the outbox reaches the NAS by itself when the `Lists` share is there, without a
line lost or written twice, and a list made offline stays a list of its own.

## Acceptance criteria

- **When**: when the share connects (at start, after a quiet reconnect of feature 6/S08, after
  *Verbinden*), and before every change written to the share, so the order on the share is the
  order of the changes.
- **How**: the outbox's lines are appended to this device's files on the share in one `write`
  and `fsync` per file, unchanged (their original moment); then the file is read back, and only
  when every line is there is the outbox emptied.
- **Without doubles**: a line already in the share's file (a send that stopped after writing,
  before emptying) isn't written again.
- **A list's id taken meanwhile**: a list created offline whose id another device created on the
  share meanwhile gets the next free id (`2026/vakantie-2`); every line of the outbox naming it is
  rewritten, and the outbox saved, before anything is sent, so a send that stops halfway sends the
  same lines again.
- **Tests**: `cargo test` for sending (appended, emptied), a send that stopped (no doubles), the
  id taken meanwhile, and nothing sent when the share's `devices.json` doesn't list this device.

## Files affected

- `src-tauri/src/outbox.rs`, `lib.rs`.

## Progress log

### [2026-11-04] Written

### [2026-11-04] Built

- `outbox::send`: per kind, a list created offline whose id another device created meanwhile gets
  the next free id in every line, saved before sending; lines already in the share's file are left
  out; the rest in one `append_lines`; the file read back, and the outbox emptied only when it has
  every line. `outbox_send` (command) sends when the share is there and writes the summaries of
  the lists it touched; `write_change` sends first too.
- Tested in `outbox.rs` (see S01).

### [2026-11-05] Done

Peter: checked in the app.
