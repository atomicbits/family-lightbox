# Story S05: Running in the background: progress, cancel, when it runs, the line in the header

**Feature:** `F007-sync-from-nas`
**Status:** In progress: built and tested, runner checked on real data; the line, the dialog and Synchroniseren in the running app still to check
**Depends on:** S03, S04

## Goal

Syncing happens on its own and out of the way. When the app has connected to the NAS at start,
it fetches the manifests, counts the previews and copies what is new, in the background, while
Lynn browses. A small line next to the NAS indicator shows how far it is and can cancel it. The
very first full previews pass asks once, with the size and an estimate of the time.

## Acceptance criteria

- **One sync at a time**, run by the Rust side on its own thread. Its phases are:
  1. manifests (S03);
  2. counting the previews (S04);
  3. waiting for confirmation, only when S04 says so;
  4. copying the previews.
  It ends as done, cancelled or failed. Starting while a sync runs does nothing, and says that one
  is running.
- **Progress events**: the Rust side sends `sync-progress` with the phase, the current folder, and
  files and bytes done of the total, at most five times a second plus once at every phase change.
  At the end it sends `sync-done` with the outcome:
  - manifests copied and changed;
  - previews copied and bytes;
  - years skipped;
  - the error, or that it was cancelled.
  `sync_status` returns the last state, so a window that reloads picks it up.
- **Cancel**: `sync_cancel` sets the cancel flag from S02. The current file is abandoned without a
  temporary file left, and the state becomes "geannuleerd". The next sync goes on where this one
  stopped.
- **When it runs**:
  - **at start**, once the start sequence of feature 6/S05 is done and `Library` is connected: the
    manifests, then the previews as far as S04 allows. Previews need a `Previews` mount; without
    one, only the manifests run;
  - **with *Synchroniseren*** in the settings panel, the same sequence;
  - **not** on window focus, and not on a timer.
- **Confirmation**:
  - when the count says confirmation is needed, the sync stops in "wacht op bevestiging" with the
    summary;
  - the header line then shows *Previews ophalen…*, and clicking it opens a dialog: "29,1 GB
    previews ophalen? Dat duurt ongeveer 77 minuten." (the estimate at 6 MB/s), with *Ophalen* and
    *Later*;
  - *Ophalen* starts the copy with `confirmed = true`, using the counted plan while it is fresh
    (S04), and counting again otherwise;
  - *Later* leaves it for the next start.
- **Space**: when the count says it doesn't fit, the sync ends with the S04 message ("nodig …
  tekort"), shown in the header line until the next sync, and nothing is copied.
- **The line in the header**, left of the NAS indicator, only while there is something to say:
  - "manifesten 120/364";
  - "previews tellen… 2023/04";
  - "previews 1 234/13 076 · 1,2/5,0 GB", with a ✕ to cancel;
  - *Previews ophalen…*;
  - after a sync that copied something: "bijgewerkt: 2 manifesten, 40 previews" for a few
    seconds. When previews or manifests were copied, a short hint follows that the index needs
    `./media index` until feature 8 exists;
  - a failure or the space message in amber, until the next sync.
  A sync that copied nothing ends quietly.
