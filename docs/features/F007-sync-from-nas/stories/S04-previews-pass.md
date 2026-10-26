# Story S04: Previews pass with size count, 20 GB reserve and confirmation the first time

**Feature:** `F007-sync-from-nas`
**Status:** Planned
**Depends on:** S02

## Goal

The app brings every preview and thumbnail from the NAS's `Previews` to `~/Pictures/Previews`,
so that a Mac without the CLI can browse the whole library on local files. This is pass 2 of
`media pull` (data contract 8): all `*.jpg`, with `--update` semantics, and at least 20 GB left
free. A first sync is tens of GB, so the app counts first, refuses when it doesn't fit, and asks
once before it starts.

## Background

- `Previews` on the NAS holds 29 GB today, for 2022–2026 only: 4.7 to 7.3 GB per year, 77 396
  files. 2000–2021 come later (plan, open point 1), to roughly 60–70 GB in all.
- Measured in S02: listing over SMB takes 20 s to more than 2 minutes for one year, and copying
  runs at about 6.6 MB/s. A first full sync of 29 GB is therefore roughly 75 minutes, and a run
  with nothing to copy is a few minutes of listing.
- Peter's Mac made these previews itself, so there a run copies next to nothing. Lynn's Mac starts
  from nothing.
- Until feature 8, the grid shows a preview based on the `preview` column of the CLI's index. On
  a Mac without the CLI, copied previews only show once feature 8 builds the index. That is
  expected, and outside this story.

## Acceptance criteria

- **What and where**:
  - from the chosen `Previews` mount (feature 6) to `MEDIA_PREVIEWS` (default
    `~/Pictures/Previews`);
  - every file ending in `.jpg`, which covers both the preview and the `.t.jpg` thumbnail, at any
    depth;
  - the S02 rules apply: missing or newer on the NAS is copied, and a newer local preview stays;
  - year folders the account may not read are skipped and listed (Lynn: before [private]);
  - nothing is deleted locally, and nothing is written on the NAS.
  No `Previews` mount means the pass doesn't run, and says why.
- **Two steps: count, then copy**:
  1. `plan_previews` walks the tree and returns a summary:
     - the number of files and bytes to copy;
     - the files kept and the years skipped;
     - the free space on the local volume, and whether it fits;
     - whether confirmation is needed.
     The plan itself is kept in the app, so the minutes of listing aren't repeated. It stays
     valid for 30 minutes.
  2. `run_previews(confirmed)` copies that plan. An older plan, or no plan, is refused with
     "tel opnieuw".
- **Space**:
  - it fits when `bytes to copy + 20 GB ≤ free space` on the volume of the local root, measured
    natively with `statvfs`;
  - free space is checked again just before copying;
  - when it doesn't fit, nothing is copied, and the message gives the three numbers, for example
    "nodig 29,1 GB, vrij 35,2 GB, reserve 20 GB: 13,9 GB tekort";
  - the count is conservative. A file that replaces an older one is counted in full, because its
    temporary copy needs that much space for a moment.
- **Confirmation the first time**:
  - as long as no full previews pass has been confirmed on this Mac, `run_previews(false)` is
    refused with "bevestiging nodig". The summary shows the size, so the UI (S05) can ask
    "29,1 GB previews ophalen (ongeveer 75 min)?";
  - `run_previews(true)` records the confirmation;
  - later runs don't ask again. They still count, and they still check the space;
  - the confirmation is kept in `~/.config/media/familylightbox-state.json` as
    `previews_confirmed_at`. This is a new file for the app's own state, separate from
    `familylightbox.json`, because saving the settings writes that whole file. It is written
    atomically, and it is not part of the contract. Data contract 7 lists it next to
    `familylightbox.json`;
  - a summary with nothing to copy needs no confirmation.
- **Result**: files and bytes copied, kept, skipped, cancelled or error, as in S02. The progress
  callback and cancelling come from S02. The UI for both is S05.
- **Commands**: `plan_previews` and `run_previews(confirmed)`, both async on a blocking thread,
  with the plan held in Tauri's managed state behind a `Mutex`. The dialog and the progress line
  are S05.
- **Tests** (`cargo test`, temporary folders):
  - the filter: `.jpg` and `.t.jpg` count; `.JPG` from a source, `.png` and `.json` don't;
  - space: fits, exactly at the limit, short by 1 byte, and the message with its numbers. Free
    space is given to the function, so the test doesn't depend on this Mac's disk;
  - confirmation: refused without, and recorded with; a second plan needs none; nothing to copy
    needs none. The state file is written atomically, and a broken one counts as "not confirmed"
    without being overwritten until the next confirmation;
  - the plan: older than 30 minutes is refused, and without a plan copying is refused;
  - an unreadable year is skipped;
  - no `Previews` mount gives a clear message.
  - Existing tests still pass, and clippy is clean.
- **By hand**, in the progress log:
  - on Peter's real `~/Pictures/Previews`: the plan, and then a run, which should copy close to
    nothing;
  - into an empty folder, limited to one year (2023, 4.7 GB): plan, refusal without
    confirmation, then the run with confirmation, the time it took, and a second plan with
    nothing to copy;
  - the space refusal: the same empty folder with the free space given as a test value, since
    this Mac has 175 GB free.

## How

- `pull.rs`:
  - `is_preview(rel)` and `previews_job(settings, mounts, local)`;
  - `free_bytes(path)` (`libc::statvfs` on macOS; `None` elsewhere, and then the pass refuses);
  - `space_check(copy_bytes, free) -> Result<(), String>`;
  - `PreviewsSummary`;
  - `load_state` and `save_state` for `familylightbox-state.json` (in `config.rs`, next to the
    settings);
  - `run_previews`.
- `lib.rs`:
  - managed state `PreviewsPlan(Mutex<Option<(Instant, Job, Plan)>>)`;
  - the commands `plan_previews` and `run_previews`.
- The data contract, section 7: `familylightbox-state.json` added (the app's own state, not
  contract).
- Deliberately not done here:
  - the confirmation dialog, the progress line, cancelling from the UI, and when it runs (S05);
  - making the missing previews for 2000–2021 (plan, open point 1);
  - limiting the years on Lynn's Mac (plan, open point 2), which the job's `only` makes possible
    later;
  - the index (feature 8).

## Files affected

- `src-tauri/src/pull.rs`, `src-tauri/src/config.rs` (state file), `src-tauri/src/lib.rs`.
- `docs/data-contract.md` section 7 and its change log.

## Progress log

### [2026-10-26] Written
