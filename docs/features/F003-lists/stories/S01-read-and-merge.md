# Story S01: Read and merge the lists

**Feature:** `F003-lists`
**Status:** Done
**Depends on:** —

## Goal

The app knows every list and what is in it, merged from the event files exactly as data
contract 5.3 says, from the `Lists` share when it is connected and otherwise from the local copy,
which the sync of feature 7 keeps current. Nothing is written yet (S02) and nothing is shown yet
(S03); a command gives the state to the frontend.

## Acceptance criteria

- **Registered devices only**: `Lists/devices.json` gives the devices; only
  `events/<device>.jsonl` of those is read. A broken or missing `devices.json` gives no lists and
  a message, not a crash.
- **Lines** (data contract 5.2): a last line without `\n`, a line that isn't valid JSON, a line
  whose `dev` isn't the file's device, a `v` other than 1, an unknown `op`, a missing required
  field or a `ts` that isn't an ISO 8601 time with an offset: ignored and counted, the rest read.
- **Merging** (data contract 5.3): sorted by the moment of `ts` (offsets taken into account), then
  `dev`, then line order; `create` makes a list or renames an existing one; `add` puts a photo in
  with `who` and `ts` as added by and at, and creates the list implicitly with its slug as name;
  `remove` takes it out; `rename`, `archive` and `unarchive` change the list, also creating it
  implicitly when it has no `create` (the contract leaves this open; treated like `add`).
  Adding a photo already in the list and removing one that isn't change nothing.
- **Per list**: id, name, year (from the id), archived, created by and at (the first event),
  updated by (device) and at (the last event), and the photos with hash, path hint, added by and
  at.
- **Where from**: the `Lists` share when it is connected, else `~/Pictures/Lists` (`MEDIA_LISTS`
  overrides it); the state says which, so the UI can say that changes are not possible.
- **The sync of feature 7** copies `Lists` to the local copy after the manifests:
  `devices.json`, `events/*.jsonl` and the summaries `<year>/*.json`; nothing else. Without the
  `Lists` share it is skipped, not a failure.
- **Fast enough** to read at every look: thousands of events in milliseconds.
- **Tests** (`cargo test`): the contract's example events, implicit creates, duplicates, equal
  times ordered by device and line, times with different offsets, an unregistered device, a
  `dev` that doesn't match, broken and half-written lines, the sync filter.

## How

- A module `src-tauri/src/lists.rs` (independent of macOS): `Event`, `read(lists_dir)`, `merge`,
  `ListState`; a command `lists_state` returning the lists without their photos, and the photos
  of one list on request (for S04).
- `pull::lists_job` and a lists pass in the runner of feature 7, after the manifests (phase
  `lists`, "lijsten…" in the sync line). A failure in it doesn't stop the previews pass.
- `Paths.lists` (`MEDIA_LISTS`, default `~/Pictures/Lists`); the commands `lists_state` and
  `list_photos(id)`, on a blocking thread because the share may be slow.
- An example program, `cargo run --example lists -- <Lists folder>`, prints the merged state.
- Deliberately not done here: writing (S02), the UI (S03).

## Files affected

- `src-tauri/src/lists.rs` (new), `src-tauri/src/pull.rs`, `src-tauri/src/runner.rs`,
  `src-tauri/src/lib.rs`, `src-tauri/examples/lists.rs` (new), `src/lib/sync.ts` and its tests,
  `docs/data-contract.md` (section 8).

## Progress log

### [2026-10-29] Written

### [2026-10-29] Built and checked

- 8 tests in `lists.rs`: the contract's example (create, adds, Peter's remove, rename, archive),
  an add without create and duplicates, the order by moment across offsets then device then
  line, nine kinds of bad lines ignored and counted, only registered devices (a retired one
  included), a broken `devices.json`, the order of the lists, the sync filter. One runner test:
  the lists pass copies devices, events and summaries but not a temporary `devices.json.tmp-…`.
  110 Rust tests, clippy clean; 58 vitest tests.
- On the real `Lists` share: read from the NAS, no lists yet (only `devices.json`), no problems.
- The contract leaves open what `rename`, `archive` and `unarchive` do for a list without a
  `create`; they create it implicitly, as `add` does.