- **Tests**:
  - `cargo test`:
    - the decision after counting (nothing to copy, needs confirmation, doesn't fit, copy) as a
      pure function;
    - the throttle (at most every 200 ms, but always on a phase change);
    - the runner refusing a second start;
    - a run of the sequence on temporary folders with a fake event sink: manifests, then counting,
      then waiting for confirmation, then the copy after confirmation, with the right events in the
      right order.
  - `pnpm test`: the header text for every phase, the GB and time estimate in the dialog, and the
    "bijgewerkt" line.
  - Existing tests still pass, typecheck, build and clippy clean.
  - **By hand** on Peter's Mac:
    - start the app: the line shows the manifests and then the count, then ends quietly (nothing to
      copy);
    - *Synchroniseren* and ✕ during the count;
    - the dialog, with `MEDIA_PREVIEWS` and `MEDIA_CONFIG` pointing at empty folders.
    Lynn's Mac comes with feature 4.

## How

- `src-tauri/src/runner.rs` (new):
  - `SyncRunner` with the state, the cancel flag and the counted plan, held in Tauri's managed
    state behind `Arc<Mutex<…>>`;
  - `start(kind, confirmed, sink)` spawns a thread that runs the sequence through `pull.rs`;
  - `Sink` is a trait with a Tauri implementation (`emit`) and a test implementation that records
    the events;
  - `decide_after_count(summary, confirmed)` and `Throttle` are pure.
- `lib.rs`:
  - the commands `sync_start(kind, confirmed)`, `sync_cancel` and `sync_status`;
  - the S03 and S04 commands (`sync_manifests`, `plan_previews`, `run_previews`) are removed in
    favour of the runner, which uses the same functions in `pull.rs`.
- Frontend:
  - `src/lib/sync.ts`: `useSync()` (listens to the events, starts, cancels, confirms) and the pure
    labels;
  - `src/components/SyncLine.tsx` and `src/components/ConfirmPreviews.tsx`;
  - `App.tsx` starts the sync after `nas.start()` when `Library` is mounted;
  - `Settings.tsx` gets *Synchroniseren*.
- Deliberately not done here:
  - the index (feature 8). Until then the hint points at `./media index`;
  - syncing on a timer or on focus;
  - limiting years on Lynn's Mac (plan, open point 2);
  - notifications outside the app.

## Files affected

- `src-tauri/src/runner.rs` (new), `src-tauri/src/lib.rs`.
- `src/lib/sync.ts` (new), `src/lib/sync.test.ts` (new), `src/components/SyncLine.tsx` (new),
  `src/components/ConfirmPreviews.tsx` (new), `src/App.tsx`, `src/components/Settings.tsx`.

## Progress log

### [2026-10-26] Written

### [2026-10-26] Built

- `src-tauri/src/runner.rs`:
  - `SyncRunner`, with its state, the cancel flag and the counted plan for the dialog, behind an
    `Arc<Mutex>`. It runs one sync at a time on its own thread;
  - `Sink`, with `EventSink` in `lib.rs` emitting `sync-progress` and `sync-done`;
  - `Throttle` (200 ms, or a phase change);
  - `decide_after_count`. A space problem wins over asking, because there is nothing to ask.
  Without a `Previews` mount, only the manifests run.
- `lib.rs`: `sync_start(kind, confirmed)`, `sync_cancel` and `sync_status`. The S03 and S04
  commands are gone, and `pull::sync_manifests` with them.
- Frontend:
  - `lib/sync.ts`: `useSync`, and the pure `syncLine`, `confirmText`, `estimate` (6 MB/s), `count`
    and `gb`. A "done" or "cancelled" line fades after 8 s; a failure and the confirmation stay;
  - `SyncLine.tsx` in the header, left of the NAS indicator;
  - `ConfirmPreviews.tsx`;
  - `useNas().start()` now resolves to the mounts it found. `App.tsx` starts `sync_start("all")`
    when `Library` is among them;
  - *Synchroniseren* in the settings panel, next to *Verbinden*;
  - the grid's keys are off while the dialog is open.
- `cargo test` 59 passed (5 new), `pnpm test` 37 passed (12 new), `tsc`, `pnpm build` and `cargo
  clippy --all-targets` clean.
- Different from the story: the dialog's estimate uses 6 MB/s, so 29,1 GB reads "ongeveer 81
  minuten". The story's example said 77, which came from the measured 6.3 MB/s.
- **On real data**, with a throwaway example on Peter's folders and mounts (removed afterwards):
  - a second start while running was refused ("er loopt al een synchronisatie");
  - manifests ran first (nothing to copy), and counting the previews began after about 22 s;
  - a cancel 10 s into the count stopped the thread within 4 ms, and the run ended as
    `Cancelled`;
  - 113 events in 33 s, which shows the throttle working;
  - no `.flb.partial` left in `~/Pictures/Previews`.
- Still to do by hand in the app: the line during a start, *Synchroniseren* and ✕, and the dialog
  with empty `MEDIA_PREVIEWS` and `MEDIA_CONFIG`.
