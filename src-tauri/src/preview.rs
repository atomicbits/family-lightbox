//! Makes the preview and the thumbnail of one original with macOS's own frameworks (F012/S01):
//! ImageIO for photos (RAW included), AVFoundation for videos, by data contract 4. No `sips`,
//! `exiftool` or `ffmpeg`, so a Mac with only the app can make previews.

use std::path::{Path, PathBuf};

pub const PREVIEW_LONG: usize = 2048;
pub const THUMB_LONG: usize = 400;
const QUALITY: f64 = 0.85;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Photo,
    Video,
}

/// `DateTimeOriginal` and `OffsetTimeOriginal` from a manifest's `taken`
/// (`2026-07-30T16:58:12+02:00`), as the CLI writes them.
pub fn exif_dates(taken: &str) -> Option<(String, Option<String>)> {
    let date = taken.get(..19)?;
    let b = date.as_bytes();
    let digits = [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18];
    if !digits.iter().all(|&i| b[i].is_ascii_digit()) || b[4] != b'-' || b[7] != b'-' || b[10] != b'T' {
        return None;
    }
    let exif = format!("{}:{}:{} {}", &date[..4], &date[5..7], &date[8..10], &date[11..]);
    let rest = &taken[19..];
    let o = rest.as_bytes();
    let offset = (o.len() == 6
        && (o[0] == b'+' || o[0] == b'-')
        && o[3] == b':'
        && [1, 2, 4, 5].iter().all(|&i| o[i].is_ascii_digit()))
    .then(|| rest.to_string());
    Some((exif, offset))
}

/// Where a video's frame is taken: 10 % of the duration, 1 s for a video under 10 s, and half the
/// duration for one under 2 s (data contract 4).
pub fn frame_time(duration: f64) -> f64 {
    if !duration.is_finite() || duration <= 0.0 {
        0.0
    } else if duration >= 10.0 {
        duration * 0.1
    } else {
        (duration / 2.0).min(1.0)
    }
}

/// The size of an image scaled so its long side is at most `long`; never upscaled.
pub fn fit_long(width: usize, height: usize, long: usize) -> (usize, usize) {
    let side = width.max(height);
    if side <= long || side == 0 {
        return (width, height);
    }
    let scale = long as f64 / side as f64;
    let w = ((width as f64 * scale).round() as usize).clamp(1, long);
    let h = ((height as f64 * scale).round() as usize).clamp(1, long);
    (w, h)
}

