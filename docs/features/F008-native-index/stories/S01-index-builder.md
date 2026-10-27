# Story S01: Index builder: manifests to the app's own index file, atomically, same rows as `media index`

**Feature:** `F008-native-index`
**Status:** In progress
**Depends on:** —

## Goal

A Rust function that reads every manifest in the working copy and writes the app's own index,
`familylightbox-index.sqlite`, with the same `items` and `folders` rows that `media index` makes.
It is the building block for S02, which switches the grid over to it and decides when it runs.

## Acceptance criteria

- **Input**:
  - every `<year>/<subfolder>/.manifest.json` under `MEDIA_LIBRARY`, where `<year>` is four
    digits and `<subfolder>` is not excluded (data contract 1); both sorted by name;
  - `MEDIA_PREVIEWS` for the preview flag.
- **Rows**, as `media index` writes them:
  - `items`: one row per entry in `files`, with path, folder, month, size, hash, taken,
    taken_from, kind, source, `sidecars` as a JSON list (`[]` when missing), `local` (the
    original is a file at `MEDIA_LIBRARY/<path>`) and `preview` (`MEDIA_PREVIEWS/<path>.jpg` is a
    file, for photos and videos only);
  - `folders`: folder, month, contract_version, generated_at, generated_by, and n_files;
  - `folder` comes from the manifest, or from the manifest's location when the field is missing
    (versions 1.0 and 1.1); `month` may be null;
  - a path that occurs twice keeps the last row, as `INSERT OR REPLACE` does in the CLI.
- **Same schema** as the CLI (`folders`, `items` and the indexes on month, hash and taken), so
  `db.rs` reads it unchanged.
- **Atomic**: the index is written to `familylightbox-index.sqlite.tmp` in one transaction and
  renamed over the old file. A failed build leaves the old index in place and removes the
  temporary file. A reader with the old file open keeps reading it.
- **Broken manifests**: a manifest that can't be read, isn't valid JSON or has no `files` list is
  skipped and named in the result; the rest is indexed. A missing `MEDIA_LIBRARY` is an error
  ("werkkopie niet gevonden: …").
- **Result**: manifests read, rows per kind, local originals, local previews, the broken
  manifests, and how long it took.
- **Data contract 7** lists `familylightbox-index.sqlite` (the app's index, same tables, only the
  app writes it), with a line in the change log.
- **Tests** (`cargo test`, temporary folders):
  - rows and flags for a manifest with a photo, a video, a sidecar and an orphan sidecar;
  - a manifest without `folder`, and one with `month` null;
  - a broken manifest is skipped and named;
  - an excluded subfolder and a non-year folder are ignored;
  - the result opens with `Db` and gives the expected months;
  - a rebuild replaces the old index, and a failed one leaves it.
  Existing tests still pass, and clippy is clean.
- **By hand**, on Peter's real working copy: build into a temporary config folder and compare
  `items` and `folders` row by row with the CLI's `index.sqlite` after a fresh `./media index`.
  Note the time it took.

## How

- `src-tauri/src/index.rs` (new):
  - `INDEX_FILE = "familylightbox-index.sqlite"`;
  - `manifest_paths(library)`;
  - `build(library, previews, target) -> Result<BuildOutcome, String>`, which creates the tables
    with the CLI's statements and inserts with one prepared statement per table;
  - manifests are parsed with serde into a small struct, so unknown fields are ignored.
- `lib.rs`: only `mod index;` for now. The commands, the switch to the new file and when it runs
  are S02.
- A throwaway example for the comparison by hand, removed afterwards.
- Deliberately not done here: reading the new file in the grid, the commands, the header line
  (S02); updating during a sync (S03).

## Files affected

- `src-tauri/src/index.rs` (new), `src-tauri/src/lib.rs`.
- `docs/data-contract.md` section 7 and its change log.

## Progress log

### [2026-10-27] Written
