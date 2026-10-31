# Story S02: Write events to this Mac's own file on the NAS, and the summaries

**Feature:** `F003-lists`
**Status:** Done
**Depends on:** S01

## Goal

Every change to a list is one event appended to this Mac's own file on the NAS, by data
contract 5.2, and the summaries of the lists that changed are written a few seconds later.

## Acceptance criteria

- **Commands** for `create` (name, year), `add` and `remove` (list, photo hash, path), `rename`,
  `archive` and `unarchive`. Each writes one line to `Lists/events/<this device>.jsonl` on the
  share: `v`, `ts` (now, with the local offset), `dev`, `who` (this device's owner), `op`,
  `list` and the fields of the op; one `write` call, then `fsync`, ending in `\n`.
- **Refused** with a clear message when: the `Lists` share isn't connected; this Mac has no
  device name or isn't in `devices.json`; the list doesn't exist (except for `create`); the name
  is empty or longer than 100 characters.
- **A new list's id**: `<year>/<slug>`, the slug from the name: lowercase, accents removed,
  everything else than letters and digits becomes `-`, at most 60 characters; `-2`, `-3`, … when
  the id exists already in that year.
- **After writing**, the merged state is read again, so the answer is the state including the
  change.
- **Summaries** (data contract 5.4): a few seconds after the last change, for each list that
  changed, `Lists/<year>/<slug>.json` is written through a temporary name and a rename, with the
  photos sorted by path, then hash, and the path from the app's index where the hash is known,
  else the hint from the event.
- **Tests**: the line written (fields, one line, `\n`), appending to an existing file, a refusal
  per reason, the slug and its uniqueness, the summary.

## How

- In `lists.rs`: `Change`, `lines`, `refusal`, `append`, `slug`, `new_id`, `summary` and
  `write_summary`. `add` and `remove` take several photos, written in one `write` (for S05).
- Commands `list_create(name, year)`, `list_add(id, photos)`, `list_remove(id, photos)`,
  `list_rename`, `list_archive(id, archived)`, on a blocking thread; each answers with the list's
  id and the merged state read again from the share.
- The summaries: the changed list goes into a set; 3 seconds later the summaries of the set are
  written, with the path of each hash from the app's index.

## Files affected

- `src-tauri/src/lists.rs`, `src-tauri/src/lib.rs`, `docs/data-contract.md` (the slug rule).

## Progress log

### [2026-10-29] Written

### [2026-10-29] Built

- 5 tests in `lists.rs`: slugs (accents, punctuation, length, empty) and unique ids; the lines
  in the contract's shape, which read back as the same events; appending and reading back; every
  refusal (empty or too long name, unregistered device, unknown list, no photos), with nothing
  written; the summary (fields, the index path preferred over the hint, no temporary file left).
  115 Rust tests, clippy clean.
- Not tried against the real share on purpose: events are never removed, so a test list would
  stay in the history. The first real list is made from the app once S03 is in.

### [2026-10-31] fsync before the rename

Peter asked whether the files on `Lists` are written atomically. The event files are each
written by one device only, appended in one `write` with `fsync`, and a half line is ignored by
every reader. The summaries and `devices.json` went through a temporary file and a rename, but
without `fsync` before the rename, which data contract 1 asks for: after a crash at that moment
the new name could point at an empty file. Both now write the temporary file with
`write_synced` (write, `fsync`, then rename).
