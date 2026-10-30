# Story S01: Reading the capture date and the camera natively, and hashing

**Feature:** `F018-import`
**Status:** In progress
**Depends on:** —

## Goal

The app reads from a photo or video itself what the import needs, without exiftool: the capture
date by the order of data contract 3.4, the camera as in 3.5, and the content hash. It is a port
of the reader in `media-manifest-nas.py` (LibraryLocal), so the app, the NAS script and the CLI
give the same answer for the same file.

## Acceptance criteria

- **Formats**, as the NAS script: JPEG (EXIF in APP1), TIFF-based RAW (NEF, DNG, CR2, ARW, …),
  PNG (`eXIf`, XMP in `iTXt`/`tEXt`), PSD (resources `0x0422` and `0x0424`), AVI (`IDIT`), CR3
  (Canon's `uuid` box, `CMT1`/`CMT2`), HEIC/HEIF (the `Exif` item through `iinf` and `iloc`),
  MP4/MOV (Canon's `uuid` box with its JPEG, Apple's `com.apple.quicktime.creationdate` with its
  zone, otherwise `mvhd` in UTC). An unknown format or a broken file gives nothing, never a panic.
- **The date** from the file: `DateTimeOriginal`, then `DateTimeDigitized`, then `DateTime`; a
  clock time without a zone, except Apple's key (with its zone) and `mvhd` (UTC).
- **The order of 3.4** over a set of files: the file itself, then the partner (same stem in the
  same folder), then a date in the name (2000 up to next year), then the modification time; with
  `taken_from` `exif`, `partner`, `name` or `mtime`. For photos and videos only the file itself
  counts as `exif`.
- **`taken` as the manifest writes it**: in the zone of this Mac, as the NAS script does.
- **The camera**: IFD0's Make and Model (JPEG, TIFF, CR3, HEIC), Apple's
  `com.apple.quicktime.make`/`.model`, GoPro's GPMF `MINF`, Canon's MP4 through its JPEG; joined
  and shortened with the rule of 3.5 (`camera_name`, the same cases as `test-manifest-nas.py`).
- **The hash**: `sha256:` plus 64 hexadecimal characters, read in blocks of 4 MB.
- **The kind** of a file by its extension (3.2): `photo`, `video`, `sidecar`, `other`.
- **Tests** (`cargo test`): every format above from small files the tests build themselves (no
  photos in the repo), the fallbacks, broken and truncated files, the order of 3.4, the camera
  names, the hash of a known text.
- **Checked on real files**: an example program prints date and camera for a list of files; on
  Peter's library its output equals the NAS script's for a sample of every format (recorded
  below, not in CI: the photos are private).

## How

- A new module `src-tauri/src/capture.rs`: a small reader over a `File` (`at(offset, n)`), the
  TIFF tag walker (IFD0 and the Exif IFD, at most two levels, at most 500 entries), the ISOBMFF box
  iterator, and per format a function, as in the Python; one `read(path) -> Capture { date,
  camera }` that opens the file once.
- `Taken` is either a clock time (`NaiveDateTime`) or an instant (`DateTime<FixedOffset>`);
  `manifest_time` writes both in the local zone with seconds and offset, like
  `dt.astimezone().isoformat(timespec="seconds")`.
- `resolve(paths) -> HashMap<PathBuf, (Taken, &str)>` for the order of 3.4.
- `hash.rs`: `sha256_file` with the `sha2` crate.
- Not done here: deciding where a file goes (S04), copying (S05).

## Files affected

- `src-tauri/src/capture.rs` (new), `src-tauri/src/hash.rs` (new), `src-tauri/src/lib.rs`
  (the modules), `src-tauri/Cargo.toml` (`sha2`), `src-tauri/examples/capture.rs` (new).

## Progress log

### [2026-10-30] Written
