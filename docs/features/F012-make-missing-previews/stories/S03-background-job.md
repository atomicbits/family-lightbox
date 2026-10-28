# Story S03: The background job: make what is missing, locally and on the NAS

**Feature:** `F012-make-missing-previews`
**Status:** In progress: built and tested; the run on real data still to check
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
- **The line in the window**: "Previews maken: 1 204 / 85 874" with a *Pauzeren* button, next to
  the sync line; *Hervatten* continues. Paused stays paused after a restart, until resumed.
- **Resume**: the record of S02 is written per preview, so a restart continues where it stopped.
- **Data contract**: section 1 (who writes `Previews`) and section 8 (transport) name the app.
- **Tests** (`cargo test`, temporary folders standing in for the shares): local original, original
  only on the NAS, no NAS, a preview that appears on the NAS during the run, pause and resume,
  a failure recorded and not retried.
- **By hand**: on Peter's Mac, the 85 874 missing previews of 2000–2021 are made and land on the
  NAS; browsing stays smooth meanwhile.

## How

- A module `src-tauri/src/previews_job.rs` with its own runner (`MakeRunner`), next to the sync
  runner of feature 7: `start`, `stop(paused)`, `set_paused`, `focus(month)` and `status`; the
  events `previews-progress` and `previews-done`.
- **When it starts**: every time an index build finishes (at start, after a sync that copied
  something, on focus, by hand), and after a sync that copied nothing. It doesn't start while a
  sync or an index build runs, while an update installs, or when paused. Starting a sync stops it;
  it starts again after the sync.
- **The plan** is S02's, without the NAS (the sync has just copied what the NAS has); each
  preview looks on the NAS once more just before it is made. Without the `Library` share, only
  previews whose original is on this Mac are queued, so the line doesn't count what can't be made.
- **The workers** take the next preview from a shared queue, so *focus* can put another month
  first at any moment. Their threads run at macOS's *utility* priority.
- **The grid**: while it runs, the index is rebuilt at most every 2 minutes when previews were
  made, and once at the end, so they show while the rest is made. S04 replaced this with a flag
  per preview.
- **Pause** is kept in `familylightbox-state.json` (`previews_paused`).
- **A failure** is recorded only when the original is still readable afterwards; when the NAS
  went away while reading, it counts as skipped and is tried again next time.
- `src/lib/previews.ts` and `src/components/PreviewsLine.tsx` for the line.
- Deliberately not done here: the grid picking up new previews and the list of failures (S04).

## Files affected

- `src-tauri/src/previews_job.rs` (new), `src-tauri/src/lib.rs`, `src-tauri/src/config.rs`,
  `src/lib/previews.ts` and `previews.test.ts` (new), `src/components/PreviewsLine.tsx` (new),
  `src/App.tsx`, `docs/data-contract.md`.

## Progress log

### [2026-10-28] Written

### [2026-10-28] Built

- 6 tests (`cargo test previews_job`), each on temporary folders standing in for the shares, with
  the test video as the original: from this Mac and from the NAS to both (no `.tmp` left behind),
  without the NAS only what is local, a preview another Mac put on the NAS copied instead of made,
  a failure recorded and not retried, pause mid-run and resume with the rest, focus. 99 Rust tests
  in all, clippy clean; 54 vitest tests with 3 new ones for the line; `pnpm build` clean.
- Speed over SMB, measured with the example program and 4 at a time: 40 photos of 2012 from the
  `Library` share in 6 s, about 6.7 a second, so the 85 874 missing previews take about 3.5 to
  4 hours.
- The job is macOS only, like S01; the app's builder registers it unconditionally, which has to
  be gated once the app builds for Windows.
