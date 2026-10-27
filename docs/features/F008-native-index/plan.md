# Feature 8: Native index: build the index from the manifests

**Status:** In progress (S01 done, S02 and S03 built; in-app checks owed)
**Depends on:** Feature 7 (manifests and previews on this Mac)
**Folder:** `F008-native-index`

## Goal

The app builds and refreshes the index it browses by itself, from the manifests in the working
copy and the files on this Mac. Lynn's Mac then needs no CLI at all: after the sync of feature 7,
the months and photos show up in the grid without `./media index`. On Peter's Mac the app no
longer depends on the CLI having run `media index` after an import or a `thin`.

## Background

- **What `media index` does today** (data contract 7). For every `<year>/<subfolder>/.manifest.json`
  in the working copy it writes one row per entry in `files` to the table `items`:
  - path, folder, month, size, hash, taken, taken_from, kind, source and sidecars (as a JSON list);
  - `local`: the original exists at that path in the working copy;
  - `preview`: `Previews/<path>.jpg` exists, for photos and videos only.
  It also writes one row per manifest to `folders`, and removes folders whose manifest is gone.
  A manifest that can't be read is counted and skipped.
- **Sizes today** (2026-10-27): 364 manifests, 44 MB of JSON, 124 066 items; 30 004 originals and
  38 698 previews local on Peter's Mac.
- **Who writes `index.sqlite`**: data contract 7 says the app may read it and only the CLI writes
  it. The CLI also keeps its own caches in that file (the tables `files` and `previews`), which
  the app must not touch.
- **What the app reads**: `months()` and `items(month)` in `db.rs`, from the tables `items`.

## Scope

### In scope

- **An index builder in Rust** that does what `media index` does, with the same tables and
  columns, so `db.rs` reads it unchanged.
- **The app's own index file**, `~/.config/media/familylightbox-index.sqlite`, written only by the
  app. The CLI's `index.sqlite` is left alone. Data contract 7 lists the new file. On Peter's Mac
  both exist until feature 10 moves his workflow into the app.
- **Building it atomically**: into a temporary file, then renamed into place, so the grid never
  reads a half-built index.
- **Keeping it current**:
  - at start, and after a sync of feature 7 that copied manifests or previews;
  - when the window comes back into focus and a manifest changed since the last build (Peter
    imported or pushed with the CLI);
  - with *Index bijwerken* in the settings panel.
  The grid reloads its months afterwards and keeps the selection where it can.
- **During Lynn's first sync** of tens of GB, the previews appear while they arrive, not only
  after 80 minutes.
- The `./media index` hints in the app go away.

### Out of scope

- **Lists** in the index (feature 3).
- **The CLI's caches** (`files`, `previews`): hashes and capture dates are read from the manifests,
  never computed. Making manifests is Peter's workflow (feature 10).
- **Writing `index.sqlite`**, or changing the CLI. `media index` stays as it is for the CLI's own
  use.
- **Windows.** Nothing in this feature is macOS-specific, but it is only tested on macOS.

## Acceptance criteria

- On Peter's Mac, the app's index has exactly the same `items` and `folders` rows as
  `media index` makes from the same manifests and files (compared row by row, apart from the
  CLI's own tables).
- On a Mac with only the app (tested with `MEDIA_CONFIG` pointing at an empty folder), the grid
  shows every month after the sync of feature 7, with previews, without the CLI.
- A rebuild of the full library takes a few seconds at most (1.3 s measured in S01), and the window stays responsive.
- A broken manifest is skipped and reported; the rest is indexed. A missing working copy gives a
  clear message, not an empty grid without explanation.
- An import with the CLI shows up in the app after switching back to it, without a restart.
- `cargo test` covers the builder on temporary folders (rows, local and preview flags, folder
  without a `folder` field, `month` null, broken manifest, removed folder, atomic replace).
  `pnpm test` covers the new header texts. Existing tests pass.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-index-builder.md) | Index builder: manifests to the app's own index file, atomically, same rows as `media index` | Done | — |
| [S02](stories/S02-app-uses-own-index.md) | The app reads its own index and keeps it current: at start, after a sync, on focus, by hand | In progress: built, in-app check owed | S01 |
| [S03](stories/S03-previews-during-first-sync.md) | Previews appear during a long sync, folder by folder | In progress: built, in-app check owed | S02 |

## Open points

Peter asked on 2026-10-27 to write the stories and start S01 straight away; these points are the
proposals the stories follow, open for review.

1. **The app's own index file** instead of writing `index.sqlite`: two writers on one file would
   mean the CLI's `media index` and the app overwrite each other's rows, and the app would have
   to keep the CLI's caches intact. The cost is a second file of about 70 MB on Peter's Mac.
2. **No fallback to the CLI's index**: once S02 is in, the app reads only its own. It can always
   build it, because the manifests are local after feature 7.
3. **When to rebuild**: a full rebuild is cheap (seconds), so the app always rebuilds everything
   instead of tracking which manifest changed.
