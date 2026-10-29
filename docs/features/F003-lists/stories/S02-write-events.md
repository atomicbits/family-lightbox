# Story S02: Write events to this Mac's own file on the NAS, and the summaries

**Feature:** `F003-lists`
**Status:** Planned
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

- In `lists.rs`: `append(lists_dir, device, owner, op)`, `slug(name)`, `summary(list, index)`.
- Commands `list_create`, `list_add`, `list_remove`, `list_rename`, `list_archive`.

## Files affected

- `src-tauri/src/lists.rs`, `src-tauri/src/lib.rs`, `docs/data-contract.md` (the slug rule).

## Progress log

### [2026-10-29] Written
