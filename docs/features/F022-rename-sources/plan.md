# Feature 22: Renaming sources

**Status:** In progress (plan accepted 2026-11-04)
**Depends on:** Feature 8 (the index), feature 18 (sources and their layout), feature 19 (one share),
feature 2/S05 (sources in the filter)
**Folder:** `F022-rename-sources`

## Goal

A source in the library can get another name, or be merged into another source, so that the
inconsistent names of the past are straightened out: a phone's old source next to its current one
(`iPhone-fotos` until 2019, `iPhone-photos` since), two names for one camera (`Hero7`, `GoProH7`),
leftovers of one or two files (`Test`, `100GOPRO`). Afterwards the source filter, the import's
proposals and the folders on the NAS show one name per way photos come in.

## Background

- A source is the folder below the month (data contract 3.1): `<year>/<month>/<source>/…`, the
  same in `Library` and `Previews`, and the `source` field of every row in that month's manifest.
  Lists, stars, hearts, hiding and stacks refer to photos by hash, so they don't change; only the
  path hints in the list summaries do, and those are rewritten at the next merge.
- Since feature 19 `Library` and `Previews` are folders of one share, on one file-system volume:
  moving a file between sources is a rename, not a copy. Renaming the largest candidate
  (`iPhone-fotos`, 4 597 files in 109 months) is minutes of work over SMB.
- The copy on Google Drive (`media-drive-push.py`, contract section 8) copies and never deletes:
  for Drive a renamed file is a new one. It uploads it again at its new path and the old path stays
  on Drive. On Peter's library the candidates are about 90 GB, most of it `Hero7`'s videos (72 GB):
  hours of uploading, and as much again on Drive until the old paths are removed.
- The contract says today that no component deletes anything on the NAS or on Drive. A rename
  removes the old path on the NAS, so the contract has to name it as an allowed operation.
- Each Mac's working copy and previews have the old paths too. The other Mac learns of the rename
  only through the NAS.

## Scope

### In scope

- **Rename** a source (`X` → a name the library doesn't have) and **merge** a source into another
  (`X` → `Y`), for all its months or a range of months.
- **The layout stays**: what is below the source (a day folder, `photos/`/`videos/`) stays as it
  is; only the source's folder changes name. A source may have months with and without day
  folders (data contract 3.1, 1.4), so a merge needs no reshuffling.
- **A plan first, as a dry run**: per month what moves where, the names that exist in the target
  month already (a different file gets `~<hash>`, contract 3.3; the same content is a duplicate),
  the bytes Drive will upload again, and the months of the old layout (`MM.<name>`, before 2022),
  which are never touched.
- **Executing**, after a click and only with a snapshot of the share made the same day (the app
  asks to confirm it): per month, move the originals and their sidecars, then the previews and
  thumbnails, then write the month's manifest; journaled like an import (S05 of feature 18), so an
  interruption continues where it was; verified afterwards (every row's file exists with its size
  and hash, nothing left under the old name).
- **This Mac and the other Macs**: the rename is written as a record in the home share (a new part
  of the data contract), and every Mac applies it to its own working copy and previews at its next
  sync, before the manifests are read, so nothing is pulled again and no original seems missing.
- **Duplicates by content** in the target month: the file under the old name isn't moved; the plan
  lists them, and they stay where they are until a later decision (see the open points).
- **In the window**: a tab *Bronnen* in the settings (feature 21) with every source, its count, its
  months and its layout, and *Hernoemen…* per source, which shows the plan and then *Uitvoeren*.

### Out of scope

- Renaming the year subfolders of the old layout (`MM.<name>`, before 2022): read-only by the
  contract.
- Removing the old paths on Drive: a deletion on Drive, decided separately (open point 2).
- Changing a source's layout without renaming it (for example flattening `Hero13`'s day folders);
  it can use the same machinery later.
- The CLI renaming: it gets the record's rules only as far as needed to keep reading (open point 4).

## Acceptance criteria

- Peter merges `iPhone-fotos` into `iPhone-photos`: the plan shows 4 597 files in 109 months, the
  names that clash, and about 9.5 GB for Drive; after *Uitvoeren* every file is under
  `iPhone-photos` on the NAS and on this Mac, the manifests say so, the grid and the lists are the
  same as before, and the source filter has one source fewer.
- The other Mac, at its next sync, moves its own copies the same way and pulls nothing again.
- An execution interrupted at any point (the app quits, the NAS goes away) continues to the same
  result and never leaves a month whose manifest names files that aren't there.
- Nothing outside the source's months is touched; the old-layout folders are never touched.
- `cargo test` covers the plan (layouts, clashes, duplicates, old layout left out), the execution
  and resuming, the record and applying it on another Mac; `pnpm test` the tab's texts.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-the-plan.md) | The plan of a rename or merge, as a dry run: per month what moves where, clashes, duplicates, bytes for Drive | Done | — |
| S02 | The data contract: a rename as an allowed operation, and the rename record every Mac applies | Planned | — |
| S03 | Executing on the NAS: originals, sidecars, previews, manifests, per month, journaled and verified | Planned | S01, S02 |
| S04 | Applying a rename record to this Mac's working copy and previews at the sync, also on the other Mac | Planned | S02, S03 |
| S05 | The tab *Bronnen* in the settings: the sources, *Hernoemen…*, the plan, the confirmation of the snapshot | Planned | S01, S03 |

## Decisions

Taken with Peter on 2026-11-04:

1. **Duplicates by content** in the target month stay under their old name and are listed;
   nothing is deleted.
2. **The old paths on Drive** stay; nothing is deleted on Drive. The renamed files are uploaded
   again by the daily copy.
3. **`Hero7` and `GoProH7`** become **`GoPro-Hero7`**.
4. **The CLI** only keeps reading; it follows through the manifests.
5. **What is below the source stays as it is** (see *The layout stays*): simpler and safer than
   moving files into the target's layout, now that a source may have both.
