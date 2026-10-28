# Story S02: The plan: what is missing or outdated

**Feature:** `F012-make-missing-previews`
**Status:** Done
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
  the NAS checked per folder, not per file, and only for what isn't local.
- **A preview another Mac made again**: when the local preview is newer than the record (the
  sync keeps the NAS's time), it is adopted with the current hash instead of made once more.
- **Tests** (`cargo test`, temporary folders): every state above, a hash change, a NAS that isn't
  connected (no *copy*, only *make* for what isn't local), a year folder on the NAS that can't be
  read.

## How

- A module `src-tauri/src/previews_plan.rs`, independent of macOS: `Record` (open, `made`,
  `failed`, `clear_failures`), `plan(index, record, previews, nas, focus)` and `order(todo, focus)`
  for S03 to put another month first. Each folder is listed once, locally and on the NAS; a
  folder that can't be read counts as empty.
- An example program, `cargo run --example previews-plan`, prints the plan for real data.
- Deliberately not done here: making or copying anything (S03).

## Files affected

- `src-tauri/src/previews_plan.rs` (new), `src-tauri/src/lib.rs`, `docs/data-contract.md`.

## Progress log

### [2026-10-28] Written

### [2026-10-28] Built and checked

- 7 tests (`cargo test previews_plan`): every state, a changed hash, a failure tried again after
  clearing, a newer preview from another Mac adopted, no NAS, an unreadable year on the NAS, the
  order. 93 tests in all, clippy clean.
- On Peter's Mac, with a scratch record (the app's own file isn't written yet):
  - first run: 38 698 adopted, 85 874 to make, 0 to copy, in 0.6 s; the next run 38 698 fine in
    0.24 s;
  - with the `Previews` share connected: the same, in 4.2 s, because every folder with missing
    previews is listed on the NAS over SMB. S03 runs the plan after the sync, which has already
    copied what the NAS has, so it can leave the NAS out and check it per preview just before
    making it.
- **The gap is larger than the CLI said**: 85 874 photos and videos have no preview (2000–2021
  and `0000`), not 40 838. `media previews` only walks month folders (`YYYY/MM`) and misses the
  historical folders such as `2006/10-13.<name>`; the app works from the index, which has them all.
