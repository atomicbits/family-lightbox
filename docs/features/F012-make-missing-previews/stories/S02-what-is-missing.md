# Story S02: The plan: what is missing or outdated

**Feature:** `F012-make-missing-previews`
**Status:** Planned
**Depends on:** S01

## Goal

A function that says, for the whole library or one month, which previews the app has to make,
which it only has to copy from the NAS, and which are fine. It keeps the app's own record of
which hash each preview was made from, so a changed original gets a new preview.

## Acceptance criteria

- **Input**: the app's index (feature 8: every `photo` and `video` with its path, month and hash),
  the local previews folder, and the `Previews` share when it is connected.
- **For every item, one of**:
  - *fine*: preview and thumbnail are local, and the record has the same hash;
  - *adopt*: both are local but there is no record (made by the CLI or another Mac): the current
    hash is recorded, nothing is made (data contract 4);
  - *copy*: not local, but both are on the NAS: left to the sync of feature 7;
  - *make*: missing locally and on the NAS, or the record's hash differs from the manifest's;
  - *failed*: making it failed before with this same hash: not tried again until the hash
    changes or by hand.
- **The record** is the app's own SQLite file, `~/.config/media/familylightbox-previews.sqlite`
  (path, hash, made_at, error), next to its index; the CLI's `index.sqlite` is not read or
  written. Data contract 7 lists the new file.
- **Order**: the month on screen first, then the newest months first.
- **Fast enough** to run at every start: the full library (124 000 items) in about a second, with
  the NAS checked per folder, not per file.
- **Tests** (`cargo test`, temporary folders): every state above, a hash change, a NAS that isn't
  connected (no *copy*, only *make* for what isn't local), a year folder on the NAS that can't be
  read.

## How

- A module `src-tauri/src/previews_plan.rs`, independent of macOS.
- Deliberately not done here: making or copying anything (S03).

## Files affected

- `src-tauri/src/previews_plan.rs` (new), `src-tauri/src/lib.rs`, `docs/data-contract.md`.

## Progress log

### [2026-10-28] Written
