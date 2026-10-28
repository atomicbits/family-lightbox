# Story S03: The background job: make what is missing, locally and on the NAS

**Feature:** `F012-make-missing-previews`
**Status:** Planned
**Depends on:** S02

## Goal

The app makes the previews of S02's plan in the background, from the original on this Mac or on
the NAS, and puts them in the local previews folder and on the NAS. Browsing stays smooth, and it
can be paused and continues after a restart.

## Acceptance criteria

- **When**: after the sync of feature 7 at start, and after a sync by hand; not during a sync or
  while the app is installing an update.
- **The original**: from the local working copy when it is there, else from the `Library` share.
  One that can't be read is skipped and stays *make* for next time.
- **Where to**: the local previews folder and, when the `Previews` share is connected, the same
  path there; both through a temporary name and a rename. Before making one, the job checks the
  NAS again: when another Mac made it in the meantime, it copies that one instead.
- **Load**: a few at a time (half the cores, at most 4), at a low priority, so the grid scrolls
  smoothly and the fan stays quiet.
- **The line in the window**: "Previews maken: 1 204 / 40 838" with a *Pauzeren* button, next to
  the sync line; *Hervatten* continues. Paused stays paused after a restart, until resumed.
- **Resume**: the record of S02 is written per preview, so a restart continues where it stopped.
- **Data contract**: section 1 (who writes `Previews`) and section 8 (transport) name the app.
- **Tests** (`cargo test`, temporary folders standing in for the shares): local original, original
  only on the NAS, no NAS, a preview that appears on the NAS during the run, pause and resume,
  a failure recorded and not retried.
- **By hand**: on Peter's Mac, the 40 838 missing previews of 2005–2021 are made and land on the
  NAS; browsing stays smooth meanwhile.

## How

- A module `src-tauri/src/previews_job.rs`, on the runner of feature 7 (`runner.rs`), with its own
  events for the line.
- Deliberately not done here: the grid picking up new previews and the list of failures (S04).

## Files affected

- `src-tauri/src/previews_job.rs` (new), `src-tauri/src/lib.rs`, `src/lib/` and a new line
  component, `docs/data-contract.md`.

## Progress log

### [2026-10-28] Written
