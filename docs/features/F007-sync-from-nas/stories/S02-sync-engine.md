# Story S02: Sync engine

**Feature:** `F007-sync-from-nas`
**Status:** Done
**Depends on:** —

## Goal

One piece of Rust that brings a folder tree on the NAS mount over to a local folder, the way
`rsync --update` does in `media pull` (data contract 8): it copies what is missing locally or
newer on the NAS, and leaves everything else alone. S03 (manifests) and S04 (previews) are each
this engine with a different filter. S05 runs it in the background. The engine never deletes a
local file other than its own temporary files, and never writes to the NAS.

## Acceptance criteria

- **Input**: a remote root (a folder on a mount from feature 6, for example `/Volumes/Previews`),
  a local root (for example `~/Pictures/Previews`), a filter on the relative path (for example
  "ends in `.jpg`", or "is `<year>/<subfolder>/.manifest.json`"), and optionally the top-level
  folders to limit it to.
- **Planning** walks the remote tree and decides per file that passes the filter:
  - **copy, missing**: there is no local file at the same relative path;
  - **copy, newer**: the remote modification time is later than the local one, compared in whole
    seconds because SMB, AFP and APFS round differently;
  - **copy, changed**: the times are equal but the sizes differ;
  - **keep, same**: same time and size;
  - **keep, local newer**: the local file is newer. A newer local preview stays (data
    contract 8).
  The plan has the totals (files and bytes to copy, files kept) that S04 needs for its space
  check before anything is copied.
- **Walking**:
  - names excluded by the data contract (section 1) are skipped as files and as folders:
    `@eaDir`, `#recycle`, `#snapshot`, `.DS_Store`, `._*`, `Thumbs.db`, `.TemporaryItems`,
    `.Spotlight-V100`, `.fseventsd`, `*.tmp`, `*.partial`;
  - symbolic links are not followed;
  - a folder the account may not read (permission denied) is skipped and listed in the result
    as skipped. That is how Lynn's years before [private] behave. Any other read error, such as the
    NAS going away, ends the run with that error instead of being treated as "nothing there".
- **Copying** (atomic):
  - the engine writes to `.<name>.flb.partial` in the target folder, creating the folders on the
    way. The name ends in `.partial`, so the CLI ignores it, and it starts with a dot, so the
    Finder hides it;
  - it copies in blocks of 1 MiB, then `fsync`, then sets the modification time to the remote
    one, then renames over the target;
  - a failed copy removes its temporary file, and the target stays as it was;
  - the remote file is only ever opened for reading.
- **Cancelling**: a shared flag is checked between files and between blocks. On cancel, the
  current temporary file is removed and the run ends with "cancelled". What was already copied
  stays, so the next run only does the rest.
- **Leftovers**: temporary files `.*.flb.partial` older than an hour, found in a local folder
  the engine visits, are removed (from a crash or a power cut).
- **Progress**: the engine calls a callback with the phase (planning or copying), the current
  folder, and files and bytes done of the total: per file while planning, and per block while
  copying. Throttling the callback for the UI is S05's job.
- **Result**: files copied (per reason) and bytes, files kept, folders skipped, and whether it
  was cancelled. A run that finds nothing to do copies nothing, and the local tree stays
  byte-identical, modification times included.
- **Tests** (`cargo test`, temporary folders for "remote" and "local"):
  - each decision above, including equal time with a different size and a local newer file;
  - the exclusions as files and as folders, and a symlink that isn't followed;
  - a remote subfolder with mode `000`, skipped and listed, while the rest is copied;
  - a copy keeps the remote modification time, and leaves no temporary file;
  - cancelling during the second file of three: the first file is there, no temporary file is
    left, and a second run copies the other two;
  - a stale temporary file older than an hour is removed, and a fresh one is left alone;
  - the remote tree has the same content and times before and after, also when it is read-only
    (mode `555`);
  - the filter and the limit to top-level folders;
  - a read error other than permission denied (the remote root removed during planning) ends
    with an error.
  - Existing tests still pass, and `cargo clippy --all-targets` is clean.

## How

- New module `src-tauri/src/sync.rs`:
  - `struct Job { remote: PathBuf, local: PathBuf, filter: fn(&str) -> bool, only: Option<Vec<String>> }`;
  - `fn plan(job, progress, cancel) -> Result<Plan, String>`, where the `Plan` holds the entries
    with relative path, decision, size and remote time, plus the totals and the skipped folders;
  - `fn execute(job, &plan, progress, cancel) -> Result<Outcome, String>`;
  - `fn excluded(name) -> bool`, following data contract section 1.
  - It uses `std::fs` only. Modification times are set with `File::set_modified` from the
    standard library, so there is no new dependency.
- Deliberately not done here:
  - what to sync and where it goes (S03, S04);
  - the space check and the confirmation (S04);
  - threads, events to the UI, and when it runs (S05);
  - deleting local files that no longer exist on the NAS (never: data contract 1);
  - checking hashes. Previews and manifests have no hash of their own in the contract, and time
    plus size is what `rsync --update` uses too.

## Files affected

- `src-tauri/src/sync.rs` (new), `src-tauri/src/lib.rs` (`mod sync`).

## Progress log

### [2026-10-26] Written

### [2026-10-26] Built

- `src-tauri/src/sync.rs`: `excluded`, `decide`, `plan`, `execute`, with an atomic copy through
  `.<name>.flb.partial` (blocks of 1 MiB, the remote time set, `fsync`, rename), the cancel flag
  checked between files and blocks, and stale temporary files older than an hour cleared in the
  local folders it visits. Only `std`, no new dependency.
- One finding while testing: a top-level folder that disappeared between listing and reading
  was skipped silently. That is now an error. A top-level folder requested with `only` that
  doesn't exist is still skipped, because that is a year without files.
- `cargo test` 40 passed (9 new; the permission and symlink tests only on Unix), `cargo clippy
  --all-targets` clean.
- **On real data**, with a throwaway example (removed afterwards):
  - plan of `/Volumes/Previews/2026` (SMB) against Peter's `~/Pictures/Previews`: 13 702 files,
    all "keep, same", nothing to copy. It took 21 s the first time and 143 s the second (the
    speed of listing over SMB varies);
  - `/Volumes/Previews/2022/02` into an empty folder: 1 072 files, 361 MB in 54 s (about 6.6 MB/s),
    with no temporary files left. Size and modification time are equal to the NAS, and the content
    is byte-identical (`cmp`). A second plan: nothing to copy, 1 072 kept;
  - at that speed the full 29 GB of previews take about 75 minutes the first time.
- Found: on the NAS the preview folders 2000–2007 are empty. Only 2022–2026 has previews. The
  F007 plan is corrected (open point 1 is now 2000–2021).
- The code was committed in `d7bcfd3`. These notes followed in a separate commit, because the
  script that updated them stopped halfway.

### [2026-10-26] Folder times (found in S03)

- Copying into a folder changed that folder's modification time, which `rsync -a` doesn't do,
  and the CLI reads folder times (`library_months`). `execute` now notes each folder's local time
  before its first copy and restores it at the end, also after a cancel or an error. A folder the
  engine created gets the NAS folder's time. Test `folder_times_stay_as_they_were_or_follow_the_nas`.
