# Story S02: Fetching

**Feature:** `F009-fetch-originals`
**Status:** Done (2026-11-05)
**Depends on:** S01

## Acceptance criteria

- Each file is copied from `Library` on the NAS to the working copy through a temporary name,
  content only, with its time; its hash checked against the manifest before the rename; a file
  whose hash differs is removed again and reported; never over an existing file.
- A journal in the app's state (`fetch/<id>.json`) keeps what is done, so a fetch that stops goes
  on by itself once the NAS is there; one fetch at a time, not during an import or a rename.
- Progress as `fetch-progress` (files and bytes), the end as `fetch-done`; the index rebuilt.
- **Tests**: `cargo test` for a whole fetch, a hash that differs, a conflict, resuming.

## Progress log

### [2026-11-05] Built

`fetch::Job` (in `fetch/<id>.json`) and `fetch::run`: per file the content and time copied to a
temporary name next to its place, the hash checked against the manifest (a sidecar has none),
then renamed, never over a file; a file here by now with the right hash counts as done; the
journal saved after every file. `lib.rs`: `fetch_plan`, `fetch_start` (one at a time; the shares
from the layout; the index rebuilt afterwards), `fetch_pending`; `fetch-progress` and
`fetch-done`. Tests: a whole fetch resumed after its first file, sidecars, no temporary file left;
a file whose content on the NAS differs from the manifest isn't kept. `cargo test` 213 passed,
`cargo clippy --all-targets` clean.

### [2026-11-05] Done

Peter: checked in the app.
