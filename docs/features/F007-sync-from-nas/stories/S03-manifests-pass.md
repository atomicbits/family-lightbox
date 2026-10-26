# Story S03: Manifests pass from `Library`

**Feature:** `F007-sync-from-nas`
**Status:** Done
**Depends on:** S02

## Goal

The app brings every manifest from the NAS's `Library` to the local working copy, so that the
Mac knows the whole library, including years whose originals aren't local. This is pass 1 of
`media pull` (data contract 8): all `<year>/<subfolder>/.manifest.json`, with `--update`
semantics. Feature 8 builds the index from these files, and until then the CLI's `media index`
reads them.

## Background

- The manifests are at `<year>/<subfolder>/.manifest.json` (data contract 3.1, 3.5), one per year
  subfolder: ordinary months (`2026/07`), historical folders (`2012/07.Trouw`), and folders
  without a month (`2012/scans Peter`, `0000/zonder-datum`). The NAS has **364** of them today.
- Peter's working copy already has **363**, and in 2000–2021 nothing but manifests: pass 1 of
  `media pull` has run there before. The one missing is most likely `0000/zonder-datum`, made on
  the NAS on 2026-10-23. A run on Peter's Mac is therefore the same as what the CLI already did,
  and the CLI is built for it (years with only manifests are normal, data contract 8).
- Walking all of `Library` over SMB to find 364 files would mean listing about 125 000 originals.
  The manifests sit exactly two levels deep, so the walk can stop there.

## Acceptance criteria

- **Only manifests, only two levels deep**: the job's filter takes exactly
  `<year>/<subfolder>/.manifest.json`, where `<year>` is four digits (`0000` included). The walk
  descends into the year folders and their subfolders, and no deeper. To do that, the engine's
  `Job` (S02) gets an optional `max_depth`: the number of folder levels below the root that are
  still walked. Here that is 2. The previews pass (S04) leaves it at `None`. A test in `sync.rs`
  covers it.
- **Source and target**: the remote root is the chosen `Library` mount from feature 6 (S02 of that
  feature); the local root is the working copy (`MEDIA_LIBRARY`, by default
  `~/Pictures/LibraryLocal`). No `Library` mount, or a `Library` that isn't readable, means the
  pass doesn't run and says why.
- **Rules from S02**: missing or newer on the NAS is copied; a newer local manifest stays. Year
  folders the account may not read are skipped and listed (Lynn: before [private]). Nothing is
  deleted locally and nothing is written on the NAS.
- **Check after copying**: every manifest copied in this run is parsed. A file that isn't valid
  JSON, or has no `files` list, is listed as "ongeldig" in the result. It stays in place, because
  the contract never deletes, and the index (feature 8) skips it. A valid copy goes in the
  result's list of changed folders, which feature 8 uses to update only those folders.
- **Function and command**:
  - `fn sync_manifests(config, cancel, progress) -> ManifestsOutcome` (copied, kept, skipped
    years, invalid, changed folders, cancelled or error);
  - an async command `sync_manifests` that runs it on a blocking thread.
  The button, the progress line and when it runs automatically are S05.
- **Tests** (`cargo test`, temporary folders):
  - the filter: `2026/07/.manifest.json`, `0000/zonder-datum/.manifest.json` and
    `2012/scans Peter/.manifest.json` count; `.manifest.json` at the root, `2026/.manifest.json`,
    `2026/07/EOSR/.manifest.json`, `2026/07/x.json` and `26/07/.manifest.json` don't;
  - `max_depth` 2: a manifest three levels deep isn't seen, and a deeper folder isn't even opened
    (checked with an unreadable folder at level 3 that doesn't end up in `skipped`);
  - a copied manifest that is broken is listed as invalid and left in place; a valid one appears
    in the changed folders;
  - an unreadable year is skipped;
  - no `Library` mount gives a clear message and no run.
  - Existing tests still pass, and clippy is clean.
- **By hand**, in the progress log:
  - with `MEDIA_LIBRARY` pointing at an empty folder: all 364 manifests come over, and a second
    run copies nothing;
  - on Peter's real working copy: first the plan only (what would be copied), shown to Peter,
    and only after his go the real run; afterwards `./media status` and `./media push -n` show
    nothing unexpected.

## How

- `sync.rs`: `max_depth: Option<usize>` on `Job`, checked in `walk` before descending.
- New module `src-tauri/src/pull.rs` (the app's counterpart of `media pull`):
  - `fn is_manifest(rel) -> bool`;
  - `fn manifests_job(settings, mounts, paths) -> Result<Job, String>`;
  - `fn sync_manifests(...)`, which runs plan and execute from S02, then checks the copied files.
  S04 adds the previews pass to the same module.
- `lib.rs`: command `sync_manifests`; `Paths` already has `library`.
- Deliberately not done here: building or updating the index (feature 8), originals, the UI
  (S05), and the space check (manifests are a few MB in total).

## Files affected

- `src-tauri/src/sync.rs` (`max_depth`, a test), `src-tauri/src/pull.rs` (new),
  `src-tauri/src/lib.rs`.

## Progress log

### [2026-10-26] Written

### [2026-10-26] Built

- `sync.rs`: `Job.max_depth`. A folder at level n below the root is walked only when
  n ≤ `max_depth`. A test shows that an unreadable folder at level 3 is never opened.
  `Job` derives `Debug`.
- `src-tauri/src/pull.rs`: `is_manifest`, `manifests_job` (the chosen `Library` mount to the
  working copy, depth 2), `run_manifests` (plan, copy, then parse every manifest that actually
  arrived: valid ones go to `changed` as their folder, others to `invalid`, left in place), and
  `sync_manifests` for the app. `lib.rs`: async command `sync_manifests`.
- `cargo test` 45 passed (5 new), `cargo clippy --all-targets` clean.
- **By hand**, with a throwaway example (removed afterwards):
  - into an empty folder: 364 manifests in 69 s over AFP, with nothing else, nothing skipped and
    nothing invalid; a second run copied nothing (364 kept);
  - **plan only** on Peter's real working copy: 2 to copy, 362 kept, none newer locally:
    - `0000/zonder-datum/.manifest.json` is missing locally (made on the NAS on 2026-10-23);
    - `2016/12/.manifest.json` is newer on the NAS (4 files added there from the old mirror on
      2026-10-23).
  - The real run, followed by `./media status` and `./media push -n`, waits for Peter's go.

### [2026-10-26] Real run on Peter's working copy

- With Peter's go:
  - copied 2, kept 362, nothing skipped or invalid, changed `0000/zonder-datum` and `2016/12`;
  - both files are byte-identical to the NAS (`md5`), with the NAS modification time;
  - `./media status` is unchanged.
- `./media push -n` then listed one month more: 2016/12. That turned up a difference from
  `rsync -a`. Writing into a folder changes the folder's own modification time, and the CLI's
  `library_months` takes folder times into account. The engine now gives folders their time back
  (see S02): an existing folder keeps its old time, and a folder it creates gets the NAS folder's
  time. There is a new test in `sync.rs`. The three folders touched in this run (`2016/12`,
  `0000`, `0000/zonder-datum`) were set to their NAS time with `touch -r`, because their old local
  times were lost.
- After that `push -n` still lists 2016/12, and that is expected. The CLI's last push was
  2026-10-17, and the manifest carries its NAS time of 2026-10-23, which is later. `media pull`
  with `rsync -a` would have done exactly the same. A real `push` of that month would rebuild the
  manifest from the NAS's own entries (`merge_manifest` keeps every remote entry whose file is
  there with the right size) plus 0 local files. The content stays the same; only the header
  (`generated_by`, `generated_at`) changes.
