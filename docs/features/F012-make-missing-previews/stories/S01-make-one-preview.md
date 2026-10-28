# Story S01: Make one preview and thumbnail natively, by data contract 4

**Feature:** `F012-make-missing-previews`
**Status:** Done
**Depends on:** —

## Goal

A Rust function that makes the preview and the thumbnail of one original with macOS's own
frameworks, ImageIO for photos and AVFoundation for videos, by the rules of data contract 4. It
is the building block of S03, which runs it for every missing preview. Nothing in the app calls
it yet.

## Acceptance criteria

- **Input**: the path of the original, its kind (`photo` or `video`), `taken` from the manifest
  (may be missing), and the two output paths (`<original>.jpg` and `<original>.t.jpg`).
- **Photos** (JPEG, HEIC, PNG, TIFF, and RAW: CR2, CR3, DNG, …):
  - scaled so the long side is 2048 px, never upscaled; an original smaller than that keeps its
    own size;
  - upright: the EXIF orientation is applied, and the result has no orientation tag;
  - RAW: through ImageIO's thumbnail of the RAW file. Where the file embeds a JPEG at least as
    large as the preview, that JPEG is used; otherwise ImageIO decodes the RAW (decision 3 of the
    plan).
- **Videos** (MP4, MOV, HEVC): the frame at 10 % of the duration, or at 1 s when the video is
  shorter than 10 s (half the duration when it is shorter than 2 s), with the track's rotation
  applied. If that frame can't be read, the first frame. Scaled as photos.
- **The thumbnail** is made from the preview, long side 400 px, never upscaled.
- **Both files**: JPEG quality 85, converted to sRGB, with only `DateTimeOriginal` and
  `OffsetTimeOriginal` (from `taken`, when it has them) and `ColorSpace = sRGB` as metadata: no
  GPS, camera settings or orientation.
- **Atomic**: both are written to `<name>.tmp` first and renamed into place only when both
  succeeded; on a failure, nothing is left behind and the error says what went wrong (a file that
  can't be read, a format macOS doesn't know, a video without a frame).
- **No external tools**: no `sips`, `exiftool` or `ffmpeg`; only the system frameworks, through
  the `objc2` binding crates.
- **Tests** (`cargo test`), on files the test makes itself, so no personal photos enter the repo:
  - a JPEG larger than 2048 px: 2048 and 400 px long side;
  - a small PNG: keeps its size (no upscaling);
  - a JPEG with orientation 6 (rotated): the preview is upright and has no orientation tag;
  - a Display P3 image: the preview is sRGB;
  - `taken` with and without an offset: the right `DateTimeOriginal` and `OffsetTimeOriginal`;
  - a short generated video (a few KB, a test pattern, in `src-tauri/tests/fixtures/`): a frame
    at the right size;
  - a file that isn't an image: an error, and no output files.
- **By hand**: a small example program (`cargo run --example make-preview -- <original> <folder>`)
  makes the previews of chosen originals, to compare them with `media previews` on the same files:
  a JPEG, an iPhone HEIC (Display P3), a portrait photo, a CR2 and a CR3, an MP4 and an iPhone MOV.
  Sizes, orientation and metadata must match (checked with `exiftool`); small pixel differences
  between the encoders are fine.

## How

- A module `src-tauri/src/preview.rs`, macOS only (`#[cfg(target_os = "macos")]`), with
  `make(src, kind, taken, out, thumb) -> Result<(), String>`.
- **Photos**: `CGImageSourceCreateWithURL`, then `CGImageSourceCreateThumbnailAtIndex` with
  `ThumbnailMaxPixelSize` = min(2048, long side), `CreateThumbnailWithTransform` and
  `CreateThumbnailFromImageAlways` (not `IfAbsent`: a JPEG's small EXIF thumbnail would be used
  as it is). Measured on a CR3 and a NEF: with `Always`, ImageIO gives the 2048 px image in
  about 80 ms, so it works from the embedded JPEG; the same option serves RAW and the rest.
- **Videos**: `AVURLAsset`, its duration, and `AVAssetImageGenerator` with
  `appliesPreferredTrackTransform` and `maximumSize` 2048 × 2048.
- **sRGB and the thumbnail**: the image is drawn into a `CGBitmapContext` in the sRGB color space,
  at the preview's size, and again at 400 px for the thumbnail.
- **Writing**: `CGImageDestinationCreateWithURL` (`public.jpeg`) with
  `LossyCompressionQuality` 0.85 and an `{Exif}` dictionary with only the dates and `ColorSpace`.
- **Data contract 4**: the RAW rule becomes "the embedded JPEG where it is at least as large as
  the preview, else the decoded RAW", and the paragraph on who makes previews names the app next
  to the CLI. The version goes up, with a line in section 12.
- Deliberately not done here: which previews are missing (S02), running it for many and writing
  to the NAS (S03), the grid (S04).

## Files affected

- `src-tauri/Cargo.toml` (the `objc2` crates for ImageIO, CoreGraphics, AVFoundation, CoreMedia),
  `src-tauri/src/preview.rs` (new), `src-tauri/src/lib.rs`, `src-tauri/examples/make-preview.rs`
  (new), `src-tauri/tests/fixtures/` (a generated test video), `docs/data-contract.md`.

## Progress log

### [2026-10-28] Written

### [2026-10-28] Built and checked

- `preview.rs` with `make()`, `exif_dates()`, `frame_time()` and `fit_long()`; the macOS part
  through `objc2-image-io`, `objc2-core-graphics`, `objc2-av-foundation` and `objc2-core-media`
  0.3. `AVAssetImageGenerator`'s synchronous `copyCGImageAtTime` is deprecated but suits a
  worker thread; S03 can move to the asynchronous call if it has to.
- 14 tests (`cargo test preview`), 86 in all, clippy clean. The test images are made by the
  test itself, with their bytes written directly (`CGContextSetRGBFillColor` works in device RGB
  and would already match the colors); the test video is a 7 KB generated test pattern with a
  quarter-turn rotation, without metadata.
- By hand with `cargo run --release --example make-preview`, on a JPEG, an iPhone HEIC (Display
  P3), a CR3, a NEF, an MP4 and an iPhone MOV (portrait), and an old AVI:
  - all made, in 30–270 ms each (CR3 270 ms, HEIC 200 ms, NEF 120 ms, MP4 30 ms);
  - sizes identical to the CLI's previews of the same originals (CR3 2048 × 1365, JPEG portrait
    1536 × 2048, MOV 1308 × 1744, MP4 480 × 848, thumbnails likewise), no orientation tag, only
    `DateTimeOriginal`, `OffsetTimeOriginal` and `ColorSpace` besides JFIF;
  - the MOV's frame is the same as the CLI's, slightly lighter: AVFoundation maps the iPhone's
    HDR to sRGB differently from ffmpeg;
  - the video previews are larger files than the CLI's (the CLI's ffmpeg frame is recompressed
    harder); within the contract.
  - The library has no CR2 files; the NEF stood in for an older RAW. AVI, which the CLI read with
    ffmpeg, also works with AVFoundation.
- Data contract 4: the RAW and video rules and who makes previews updated (1.2 addition).