fn tmp(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

/// Makes `out` (2048 px) and `thumb` (400 px) from `src`. Both are written to `<name>.tmp` and
/// renamed only when both succeeded; on a failure nothing is left behind.
pub fn make(src: &Path, kind: Kind, taken: Option<&str>, out: &Path, thumb: &Path) -> Result<(), String> {
    for dir in [out.parent(), thumb.parent()].into_iter().flatten() {
        std::fs::create_dir_all(dir).map_err(|e| format!("map {} niet aangemaakt: {e}", dir.display()))?;
    }
    let (tmp_out, tmp_thumb) = (tmp(out), tmp(thumb));
    let result = macos::make(src, kind, taken.and_then(exif_dates), &tmp_out, &tmp_thumb)
        .and_then(|()| std::fs::rename(&tmp_out, out).map_err(|e| format!("preview niet weggeschreven: {e}")))
        .and_then(|()| std::fs::rename(&tmp_thumb, thumb).map_err(|e| format!("miniatuur niet weggeschreven: {e}")));
    if result.is_err() {
        for p in [&tmp_out, &tmp_thumb] {
            let _ = std::fs::remove_file(p);
        }
        if !thumb.exists() {
            let _ = std::fs::remove_file(out);
        }
    }
    result
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use objc2_av_foundation::{AVAsset, AVAssetImageGenerator};
    use objc2_core_foundation::{CFBoolean, CFDictionary, CFNumber, CFRetained, CFString, CFType, CFURL, CGPoint, CGRect, CGSize};
    use objc2_core_graphics::{
        CGBitmapContextCreate, CGBitmapContextCreateImage, CGColorSpace, CGContext, CGImage,
        CGImageAlphaInfo, CGInterpolationQuality, kCGColorSpaceSRGB,
    };
    use objc2_core_media::{CMTime, kCMTimeZero};
    use objc2_foundation::NSURL;
    use objc2_image_io::{
        CGImageDestination, CGImageSource, kCGImageDestinationLossyCompressionQuality, kCGImagePropertyExifColorSpace,
        kCGImagePropertyExifDateTimeOriginal, kCGImagePropertyExifDictionary, kCGImagePropertyExifOffsetTimeOriginal,
        kCGImagePropertyPixelHeight, kCGImagePropertyPixelWidth, kCGImageSourceCreateThumbnailFromImageAlways,
        kCGImageSourceCreateThumbnailWithTransform, kCGImageSourceThumbnailMaxPixelSize,
    };

    pub fn make(
        src: &Path,
        kind: Kind,
        dates: Option<(String, Option<String>)>,
        out: &Path,
        thumb: &Path,
    ) -> Result<(), String> {
        let image = match kind {
            Kind::Photo => photo(src)?,
            Kind::Video => video(src)?,
        };
        let preview = redraw(&image, PREVIEW_LONG)?;
        let small = redraw(&preview, THUMB_LONG)?;
        let props = properties(dates);
        write(&preview, &props, out)?;
        write(&small, &props, thumb)
    }

    fn url(path: &Path) -> Result<CFRetained<CFURL>, String> {
        CFURL::from_file_path(path).ok_or_else(|| format!("ongeldig pad: {}", path.display()))
    }

    fn dict(pairs: &[(&CFString, &CFType)]) -> CFRetained<CFDictionary<CFString, CFType>> {
        let (keys, values): (Vec<_>, Vec<_>) = pairs.iter().copied().unzip();
        CFDictionary::from_slices(&keys, &values)
    }

    fn number(dict: &CFDictionary, key: &CFString) -> Option<usize> {
        let dict: &CFDictionary<CFString, CFType> = unsafe { dict.cast_unchecked() };
        dict.get(key)?.downcast_ref::<CFNumber>()?.as_i64().map(|n| n as usize)
    }

    /// The upright image, at most 2048 px on its long side. ImageIO uses a RAW file's embedded JPEG
    /// where it is large enough and decodes the RAW otherwise. `FromImageAlways`, because a JPEG's
    /// small EXIF thumbnail would otherwise be taken as it is.
    fn photo(src: &Path) -> Result<CFRetained<CGImage>, String> {
        let source = unsafe { CGImageSource::with_url(&*url(src)?, None) }
            .ok_or_else(|| format!("kan {} niet openen", src.display()))?;
        if unsafe { source.count() } == 0 {
            return Err("geen beeld dat macOS kan lezen".into());
        }
        let props = unsafe { source.properties_at_index(0, None) };
        let long = props
            .as_deref()
            .and_then(|p| Some(number(p, unsafe { kCGImagePropertyPixelWidth })?.max(number(p, unsafe { kCGImagePropertyPixelHeight })?)))
            .map_or(PREVIEW_LONG, |side| side.min(PREVIEW_LONG));
        let max = CFNumber::new_i64(long as i64);
        let options = dict(&[
            (unsafe { kCGImageSourceThumbnailMaxPixelSize }, &max),
            (unsafe { kCGImageSourceCreateThumbnailWithTransform }, CFBoolean::new(true)),
            (unsafe { kCGImageSourceCreateThumbnailFromImageAlways }, CFBoolean::new(true)),
        ]);
        unsafe { source.thumbnail_at_index(0, Some(options.as_opaque())) }
            .ok_or_else(|| "geen beeld dat macOS kan lezen".into())
    }

    /// The frame at `frame_time`, with the track's rotation applied; the first frame if that fails.
    #[allow(deprecated)] // copyCGImageAtTime is synchronous, which suits a worker thread
    fn video(src: &Path) -> Result<CFRetained<CGImage>, String> {
        let url = NSURL::from_file_path(src).ok_or_else(|| format!("ongeldig pad: {}", src.display()))?;
        unsafe {
            let asset = AVAsset::assetWithURL(&url);
            let generator = AVAssetImageGenerator::assetImageGeneratorWithAsset(&asset);
            generator.setAppliesPreferredTrackTransform(true);
            generator.setMaximumSize(CGSize::new(PREVIEW_LONG as f64, PREVIEW_LONG as f64));
            generator.setRequestedTimeToleranceBefore(kCMTimeZero);
            generator.setRequestedTimeToleranceAfter(kCMTimeZero);
            let at = frame_time(asset.duration().seconds());
            let mut error = String::new();
            for t in [at, 0.0] {
                match generator.copyCGImageAtTime_actualTime_error(CMTime::with_seconds(t, 600), std::ptr::null_mut()) {
                    Ok(image) => return Ok(CFRetained::retain(std::ptr::NonNull::from(&*image).cast())),
                    Err(e) => error = e.localizedDescription().to_string(),
                }
                if t == 0.0 {
                    break;
                }
            }
            Err(format!("geen beeld uit de video: {error}"))
        }
    }

    /// The image in sRGB, scaled to at most `long` px; transparency becomes white.
    fn redraw(image: &CGImage, long: usize) -> Result<CFRetained<CGImage>, String> {
        let (w, h) = fit_long(CGImage::width(Some(image)), CGImage::height(Some(image)), long);
        let srgb = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB })).ok_or("geen sRGB-kleurruimte")?;
        let info = CGImageAlphaInfo::NoneSkipLast.0; // default byte order
        let ctx: CFRetained<CGContext> =
            unsafe { CGBitmapContextCreate(std::ptr::null_mut(), w, h, 8, 0, Some(&srgb), info) }
                .ok_or("geen tekenvlak")?;
        let rect = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(w as f64, h as f64));
        CGContext::set_interpolation_quality(Some(&ctx), CGInterpolationQuality::High);
        CGContext::set_rgb_fill_color(Some(&ctx), 1.0, 1.0, 1.0, 1.0);
        CGContext::fill_rect(Some(&ctx), rect);
        CGContext::draw_image(Some(&ctx), rect, Some(image));
        CGBitmapContextCreateImage(Some(&ctx)).ok_or_else(|| "beeld niet getekend".into())
    }

    /// Quality 85 and only the metadata data contract 4 allows: the dates and `ColorSpace = sRGB`.
    fn properties(dates: Option<(String, Option<String>)>) -> CFRetained<CFDictionary<CFString, CFType>> {
        let srgb = CFNumber::new_i32(1);
        let (date, offset) = match &dates {
            Some((d, o)) => (Some(CFString::from_str(d)), o.as_deref().map(CFString::from_str)),
            None => (None, None),
        };
        let mut exif: Vec<(&CFString, &CFType)> = vec![(unsafe { kCGImagePropertyExifColorSpace }, &srgb)];
        if let Some(d) = &date {
            exif.push((unsafe { kCGImagePropertyExifDateTimeOriginal }, d));
        }
        if let Some(o) = &offset {
            exif.push((unsafe { kCGImagePropertyExifOffsetTimeOriginal }, o));
        }
        let exif = dict(&exif);
        let quality = CFNumber::new_f64(QUALITY);
        dict(&[
            (unsafe { kCGImageDestinationLossyCompressionQuality }, &quality),
            (unsafe { kCGImagePropertyExifDictionary }, &exif),
        ])
    }

    fn write(image: &CGImage, props: &CFDictionary<CFString, CFType>, path: &Path) -> Result<(), String> {
        let jpeg = CFString::from_static_str("public.jpeg");
        let dest = unsafe { CGImageDestination::with_url(&*url(path)?, &jpeg, 1, None) }
            .ok_or_else(|| format!("kan {} niet schrijven", path.display()))?;
        unsafe {
            dest.add_image(image, Some(props.as_opaque()));
            if dest.finalize() { Ok(()) } else { Err(format!("JPEG {} niet geschreven", path.display())) }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use objc2_core_graphics::kCGColorSpaceDisplayP3;
        use objc2_image_io::{kCGImagePropertyOrientation, kCGImagePropertyExifDictionary as EXIF};

        /// An image of `w` × `h` with these numbers in `space`: the top half `top`, the bottom half
        /// `bottom`. The bytes are written directly, so no color matching changes them.
        fn image(w: usize, h: usize, space: &CFString, top: [f64; 3], bottom: [f64; 3]) -> CFRetained<CGImage> {
            let cs = CGColorSpace::with_name(Some(space)).unwrap();
            let mut buf = vec![0u8; w * h * 4];
            for (row, line) in buf.chunks_mut(w * 4).enumerate() {
                let c = if row < h / 2 { top } else { bottom };
                for px in line.chunks_mut(4) {
                    for (b, v) in px.iter_mut().zip(c) {
                        *b = (v * 255.0).round() as u8;
                    }
                }
            }
            let ctx = unsafe {
                CGBitmapContextCreate(buf.as_mut_ptr().cast(), w, h, 8, w * 4, Some(&cs), CGImageAlphaInfo::NoneSkipLast.0)
            }
            .unwrap();
            CGBitmapContextCreateImage(Some(&ctx)).unwrap()
        }

        fn save(image: &CGImage, uti: &'static str, orientation: Option<i32>, path: &Path) {
            let dest = unsafe { CGImageDestination::with_url(&url(path).unwrap(), &CFString::from_static_str(uti), 1, None) }.unwrap();
            let o = orientation.map(CFNumber::new_i32);
            let props = o.as_ref().map(|o| dict(&[(unsafe { kCGImagePropertyOrientation }, &**o)]));
            unsafe {
                dest.add_image(image, props.as_ref().map(|p| p.as_opaque()));
                assert!(dest.finalize());
            }
        }

        struct Out {
            width: usize,
            height: usize,
            orientation: Option<usize>,
            exif: Vec<(String, String)>,
        }

        fn read(path: &Path) -> Out {
            let source = unsafe { CGImageSource::with_url(&url(path).unwrap(), None) }.unwrap();
            assert_eq!(unsafe { source.r#type() }.unwrap().to_string(), "public.jpeg");
            let props = unsafe { source.properties_at_index(0, None) }.unwrap();
            let all: &CFDictionary<CFString, CFType> = unsafe { props.cast_unchecked() };
            let exif = all
                .get(unsafe { EXIF })
                .and_then(|e| e.downcast::<CFDictionary>().ok())
                .map(|e| {
                    let e: &CFDictionary<CFString, CFType> = unsafe { e.cast_unchecked() };
                    let (k, v) = e.to_vecs();
                    let mut pairs: Vec<_> = k
                        .iter()
                        .zip(v.iter())
                        .filter(|(k, _)| !k.to_string().starts_with("Pixel"))
                        .map(|(k, v)| {
                            let v = v.downcast_ref::<CFString>().map(|s| s.to_string()).or_else(|| {
                                v.downcast_ref::<CFNumber>().and_then(|n| n.as_i64()).map(|n| n.to_string())
                            });
                            (k.to_string(), v.unwrap_or_default())
                        })
                        .collect();
                    pairs.sort();
                    pairs
                })
                .unwrap_or_default();
            Out {
                width: number(&props, unsafe { kCGImagePropertyPixelWidth }).unwrap(),
                height: number(&props, unsafe { kCGImagePropertyPixelHeight }).unwrap(),
                orientation: number(&props, unsafe { kCGImagePropertyOrientation }),
                exif,
            }
        }

        /// The sRGB color at (x, y) from the top left, as 0–255.
        fn pixel(path: &Path, x: usize, y: usize) -> [u8; 3] {
            let source = unsafe { CGImageSource::with_url(&url(path).unwrap(), None) }.unwrap();
            let img = unsafe { source.image_at_index(0, None) }.unwrap();
            let (w, h) = (CGImage::width(Some(&img)), CGImage::height(Some(&img)));
            let mut buf = vec![0u8; w * h * 4];
            let srgb = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB })).unwrap();
            let ctx = unsafe {
                CGBitmapContextCreate(buf.as_mut_ptr().cast(), w, h, 8, w * 4, Some(&srgb), CGImageAlphaInfo::NoneSkipLast.0)
            }
            .unwrap();
            CGContext::draw_image(Some(&ctx), CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(w as f64, h as f64)), Some(&img));
            drop(ctx);
            let i = (y * w + x) * 4;
            [buf[i], buf[i + 1], buf[i + 2]]
        }

        fn outputs(dir: &Path, name: &str) -> (PathBuf, PathBuf) {
            (dir.join(format!("{name}.jpg")), dir.join(format!("{name}.t.jpg")))
        }

        const RED: [f64; 3] = [1.0, 0.0, 0.0];
        const BLUE: [f64; 3] = [0.0, 0.0, 1.0];

        #[test]
        fn a_large_jpeg_gets_2048_and_400_and_only_the_allowed_metadata() {
            let dir = tempfile::tempdir().unwrap();
            let src = dir.path().join("big.JPG");
            save(&image(3000, 2000, unsafe { kCGColorSpaceSRGB }, RED, BLUE), "public.jpeg", None, &src);
            let (out, thumb) = outputs(dir.path(), "previews/2026/07/big.JPG");
            super::super::make(&src, Kind::Photo, Some("2026-07-30T16:58:12+02:00"), &out, &thumb).unwrap();
            let (p, t) = (read(&out), read(&thumb));
            assert_eq!((p.width, p.height, t.width, t.height), (2048, 1365, 400, 267));
            let expected = vec![
                ("ColorSpace".to_string(), "1".to_string()),
                ("DateTimeOriginal".into(), "2026:07:30 16:58:12".into()),
                ("OffsetTimeOriginal".into(), "+02:00".into()),
            ];
            assert_eq!(p.exif, expected);
            assert_eq!(t.exif, expected);
            assert_eq!(p.orientation, None);
        }

        #[test]
        fn a_small_png_is_not_upscaled() {
            let dir = tempfile::tempdir().unwrap();
            let src = dir.path().join("small.PNG");
            save(&image(300, 200, unsafe { kCGColorSpaceSRGB }, RED, BLUE), "public.png", None, &src);
            let (out, thumb) = outputs(dir.path(), "small.PNG");
            super::super::make(&src, Kind::Photo, None, &out, &thumb).unwrap();
            let (p, t) = (read(&out), read(&thumb));
            assert_eq!((p.width, p.height, t.width, t.height), (300, 200, 300, 200));
            assert_eq!(p.exif, vec![("ColorSpace".to_string(), "1".to_string())]);
        }

        #[test]
        fn a_rotated_jpeg_comes_out_upright_without_an_orientation_tag() {
            let dir = tempfile::tempdir().unwrap();
            let src = dir.path().join("rotated.JPG");
            // stored landscape, red on top; orientation 6 shows it turned 90° clockwise, red on the right
            save(&image(3000, 2000, unsafe { kCGColorSpaceSRGB }, RED, BLUE), "public.jpeg", Some(6), &src);
            let (out, thumb) = outputs(dir.path(), "rotated.JPG");
            super::super::make(&src, Kind::Photo, None, &out, &thumb).unwrap();
            let p = read(&out);
            assert_eq!((p.width, p.height, p.orientation), (1365, 2048, None));
            let right = pixel(&out, 1300, 1024);
            let left = pixel(&out, 60, 1024);
            assert!(right[0] > 200 && right[2] < 60, "right {right:?}");
            assert!(left[2] > 200 && left[0] < 60, "left {left:?}");
        }

        #[test]
        fn display_p3_is_converted_to_srgb() {
            let dir = tempfile::tempdir().unwrap();
            let src = dir.path().join("p3.tiff");
            let p3 = [0.8, 0.3, 0.2];
            save(&image(600, 400, unsafe { kCGColorSpaceDisplayP3 }, p3, p3), "public.tiff", None, &src);
            let (out, thumb) = outputs(dir.path(), "p3.tiff");
            super::super::make(&src, Kind::Photo, None, &out, &thumb).unwrap();
            // the same color in sRGB has more red and less green than its P3 numbers (204, 77, 51)
            let [r, g, _] = pixel(&out, 300, 200);
            assert!(r > 212 && g < 70, "{:?}", pixel(&out, 300, 200));
        }

        #[test]
        fn a_video_gives_an_upright_frame() {
            let dir = tempfile::tempdir().unwrap();
            let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/testsrc-rotated.mp4");
            let (out, thumb) = outputs(dir.path(), "testsrc-rotated.mp4");
            super::super::make(&src, Kind::Video, Some("2026-07-30T16:58:12"), &out, &thumb).unwrap();
            let (p, t) = (read(&out), read(&thumb));
            // 320 × 240, shown rotated a quarter turn: 240 × 320; no upscaling
            assert_eq!((p.width, p.height, t.width, t.height), (240, 320, 240, 320));
            assert!(p.exif.contains(&("DateTimeOriginal".to_string(), "2026:07:30 16:58:12".to_string())));
            assert!(!p.exif.iter().any(|(k, _)| k == "OffsetTimeOriginal"));
        }

        #[test]
        fn a_file_that_is_not_an_image_fails_and_leaves_nothing() {
            let dir = tempfile::tempdir().unwrap();
            let src = dir.path().join("notes.JPG");
            std::fs::write(&src, b"no image here").unwrap();
            let (out, thumb) = outputs(dir.path(), "notes.JPG");
            assert!(super::super::make(&src, Kind::Photo, None, &out, &thumb).is_err());
            let video = super::super::make(&src, Kind::Video, None, &out, &thumb);
            assert!(video.is_err(), "{video:?}");
            let left: Vec<_> = std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().file_name()).collect();
            assert_eq!(left, vec![std::ffi::OsString::from("notes.JPG")]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exif_dates_from_taken() {
        assert_eq!(
            exif_dates("2026-07-30T16:58:12+02:00"),
            Some(("2026:07:30 16:58:12".into(), Some("+02:00".into())))
        );
        assert_eq!(exif_dates("2026-07-30T16:58:12"), Some(("2026:07:30 16:58:12".into(), None)));
        assert_eq!(exif_dates("2026-07-30T16:58:12Z"), Some(("2026:07:30 16:58:12".into(), None)));
        assert_eq!(exif_dates("2026-07-30"), None);
        assert_eq!(exif_dates("onbekend datum en uur"), None);
    }

    #[test]
    fn frame_time_by_duration() {
        assert_eq!(frame_time(60.0), 6.0);
        assert_eq!(frame_time(10.0), 1.0);
        assert_eq!(frame_time(5.0), 1.0);
        assert_eq!(frame_time(1.0), 0.5);
        assert_eq!(frame_time(f64::NAN), 0.0);
    }

    #[test]
    fn fit_long_never_upscales() {
        assert_eq!(fit_long(6000, 4000, 2048), (2048, 1365));
        assert_eq!(fit_long(4000, 6000, 2048), (1365, 2048));
        assert_eq!(fit_long(800, 600, 2048), (800, 600));
        assert_eq!(fit_long(2048, 1365, 400), (400, 267));
    }
}
