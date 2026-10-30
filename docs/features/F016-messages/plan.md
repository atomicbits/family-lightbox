# Feature 16: Messages and a log

**Status:** In progress (plan accepted 2026-10-30; S01 done, S02 built)
**Depends on:** Feature 7 (the sync line), feature 12 (the previews line), feature 13 (the message with undo)
**Folder:** `F016-messages`

## Goal

The footer says what is going on now and shows short news that fades. Warnings and errors with
their technical details, such as a file the NAS didn't deliver in time, go to a list of messages
the user opens from the footer, and to a log file on disk for later. Only what asks for an action
stays in the footer until it is resolved.

## Background

- **Today** the footer's lines (sync, index, previews, update) and the lists' messages show
  their text there; a failed sync stays in amber until the next one, with the full path of the
  file that failed (Peter, 2026-10-30).
- **What asks for an action**: the confirmation to fetch the previews the first time, too little
  disk space, a `device.json` that is wrong, an update to install.

## Scope

### In scope

- **The messages**: every warning and error from the sync, the index, the previews, the lists
  and marks, the NAS and updates goes to one list in the app, with time, source and the full
  text; the newest first, at most 500.
- **The footer**: a small ⚠ with the number of messages not yet seen, when there are any; a click
  opens the list. The lines themselves show a short text and fade after a few seconds, unless
  they ask for an action.
- **The overlay**: the list on a translucent overlay, with *Wissen* and the path of the log file;
  also reachable from the settings.
- **The log file**: `~/.config/media/familylightbox.log`, one line per message with time, level,
  source and text, written by the Rust core; at 1 MB it is renamed to `.log.1` and started anew.
  Also the Rust core's own warnings that the UI never sees.
- **Data contract 7** lists the log file (not part of the contract).

### Out of scope

- Sending logs anywhere.
- Notifications outside the app (macOS notification center).

## Acceptance criteria

- The timed-out preview of 2026-10-30 gives "1 overgeslagen" in the sync line, which fades; the
  ⚠ shows 1; the overlay shows the path and the error; the log file has the line.
- A failed write of a star (F013) shows briefly in the footer and stays in the messages.
- The previews confirmation and too little disk space stay in the footer until resolved.
- `cargo test` covers the log file and its rotation; `pnpm test` covers the list, the count of
  unseen messages and which lines fade.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-the-log.md) | The log file and the messages in the Rust core, as events to the UI | Done | — |
| [S02](stories/S02-footer-and-overlay.md) | The ⚠ in the footer, the overlay, fading lines, what stays | In progress: built, to try in the app | S01 |
