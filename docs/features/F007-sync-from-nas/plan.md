# Feature 7: Sync manifests and previews from the NAS

**Status:** In progress (S01–S04 done, S05 built; in-app check owed)
**Depends on:** Feature 6 (a connected, readable NAS)
**Folder:** `F007-sync-from-nas`

## Goal

A Mac with only FamilyLightbox installed gets everything it needs to browse the whole library
from the NAS on its own: every manifest and every preview, kept up to date as Peter imports. This
replaces the first two passes of `media pull` (data contract section 8) for Lynn. Together with
feature 8 (the native index) it means Lynn's Mac needs no CLI at all. With this feature the data
contract moves into this repo, in English, because the app becomes the implementation everything
moves towards (D13).

## Background

- **What `media pull` does today** (data contract 8, "NAS → laptop, bulk"), in three passes per
  readable year:
  1. all `.manifest.json` with `--update`, also for years without originals;
  2. all `*.jpg` from `Previews` with `--update` (a newer local preview stays);
  3. originals of the last 18 months with `--ignore-existing`, without camera video.
  Before it starts there is a dry run with `--stats`, and at least 20 GB must stay free. Year
  folders the user may not read are skipped without a message. Afterwards it runs `media index`.
- **Where things land** (data contract 7): manifests in the working copy
  `~/Pictures/LibraryLocal/<year>/<month>/.manifest.json`, previews in `~/Pictures/Previews/` on
  the same paths as on the NAS. These are the places the app already reads (`MEDIA_LIBRARY`,
  `MEDIA_PREVIEWS`).
- **Sizes today** (measured 2026-10-26):
  - `Library` has 364 manifests;
  - `Previews` on the NAS is 29 GB with 77 396 files (a preview and a thumbnail for about
    38 700 items), for 2022–2026 only. The folders 2000–2007 exist but are empty (found in
    S02; this plan first said 2000–2007 had previews);
  - the index knows 123 801 items, so **the years 2000–2021 have no previews yet** (see open
    points);
  - Peter's own `~/Pictures/Previews` is 28 GB.
- **What is read over the network**: everything comes from the mounts of feature 6, over SMB or
  AFP. That means listing folders and copying files on a mounted share, with no rsync.
- **Lynn's account** can't read some year folders (ACL on the NAS). Those are skipped, just as
  `pull` skips them.

## Scope

### In scope (macOS first; nothing here is macOS-specific except the mounts from feature 6)

- **Data contract into this repo**: `docs/data-contract.md`, an English translation of
  `~/Pictures/LibraryLocal/docs/datacontract.md` with its change log. The Dutch file becomes a
  short pointer. From then on, contract changes are made here; the CLI follows.
- **A sync engine** for one share: it walks the remote tree and the local tree for the folders
  in scope and decides per file what to do:
  - copy when missing locally;
  - copy when the remote file is newer (this is the `--update` semantics);
  - otherwise leave it.
  It copies to a temporary file next to the target, keeps the remote modification time, and
  renames into place. It never deletes locally and never writes on the NAS. It skips DSM's
  entries and `.DS_Store`, and skips folders it can't read without an error.
- **Manifests pass**: every `<year>/<subfolder>/.manifest.json` from `Library` to the working
  copy, including years without local originals.
- **Previews pass**: every `*.jpg` from `Previews`. A first sync can be tens of GB, so:
  - before copying, the app counts what is to be copied and how big it is;
  - it refuses when less than 20 GB would remain free;
  - the first time, it asks for confirmation, showing the size.
- **Running it**: in the background, with progress (current folder, files and bytes done of the
  total), cancellable, and resumable by simply running again. It runs:
  - after the start check of feature 6, when the NAS is connected (manifests first; previews only
    once confirmed);
  - with *Synchroniseren* in the settings panel.
  A small line next to the NAS indicator shows that a sync is running, and how far it is.
- **After a sync**, the app tells the next step that manifests changed: feature 8 rebuilds the
  index. Until feature 8 exists, the app keeps reading the CLI's `index.sqlite`, and on Peter's
  Mac `media index` stays the way to refresh it.

### Out of scope

- **Originals.** Pass 3 of `pull` is not done here. Lynn opens originals from the NAS
  (feature 9). Keeping the last 18 months of originals locally is part of Peter's workflow in the
  app (feature 10), since his laptop already has them.
- **Making previews.** This feature only copies what is on the NAS. Making the missing previews
  for 2000–2021 is a separate decision (open points).
- **Building the index** (feature 8), **lists** (feature 3), **Windows**.
- **Pushing anything to the NAS.**

## Acceptance criteria

- `docs/data-contract.md` exists in English with the full content and change log of the Dutch
  contract, and the Dutch file points to it. `CLAUDE.md` and the features README point to the new
  file.
- On a Mac with an empty `~/Pictures/LibraryLocal` and `~/Pictures/Previews` (tested on Peter's
  Mac with `MEDIA_LIBRARY` and `MEDIA_PREVIEWS` pointing at empty folders), a sync brings over
  all 364 manifests and all previews that are on the NAS. A second sync copies nothing.
- On Peter's Mac with his real folders, a sync copies only what is missing or newer on the NAS,
  and never overwrites a newer local file. Nothing is deleted locally or written on the NAS.
- A sync that is cancelled or interrupted (for example the NAS goes away) leaves no half-written
  files, and the next run continues where it stopped.
- Too little space: the previews pass refuses with the numbers (needed, free, the 20 GB
  reserve), and copies nothing.
- With an account that can't read some years, those years are skipped without an error, and the
  rest syncs.
- The window stays responsive during a sync of tens of thousands of files, and the progress line
  updates.
- `cargo test` covers the planning (missing, newer, older, same), the exclusions, unreadable
  folders, the atomic copy that keeps the modification time, cancellation and the space check, on
  temporary folders. `pnpm test` covers the progress text. Existing tests pass.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-move-data-contract.md) | Move the data contract into this repo, in English | Done | — |
| [S02](stories/S02-sync-engine.md) | Sync engine: plan per folder, atomic copy with the remote time, exclusions, unreadable folders | Done | — |
| [S03](stories/S03-manifests-pass.md) | Manifests pass from `Library` | Done | S02 |
| [S04](stories/S04-previews-pass.md) | Previews pass with size count, 20 GB reserve and confirmation the first time | Done | S02 |
| [S05](stories/S05-background-sync.md) | Running in the background: progress, cancel, when it runs, the line in the header | In progress: built, in-app check owed | S03, S04 |
| [S06](stories/S06-one-file-failing.md) | One file that fails is retried once, then skipped; the sync goes on | Planned | S05 |

## Open points

Peter approved the plan on 2026-10-26, with the proposals for points 1 and 3. Point 2 is checked
on Lynn's Mac.

1. **Previews for 2000–2021 don't exist yet.** About 85 000 items (the 123 801 in the index
   minus the ~38 700 with a preview) would show as "geen preview" on Lynn's Mac, and on Peter's
   too once his local copies are gone. The originals of those years are only on the NAS.
   Options:
   - (a) Peter runs `media previews` on his Mac against the NAS mount (`MEDIA_LIBRARY=/Volumes/Library`),
     then `previews push`. That works today, and takes hours over the network;
   - (b) the app makes previews natively (part of feature 10), later;
   - (c) the NAS makes them (slow Celeron, D4 rejected this).
   My proposal is (a), as a one-off with the CLI in LibraryLocal, outside this feature. Lynn's
   account can't see every year, so for her it matters for fewer years than for Peter, who sees
   2000–2021.
2. **Disk space on Lynn's Mac**: the previews need 29 GB now, and roughly 60–70 GB once 2000–2021
   has previews. Is that fine on her laptop, or should the previews pass be limited (for example
   to the years she opens)? The 20 GB reserve protects her either way.
3. **Whether to sync at every start**: the proposal is manifests at every start (small and fast)
   and previews at every start once the first full sync was confirmed. The alternative is
   previews only on demand.
