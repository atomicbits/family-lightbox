//! What an import reads from a photo or video itself (F018/S01): the capture date in the order of
//! data contract 3.4 and the camera of 3.5, without exiftool. A port of the reader in
//! `media-manifest-nas.py` (LibraryLocal), so the app, the NAS script and the CLI give the same
//! answer for the same file. Every read is bounded and a broken file gives nothing, never a panic.

use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, TimeZone};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub const PHOTO_EXT: &[&str] = &[
    "jpg", "jpeg", "heic", "heif", "png", "gif", "tif", "tiff", "webp", "bmp", "dng", "cr2", "cr3", "crw", "raf",
    "arw", "nef", "orf", "rw2", "pef", "srw", "psd",
];
pub const VIDEO_EXT: &[&str] = &[
    "mp4", "mov", "m4v", "avi", "3gp", "3g2", "mkv", "mts", "m2ts", "webm", "mpg", "mpeg", "wmv", "flv", "360",
    "insv", "lrv", "lrf",
];
pub const SIDECAR_EXT: &[&str] = &["xmp", "gpx", "thm", "sav", "json", "aae"];

fn ext_of(path: &Path) -> String {
    path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default()
}

/// The kind of an original by its extension (data contract 3.2).
pub fn kind_of(path: &Path) -> &'static str {
    let ext = ext_of(path);
    if SIDECAR_EXT.contains(&ext.as_str()) {
        "sidecar"
    } else if PHOTO_EXT.contains(&ext.as_str()) {
        "photo"
    } else if VIDEO_EXT.contains(&ext.as_str()) {
        "video"
    } else {
        "other"
    }
}

/// A capture date: a camera's clock time without a zone, or an instant (Apple's key with its
/// zone, a video's `mvhd` in UTC, a file's modification time).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Taken {
    Clock(NaiveDateTime),
    Instant(DateTime<FixedOffset>),
}

impl Taken {
    /// As the manifest writes it: in this Mac's zone, with seconds and offset, like Python's
    /// `dt.astimezone().isoformat(timespec="seconds")`.
    pub fn manifest_time(&self) -> String {
        let local = match self {
            Taken::Instant(t) => t.with_timezone(&Local),
            Taken::Clock(n) => match Local.from_local_datetime(n).earliest() {
                Some(t) => t,
                // a clock time in the hour skipped when summer time starts
                None => Local.from_local_datetime(&(*n + chrono::Duration::hours(1))).earliest().unwrap_or_else(|| Local.from_utc_datetime(n)),
            },
        };
        local.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
    }

    /// The clock time as the camera or phone showed it.
    pub fn clock(&self) -> NaiveDateTime {
        match self {
            Taken::Clock(n) => *n,
            Taken::Instant(t) => t.naive_local(),
        }
    }
}

/// What one file says about itself.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Capture {
    pub date: Option<Taken>,
    pub camera: Option<String>,
}

/// The date and the camera of one file, opened once.
pub fn read(path: &Path) -> Capture {
    let Ok(f) = File::open(path) else { return Capture::default() };
    let Ok(meta) = f.metadata() else { return Capture::default() };
    let mut r = R { f };
    let size = meta.len();
    let ext = ext_of(path);
    Capture { date: date_of(&mut r, size, &ext), camera: camera_of(&mut r, size, &ext) }
}

// ── reading at positions ─────────────────────────────────────────────────────

struct R {
    f: File,
}

impl R {
    /// Up to `n` bytes at `off`; fewer at the end of the file, none on an error.
    fn at(&mut self, off: u64, n: u64) -> Vec<u8> {
        let mut out = Vec::new();
        if self.f.seek(SeekFrom::Start(off)).is_ok() {
            let _ = (&mut self.f).take(n).read_to_end(&mut out);
        }
        out
    }
}

fn be16(b: &[u8]) -> Option<u16> {
    Some(u16::from_be_bytes(b.get(..2)?.try_into().ok()?))
}
fn be32(b: &[u8]) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(..4)?.try_into().ok()?))
}
fn be64(b: &[u8]) -> Option<u64> {
    Some(u64::from_be_bytes(b.get(..8)?.try_into().ok()?))
}
fn u16e(b: &[u8], le: bool) -> Option<u16> {
    let a: [u8; 2] = b.get(..2)?.try_into().ok()?;
    Some(if le { u16::from_le_bytes(a) } else { u16::from_be_bytes(a) })
}
fn u32e(b: &[u8], le: bool) -> Option<u32> {
    let a: [u8; 4] = b.get(..4)?.try_into().ok()?;
    Some(if le { u32::from_le_bytes(a) } else { u32::from_be_bytes(a) })
}

/// Text up to the first NUL, trimmed.
fn text(raw: &[u8]) -> String {
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    String::from_utf8_lossy(&raw[..end]).trim().to_string()
}

// ── TIFF (EXIF) ──────────────────────────────────────────────────────────────

const TAG_DATETIME: u16 = 0x0132;
const TAG_EXIF_IFD: u16 = 0x8769;
const TAG_DT_ORIGINAL: u16 = 0x9003;
const TAG_DT_DIGITIZED: u16 = 0x9004;
const TAG_MAKE: u16 = 0x010F;
const TAG_MODEL: u16 = 0x0110;

/// The dates (IFD0 and the Exif IFD) and make and model (IFD0) of a TIFF structure at `base`.
fn tiff_tags(r: &mut R, base: u64) -> HashMap<u16, Vec<u8>> {
    let mut found = HashMap::new();
    let hdr = r.at(base, 8);
    if hdr.len() < 8 || (&hdr[..2] != b"II" && &hdr[..2] != b"MM") {
        return found;
    }
    let le = &hdr[..2] == b"II";
    let Some(ifd0) = u32e(&hdr[4..], le) else { return found };
    tiff_ifd(r, base, le, ifd0 as u64, &[TAG_DATETIME, TAG_MAKE, TAG_MODEL], 0, &mut found);
    found
}

fn tiff_ifd(r: &mut R, base: u64, le: bool, off: u64, wanted: &[u16], depth: u32, found: &mut HashMap<u16, Vec<u8>>) {
    if depth > 2 || off == 0 || off > 64 * 1024 * 1024 {
        return;
    }
    let Some(n) = u16e(&r.at(base + off, 2), le) else { return };
    let n = n.min(500) as u64;
    let entries = r.at(base + off + 2, 12 * n);
    for e in entries.as_chunks::<12>().0 {
        let (Some(tag), Some(typ), Some(count)) = (u16e(e, le), u16e(&e[2..], le), u32e(&e[4..], le)) else { continue };
        let val = &e[8..12];
        if tag == TAG_EXIF_IFD && (typ == 4 || typ == 13) {
            if let Some(sub) = u32e(val, le) {
                tiff_ifd(r, base, le, sub as u64, &[TAG_DT_ORIGINAL, TAG_DT_DIGITIZED], depth + 1, found);
            }
        } else if wanted.contains(&tag) && typ == 2 && count > 0 && count <= 64 {
            // up to four characters are in the entry itself, longer text at an offset
            let v = if count <= 4 {
                val[..count as usize].to_vec()
            } else {
                match u32e(val, le) {
                    Some(at) => r.at(base + at as u64, count as u64),
                    None => continue,
                }
            };
            found.insert(tag, v);
        }
    }
}

/// `2026:07:30 16:58:12`, with `T` allowed, NUL and spaces around; year 0000 is no date.
fn parse_exif_date(raw: &[u8]) -> Option<NaiveDateTime> {
    let s = String::from_utf8_lossy(raw);
    let s = s.trim_matches(|c| c == '\0' || c == ' ').as_bytes();
    if s.len() < 19 || s[4] != b':' || s[7] != b':' || (s[10] != b' ' && s[10] != b'T') || s[13] != b':' || s[16] != b':' {
        return None;
    }
    let num = |a: usize, b: usize| -> Option<u32> {
        let t = &s[a..b];
        if t.iter().all(u8::is_ascii_digit) { std::str::from_utf8(t).ok()?.parse().ok() } else { None }
    };
    let year = num(0, 4)?;
    if year == 0 {
        return None;
    }
    NaiveDate::from_ymd_opt(year as i32, num(5, 7)?, num(8, 10)?)?.and_hms_opt(num(11, 13)?, num(14, 16)?, num(17, 19)?)
}

fn tiff_date(r: &mut R, base: u64) -> Option<Taken> {
    let found = tiff_tags(r, base);
    [TAG_DT_ORIGINAL, TAG_DT_DIGITIZED, TAG_DATETIME]
        .iter()
        .find_map(|t| found.get(t).and_then(|v| parse_exif_date(v)))
        .map(Taken::Clock)
}

fn tiff_camera(r: &mut R, base: u64) -> Option<String> {
    let found = tiff_tags(r, base);
    let get = |t| found.get(&t).map(|v| text(v)).unwrap_or_default();
    camera_name(&get(TAG_MAKE), &get(TAG_MODEL))
}

type Tiff<T> = fn(&mut R, u64) -> Option<T>;

/// A JPEG at `base`: the TIFF in its APP1 `Exif` segment.
fn jpeg<T>(r: &mut R, base: u64, tiff: Tiff<T>) -> Option<T> {
    let mut pos = base + 2;
    for _ in 0..64 {
        let h = r.at(pos, 4);
        if h.len() < 4 || h[0] != 0xFF {
            return None;
        }
        let (marker, seglen) = (h[1], be16(&h[2..])? as u64);
        if marker == 0xDA || marker == 0xD9 {
            return None; // image data or the end
        }
        if marker == 0xE1 && r.at(pos + 4, 6) == b"Exif\0\0" {
            return tiff(r, pos + 10);
        }
        pos += 2 + seglen;
    }
    None
}

// ── PNG, PSD, AVI ────────────────────────────────────────────────────────────

const XMP_KEYS: &[&[u8]] = &[b"exif:DateTimeOriginal", b"photoshop:DateCreated", b"xmp:CreateDate"];

/// The first of XMP's date keys followed by `YYYY-MM-DDTHH:MM:SS`.
fn xmp_date(blob: &[u8]) -> Option<Taken> {
    for i in 0..blob.len() {
        let Some(key) = XMP_KEYS.iter().find(|k| blob[i..].starts_with(k)) else { continue };
        let mut j = i + key.len();
        while j < blob.len() && (b"=>\"".contains(&blob[j]) || blob[j].is_ascii_whitespace()) {
            j += 1;
        }
        let Some(s) = blob.get(j..j + 19) else { continue };
        let shape = s.iter().enumerate().all(|(k, &c)| match k {
            4 | 7 => c == b'-',
            10 => c == b'T',
            13 | 16 => c == b':',
            _ => c.is_ascii_digit(),
        });
        if !shape {
            continue;
        }
        let n = |a: usize, b: usize| std::str::from_utf8(&s[a..b]).ok().and_then(|t| t.parse::<u32>().ok());
        return NaiveDate::from_ymd_opt(n(0, 4)? as i32, n(5, 7)?, n(8, 10)?)?
            .and_hms_opt(n(11, 13)?, n(14, 16)?, n(17, 19)?)
            .map(Taken::Clock);
    }
    None
}

/// PNG: an `eXIf` chunk (TIFF), or XMP in an `iTXt` chunk (iPhone screenshots).
fn png_date(r: &mut R, size: u64) -> Option<Taken> {
    let mut pos = 8;
    while pos + 8 <= size {
        let h = r.at(pos, 8);
        if h.len() < 8 {
            return None;
        }
        let (ln, typ) = (be32(&h)? as u64, &h[4..8]);
        if typ == b"eXIf"
            && let Some(d) = tiff_date(r, pos + 8) {
                return Some(d);
            }
        if (typ == b"iTXt" || typ == b"tEXt" || typ == b"zTXt") && ln < 4 << 20 {
            let blob = r.at(pos + 8, ln);
            if blob.starts_with(b"XML:com.adobe.xmp")
                && let Some(d) = xmp_date(&blob) {
                    return Some(d);
                }
        }
        if typ == b"IDAT" {
            return None;
        }
        pos += 12 + ln;
    }
    None
}

/// Photoshop: image resources `0x0422` (EXIF as TIFF) or `0x0424` (XMP).
fn psd_date(r: &mut R) -> Option<Taken> {
    let cm = be32(&r.at(26, 4))? as u64;
    let mut pos = 30 + cm;
    let end = pos + 4 + be32(&r.at(pos, 4))? as u64;
    pos += 4;
    let mut xmp = None;
    while pos + 10 < end {
        let h = r.at(pos, 8);
        if h.len() < 8 || &h[..4] != b"8BIM" {
            break;
        }
        let (rid, nl) = (be16(&h[4..])?, h[6] as u64);
        let p2 = pos + 6 + (nl + 2) / 2 * 2; // the name as a Pascal string, padded to even
        let ln = be32(&r.at(p2, 4))? as u64;
        if rid == 0x0422
            && let Some(d) = tiff_date(r, p2 + 4) {
                return Some(d);
            }
        if rid == 0x0424 && ln < 8 << 20 {
            xmp = xmp_date(&r.at(p2 + 4, ln));
        }
        pos = p2 + 4 + ln + (ln % 2);
    }
    xmp
}

const MONTHS: [&str; 12] = ["JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC"];

/// RIFF/AVI of old cameras: an `IDIT` chunk with `WED MAY 04 23:43:00 2005`.
fn avi_date(r: &mut R) -> Option<Taken> {
    let head = r.at(0, 1 << 20);
    let at = head.windows(4).position(|w| w == b"IDIT")?;
    let ln = u32e(head.get(at + 4..at + 8)?, true)? as usize;
    let start = at + 8;
    let txt = String::from_utf8_lossy(&head[start..(start + ln.min(64)).min(head.len())]).to_uppercase();
    let t = txt.as_bytes();
    // `<MON> +<d|dd> <HH>:<MM>:<SS> <YYYY>` at the first place it fits
    for i in 0..t.len() {
        let Some(mon) = t.get(i..i + 3) else { break };
        if !mon.iter().all(u8::is_ascii_uppercase) {
            continue;
        }
        let mut j = i + 3;
        let spaces = t[j..].iter().take_while(|&&c| c == b' ').count();
        if spaces == 0 {
            continue;
        }
        j += spaces;
        let digits = t[j..].iter().take_while(|c| c.is_ascii_digit()).count();
        if !(1..=2).contains(&digits) {
            continue;
        }
        let day: u32 = std::str::from_utf8(&t[j..j + digits]).ok()?.parse().ok()?;
        j += digits;
        let Some(rest) = t.get(j..j + 14) else { continue };
        let shape = rest.iter().enumerate().all(|(k, &c)| match k {
            0 | 9 => c == b' ',
            3 | 6 => c == b':',
            _ => c.is_ascii_digit(),
        });
        if !shape {
            continue;
        }
        let n = |a: usize, b: usize| std::str::from_utf8(&rest[a..b]).ok().and_then(|s| s.parse::<u32>().ok());
        let month = MONTHS.iter().position(|m| m.as_bytes() == mon)? as u32 + 1;
        return NaiveDate::from_ymd_opt(n(10, 14)? as i32, month, day)?.and_hms_opt(n(1, 3)?, n(4, 6)?, n(7, 9)?).map(Taken::Clock);
    }
    None
}

// ── ISOBMFF: MP4, MOV, HEIC, CR3 ─────────────────────────────────────────────

/// The boxes between `start` and `end`: (type, payload start, payload end).
fn boxes(r: &mut R, start: u64, end: u64) -> Vec<([u8; 4], u64, u64)> {
    let mut out = Vec::new();
    let mut pos = start;
    while pos + 8 <= end && out.len() < 10_000 {
        let h = r.at(pos, 16);
        if h.len() < 8 {
            break;
        }
        let mut size = be32(&h).unwrap_or(0) as u64;
        let typ: [u8; 4] = h[4..8].try_into().unwrap_or_default();
        let mut hlen = 8;
        if size == 1 {
            let Some(big) = be64(h.get(8..).unwrap_or_default()) else { break };
            size = big;
            hlen = 16;
        } else if size == 0 {
            size = end - pos;
        }
        if size < hlen {
            break;
        }
        let Some(next) = pos.checked_add(size) else { break };
        out.push((typ, pos + hlen, next));
        pos = next;
    }
    out
}

/// Follow a path of box types; `skip` gives per type how many header bytes (a full box) to skip.
fn find_box(r: &mut R, start: u64, end: u64, path: &[&[u8; 4]], skip: &[(&[u8; 4], u64)]) -> Option<(u64, u64)> {
    let Some((first, rest)) = path.split_first() else { return Some((start, end)) };
    let (_, p0, p1) = boxes(r, start, end).into_iter().find(|(t, _, _)| t == *first)?;
    let extra = skip.iter().find(|(t, _)| t == first).map(|(_, n)| *n).unwrap_or(0);
    find_box(r, p0 + extra, p1, rest, skip)
}

const CR3_UUID: [u8; 16] = [0x85, 0xc0, 0xb6, 0x87, 0x82, 0x0f, 0x11, 0xe0, 0x81, 0x11, 0xf4, 0xce, 0x46, 0x2b, 0x6a, 0x48];
const HEIF_BRANDS: &[&[u8; 4]] = &[b"heic", b"heix", b"hevc", b"mif1", b"msf1", b"heim", b"heis", b"avif"];

fn brand(r: &mut R, size: u64) -> Vec<u8> {
    find_box(r, 0, size, &[b"ftyp"], &[]).map(|(p0, _)| r.at(p0, 4)).unwrap_or_default()
}

/// Canon's own `uuid` box (in `moov`): `CMT2`/`CMT1` (CR3, TIFF) or `CNTH`/`CNDA` (MP4, a JPEG).
fn canon_uuid<T>(r: &mut R, start: u64, end: u64, tiff: Tiff<T>) -> Option<T> {
    for (typ, p0, p1) in boxes(r, start, end) {
        if &typ != b"uuid" || r.at(p0, 16) != CR3_UUID {
            continue;
        }
        let kids = boxes(r, p0 + 16, p1);
        for want in [b"CMT2", b"CMT1"] {
            for (t2, q0, _) in &kids {
                if t2 == want
                    && let Some(v) = tiff(r, *q0) {
                        return Some(v);
                    }
            }
        }
        for (t2, q0, q1) in &kids {
            if t2 == b"CNTH" {
                for (t3, s0, _) in boxes(r, *q0, *q1) {
                    if &t3 == b"CNDA" && r.at(s0, 2) == [0xFF, 0xD8] {
                        return jpeg(r, s0, tiff);
                    }
                }
            }
        }
    }
    None
}

fn cr3<T>(r: &mut R, size: u64, tiff: Tiff<T>) -> Option<T> {
    let (m0, m1) = find_box(r, 0, size, &[b"moov"], &[])?;
    canon_uuid(r, m0, m1, tiff)
}

/// Big-endian integer of `n` bytes (0 to 8) at `p`; none past the end.
fn int_at(r: &mut R, p: u64, n: u64) -> Option<u64> {
    if n == 0 {
        return Some(0);
    }
    if n > 8 {
        return None;
    }
    let b = r.at(p, n);
    (b.len() as u64 == n).then(|| b.iter().fold(0u64, |a, &x| (a << 8) | x as u64))
}

/// HEIC/HEIF: the `Exif` item, found through `iinf` and `iloc`.
fn heif<T>(r: &mut R, size: u64, tiff: Tiff<T>) -> Option<T> {
    let (meta0, m1) = find_box(r, 0, size, &[b"meta"], &[])?;
    let m0 = meta0 + 4; // full box
    let (i0, i1) = find_box(r, m0, m1, &[b"iinf"], &[])?;
    let ver = *r.at(i0, 1).first()?;
    let first = i0 + 4 + if ver == 0 { 2 } else { 4 };
    let mut exif_id = None;
    for (typ, p0, _) in boxes(r, first, i1) {
        if &typ != b"infe" {
            continue;
        }
        let v = *r.at(p0, 1).first()?;
        if v >= 2 {
            let body = r.at(p0 + 4, 8);
            let (id, kind) = if v == 2 {
                (be16(&body)? as u64, body.get(4..8)?.to_vec())
            } else {
                (be32(&body)? as u64, r.at(p0 + 10, 4))
            };
            if kind == b"Exif" {
                exif_id = Some(id);
                break;
            }
        }
    }
    let exif_id = exif_id?;
    let (l0, _) = find_box(r, m0, m1, &[b"iloc"], &[])?;
    let hdr = r.at(l0, 8);
    let ver = *hdr.first()?;
    let (offset_size, length_size) = ((*hdr.get(4)? >> 4) as u64, (*hdr.get(4)? & 0xF) as u64);
    let base_offset_size = (*hdr.get(5)? >> 4) as u64;
    let index_size = if ver == 1 || ver == 2 { (*hdr.get(5)? & 0xF) as u64 } else { 0 };
    let mut pos = l0 + 6;
    let id_size = if ver < 2 { 2 } else { 4 };
    let count = int_at(r, pos, id_size)?;
    pos += id_size;
    for _ in 0..count {
        let id = int_at(r, pos, id_size)?;
        pos += id_size;
        let mut construction = 0;
        if ver == 1 || ver == 2 {
            construction = int_at(r, pos, 2)? & 0xF;
            pos += 2;
        }
        pos += 2; // data_reference_index
        let base = int_at(r, pos, base_offset_size)?;
        pos += base_offset_size;
        let extents = int_at(r, pos, 2)?;
        pos += 2;
        let mut first_extent = None;
        for _ in 0..extents {
            if (ver == 1 || ver == 2) && index_size > 0 {
                pos += index_size;
            }
            let off = int_at(r, pos, offset_size)?;
            pos += offset_size;
            let ln = int_at(r, pos, length_size)?;
            pos += length_size;
            first_extent.get_or_insert((base.wrapping_add(off), ln));
        }
        if id == exif_id && construction == 0 {
            let (off, _) = first_extent?;
            let tiff_off = be32(&r.at(off, 4))? as u64;
            return tiff(r, off.checked_add(4 + tiff_off)?);
        }
    }
    None
}

/// Apple's `mdta` keys in `moov/meta` (`keys` and `ilst`), `com.apple.quicktime.make` …, those
/// in `only`, with values shorter than `max_len`; `meta` is a full box in an MP4 and not in a
/// MOV, so both are tried.
fn qt_keys(r: &mut R, moov: (u64, u64), only: &[&str], max_len: Option<u64>) -> HashMap<String, Vec<u8>> {
    let mut out = HashMap::new();
    for meta_skip in [0, 4] {
        let skip = [(b"meta", meta_skip)];
        let keys = find_box(r, moov.0, moov.1, &[b"meta", b"keys"], &skip);
        let ilst = find_box(r, moov.0, moov.1, &[b"meta", b"ilst"], &skip);
        let (Some(keys), Some(ilst)) = (keys, ilst) else { continue };
        let Some(n) = be32(&r.at(keys.0 + 4, 4)) else { continue };
        let mut names = HashMap::new();
        let mut pos = keys.0 + 8;
        for i in 1..=n.min(200) {
            let Some(klen) = be32(&r.at(pos, 4)) else { break };
            if klen < 8 {
                break;
            }
            names.insert(i, String::from_utf8_lossy(&r.at(pos + 8, klen as u64 - 8)).into_owned());
            pos += klen as u64;
        }
        for (typ, p0, p1) in boxes(r, ilst.0, ilst.1) {
            let Some(name) = names.get(&u32::from_be_bytes(typ)).filter(|n| only.contains(&n.as_str())) else { continue };
            let Some((d0, d1)) = find_box(r, p0, p1, &[b"data"], &[]) else { continue };
            let len = d1.saturating_sub(d0);
            if max_len.is_some_and(|m| len >= m) {
                continue;
            }
            out.insert(name.clone(), r.at(d0 + 8, len.saturating_sub(8)));
        }
        if !out.is_empty() {
            break;
        }
    }
    out
}

const APPLE_DATE: &str = "com.apple.quicktime.creationdate";
const APPLE_MAKE: &str = "com.apple.quicktime.make";
const APPLE_MODEL: &str = "com.apple.quicktime.model";

/// `2025-12-13T21:42:54+0100` (or `+01:00`, or `+01`), as Apple writes its creation date; none
/// when it isn't that shape, `Some(None)` when it is but the date doesn't exist.
fn apple_date(raw: &str) -> Option<Option<Taken>> {
    let s = raw.as_bytes();
    if s.len() < 22 {
        return None;
    }
    let shape = s[..19].iter().enumerate().all(|(k, &c)| match k {
        4 | 7 => c == b'-',
        10 => c == b'T',
        13 | 16 => c == b':',
        _ => c.is_ascii_digit(),
    }) && (s[19] == b'+' || s[19] == b'-')
        && s[20].is_ascii_digit()
        && s[21].is_ascii_digit();
    if !shape {
        return None;
    }
    let n = |a: usize, b: usize| raw.get(a..b).and_then(|t| t.parse::<i32>().ok()).unwrap_or(0);
    let mut j = 22;
    if s.get(j) == Some(&b':') {
        j += 1;
    }
    let minutes = if s.get(j..j + 2).is_some_and(|m| m.iter().all(u8::is_ascii_digit)) { n(j, j + 2) } else { 0 };
    let sign = if s[19] == b'+' { 1 } else { -1 };
    let secs = sign * (n(20, 22) * 3600 + minutes * 60);
    let ymd = [n(0, 4), n(5, 7), n(8, 10), n(11, 13), n(14, 16), n(17, 19)];
    Some(zoned(ymd, secs))
}

fn zoned([y, mo, d, h, mi, s]: [i32; 6], offset: i32) -> Option<Taken> {
    let clock = NaiveDate::from_ymd_opt(y, mo as u32, d as u32)?.and_hms_opt(h as u32, mi as u32, s as u32)?;
    FixedOffset::east_opt(offset)?.from_local_datetime(&clock).single().map(Taken::Instant)
}

fn quicktime_date(r: &mut R, size: u64) -> Option<Taken> {
    let moov = find_box(r, 0, size, &[b"moov"], &[])?;
    // Canon writes the camera's clock time in an embedded JPEG; it wins, as with exiftool
    if let Some(d) = canon_uuid(r, moov.0, moov.1, tiff_date) {
        return Some(d);
    }
    // Apple's own key gives local time with its zone; mvhd only UTC
    if let Some(raw) = qt_keys(r, moov, &[APPLE_DATE], None).get(APPLE_DATE)
        && let Some(parsed) = apple_date(&String::from_utf8_lossy(raw)) {
            return parsed;
        }
    let (h0, _) = find_box(r, moov.0, moov.1, &[b"mvhd"], &[])?;
    let ver = *r.at(h0, 1).first()?;
    let secs = if ver == 1 { be64(&r.at(h0 + 4, 8))? } else { be32(&r.at(h0 + 4, 4))? as u64 };
    if secs == 0 {
        return None;
    }
    const QT_EPOCH: i64 = -2_082_844_800; // 1904-01-01 in Unix time
    let unix = QT_EPOCH.checked_add(i64::try_from(secs).ok()?)?;
    DateTime::from_timestamp(unix, 0).map(|t| Taken::Instant(t.fixed_offset()))
}

fn quicktime_camera(r: &mut R, size: u64) -> Option<String> {
    let moov = find_box(r, 0, size, &[b"moov"], &[])?;
    let keys = qt_keys(r, moov, &[APPLE_MAKE, APPLE_MODEL], Some(512));
    let get = |k: &str| keys.get(k).map(|v| text(v)).unwrap_or_default();
    let (make, model) = (get(APPLE_MAKE), get(APPLE_MODEL));
    if !make.is_empty() || !model.is_empty() {
        return camera_name(&make, &model);
    }
    // GoPro: the model is in the GPMF metadata (key MINF); the make is GoPro, as in its JPEGs
    if let Some((g0, g1)) = find_box(r, moov.0, moov.1, &[b"udta", b"GPMF"], &[]) {
        let blob = r.at(g0, (g1 - g0).min(1 << 16));
        if let Some(at) = blob.windows(4).position(|w| w == b"MINF")
            && blob.len() >= at + 8 && blob[at + 4] == b'c' {
                let n = blob[at + 5] as usize * be16(&blob[at + 6..])? as usize;
                let model = text(&blob[at + 8..(at + 8 + n).min(blob.len())]);
                if !model.is_empty() {
                    return camera_name("GoPro", &model);
                }
            }
    }
    canon_uuid(r, moov.0, moov.1, tiff_camera) // Canon's MP4: the embedded JPEG
}

// ── per file ─────────────────────────────────────────────────────────────────

const ISOBMFF_EXT: &[&str] = &["mov", "mp4", "m4v", "heic", "heif", "cr3", "3gp"];

fn date_of(r: &mut R, size: u64, ext: &str) -> Option<Taken> {
    let head = r.at(0, 12);
    if head.starts_with(&[0xFF, 0xD8]) {
        return jpeg(r, 0, tiff_date);
    }
    if head.starts_with(b"\x89PNG\r\n\x1a\n") {
        return png_date(r, size);
    }
    if head.starts_with(b"RIFF") && head.get(8..12) == Some(b"AVI ") {
        return avi_date(r);
    }
    if head.starts_with(b"8BPS") {
        return psd_date(r);
    }
    if (head.starts_with(b"II") || head.starts_with(b"MM")) && PHOTO_EXT.contains(&ext) {
        return tiff_date(r, 0);
    }
    if head.get(4..8) == Some(b"ftyp") || ISOBMFF_EXT.contains(&ext) {
        let b = brand(r, size);
        if b == b"crx " {
            return cr3(r, size, tiff_date);
        }
        if HEIF_BRANDS.iter().any(|h| b == *h) {
            return heif(r, size, tiff_date);
        }
        return quicktime_date(r, size);
    }
    None
}

fn camera_of(r: &mut R, size: u64, ext: &str) -> Option<String> {
    let head = r.at(0, 12);
    if head.starts_with(&[0xFF, 0xD8]) {
        return jpeg(r, 0, tiff_camera);
    }
    if (head.starts_with(b"II") || head.starts_with(b"MM")) && PHOTO_EXT.contains(&ext) {
        return tiff_camera(r, 0);
    }
    if head.get(4..8) == Some(b"ftyp") || ISOBMFF_EXT.contains(&ext) {
        let b = brand(r, size);
        if b == b"crx " {
            return cr3(r, size, tiff_camera);
        }
        if HEIF_BRANDS.iter().any(|h| b == *h) {
            return heif(r, size, tiff_camera);
        }
        return quicktime_camera(r, size);
    }
    None
}

// ── the camera's name (data contract 3.5) ────────────────────────────────────

/// Makes that write a company instead of the brand.
const MAKE_ALIAS: &[(&str, &str)] = &[("eastman kodak", "KODAK"), ("asahi", "PENTAX")];

/// Where a company form at the end of `make` starts (`OLYMPUS OPTICAL CO.,LTD` → before
/// ` OPTICAL`); the same as `_MAKE_TAIL` in the NAS script.
fn company_tail(make: &str) -> Option<usize> {
    let l = make.to_ascii_lowercase();
    let b = l.as_bytes();
    let word = |i: usize| i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_');
    let strip = |s: &str| s.strip_suffix('.').map(|x| x.to_string()).unwrap_or_else(|| s.to_string());
    let mut k = None;
    for full in ["corporation", "company"] {
        if l.ends_with(full) {
            k = Some(l.len() - full.len());
        }
    }
    for short in ["corp", "inc"] {
        let t = strip(&l);
        if k.is_none() && t.ends_with(short) {
            k = Some(t.len() - short.len());
        }
    }
    if k.is_none() {
        let t = strip(&l);
        if let Some(before) = t.strip_suffix("ltd") {
            // `co` `.`? ws* `,`? ws* `ltd`
            let mut j = before.trim_end().len();
            if before[..j].ends_with(',') {
                j -= 1;
            }
            j = before[..j].trim_end().len();
            if before[..j].ends_with('.') {
                j -= 1;
            }
            if before[..j].ends_with("co") {
                k = Some(j - 2);
            }
        }
    }
    let k = k.filter(|&k| !word(k))?;
    let m1 = l[..k].trim_end().len();
    for w in ["optical", "imaging", "computer"] {
        if l[..m1].ends_with(w) {
            let w0 = m1 - w.len();
            let w1 = l[..w0].trim_end().len();
            if w1 < w0 {
                return Some(w1);
            }
        }
    }
    Some(m1)
}

fn short_make(make: &str) -> String {
    let mut m = make.to_string();
    for _ in 0..2 {
        if let Some(k) = company_tail(&m) {
            m.truncate(k);
        }
        m = m.trim_matches(|c| c == ' ' || c == ',').to_string();
    }
    let lower = m.to_lowercase();
    MAKE_ALIAS.iter().find(|(a, _)| *a == lower).map(|(_, b)| b.to_string()).unwrap_or(m)
}

/// The model without a trailing `ZOOM DIGITAL CAMERA` or `DIGITAL CAMERA`.
fn short_model(model: &str) -> String {
    let l = model.to_ascii_lowercase();
    for tail in [" zoom digital camera", " digital camera"] {
        if l.ends_with(tail) {
            return model[..model.len() - tail.len()].to_string();
        }
    }
    model.to_string()
}

/// Make and model joined, the make not repeated and without its company form:
/// `Canon` + `Canon EOS R` → `Canon EOS R`, `Apple` + `iPhone 13 Pro` → `Apple iPhone 13 Pro`.
pub fn camera_name(make: &str, model: &str) -> Option<String> {
    let make = short_make(&make.split_whitespace().collect::<Vec<_>>().join(" "));
    let model = short_model(&model.split_whitespace().collect::<Vec<_>>().join(" "));
    if model.is_empty() {
        return (!make.is_empty()).then_some(make);
    }
    let lm = model.to_lowercase();
    let first = make.split(' ').next().unwrap_or("").to_lowercase();
    if make.is_empty() || lm.starts_with(&make.to_lowercase()) || (!first.is_empty() && lm.starts_with(&first)) {
        return Some(model);
    }
    Some(format!("{make} {model}"))
}

// ── the order of data contract 3.4 ───────────────────────────────────────────

/// A date in the file name: `YYYY-MM-DD`, `YYYYMMDD` or `YYYY_MM_DD`, from 2000 up to next year.
pub fn date_from_name(path: &Path) -> Option<NaiveDateTime> {
    let name = path.file_name()?.to_string_lossy().into_owned();
    let b = name.as_bytes();
    let digit = |i: usize| b.get(i).is_some_and(u8::is_ascii_digit);
    let num = |i: usize, n: usize| -> Option<u32> { std::str::from_utf8(b.get(i..i + n)?).ok()?.parse().ok() };
    for i in 0..b.len() {
        if i > 0 && digit(i - 1) {
            continue;
        }
        if !(b[i..].starts_with(b"20") || b[i..].starts_with(b"19")) || !(digit(i + 2) && digit(i + 3)) {
            continue;
        }
        let mut j = i + 4;
        if b.get(j).is_some_and(|c| b"-_.".contains(c)) {
            j += 1;
        }
        let Some(month) = num(j, 2).filter(|m| (1..=12).contains(m) && digit(j) && digit(j + 1)) else { continue };
        j += 2;
        if b.get(j).is_some_and(|c| b"-_.".contains(c)) {
            j += 1;
        }
        let Some(day) = num(j, 2).filter(|d| (1..=31).contains(d) && digit(j) && digit(j + 1)) else { continue };
        if digit(j + 2) {
            continue;
        }
        let year = num(i, 4)? as i32;
        let dt = NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(0, 0, 0)?;
        let next = Local::now().format("%Y").to_string().parse::<i32>().unwrap_or(9999) + 1;
        return (2000..=next).contains(&year).then_some(dt);
    }
    None
}

/// The capture date of each file and where it came from (`exif`, `partner`, `name`, `mtime`), in
/// the order of data contract 3.4; for photos and videos only the file itself counts as `exif`.
pub fn resolve(paths: &[PathBuf]) -> HashMap<PathBuf, (Taken, &'static str)> {
    let mut out = HashMap::new();
    let mut by_stem: HashMap<(PathBuf, String), Taken> = HashMap::new();
    let key = |p: &Path| (p.parent().map(Path::to_path_buf).unwrap_or_default(), p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default());
    let mut unresolved = Vec::new();
    for p in paths {
        let kind = kind_of(p);
        match ((kind == "photo" || kind == "video").then(|| read(p).date)).flatten() {
            Some(d) => {
                by_stem.entry(key(p)).or_insert(d);
                out.insert(p.clone(), (d, "exif"));
            }
            None => unresolved.push(p),
        }
    }
    for p in unresolved {
        let found = if let Some(d) = by_stem.get(&key(p)) {
            (*d, "partner")
        } else if let Some(d) = date_from_name(p) {
            (Taken::Clock(d), "name")
        } else {
            let modified = std::fs::metadata(p).and_then(|m| m.modified()).ok();
            let t: DateTime<Local> = modified.map(Into::into).unwrap_or_else(Local::now);
            (Taken::Instant(t.fixed_offset()), "mtime")
        };
        out.insert(p.clone(), found);
    }
    out
}

#[cfg(test)]
pub(crate) mod fixtures {
    //! Small files of every format, built in the tests: no photos in the repository.

    pub fn bx(typ: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut v = ((8 + payload.len()) as u32).to_be_bytes().to_vec();
        v.extend_from_slice(typ);
        v.extend_from_slice(payload);
        v
    }

    /// A TIFF with ASCII tags in IFD0 and, if any, in an Exif IFD.
    pub fn tiff(le: bool, ifd0: &[(u16, &str)], exif: &[(u16, &str)]) -> Vec<u8> {
        let u16b = |x: u16| if le { x.to_le_bytes() } else { x.to_be_bytes() };
        let u32b = |x: u32| if le { x.to_le_bytes() } else { x.to_be_bytes() };
        let n0 = ifd0.len() + usize::from(!exif.is_empty());
        let ifd0_size = 2 + 12 * n0 + 4;
        let exif_size = if exif.is_empty() { 0 } else { 2 + 12 * exif.len() + 4 };
        let data_start = 8 + ifd0_size + exif_size;
        let mut data = Vec::new();
        let entry = |tag: u16, s: &str, data: &mut Vec<u8>| {
            let mut v = s.as_bytes().to_vec();
            v.push(0);
            let mut e = u16b(tag).to_vec();
            e.extend(u16b(2));
            e.extend(u32b(v.len() as u32));
            if v.len() <= 4 {
                v.resize(4, 0);
                e.extend(v);
            } else {
                e.extend(u32b((data_start + data.len()) as u32));
                data.extend(v);
            }
            e
        };
        let mut out = if le { b"II*\0".to_vec() } else { b"MM\0*".to_vec() };
        out.extend(u32b(8));
        out.extend(u16b(n0 as u16));
        for (t, s) in ifd0 {
            out.extend(entry(*t, s, &mut data));
        }
        if !exif.is_empty() {
            out.extend(u16b(0x8769));
            out.extend(u16b(4));
            out.extend(u32b(1));
            out.extend(u32b((8 + ifd0_size) as u32));
        }
        out.extend(u32b(0));
        if !exif.is_empty() {
            out.extend(u16b(exif.len() as u16));
            for (t, s) in exif {
                out.extend(entry(*t, s, &mut data));
            }
            out.extend(u32b(0));
        }
        out.extend(data);
        out
    }

    /// A JPEG with an APP0 segment first and the TIFF in APP1.
    pub fn jpeg_file(tiff: &[u8]) -> Vec<u8> {
        let mut v = vec![0xFF, 0xD8, 0xFF, 0xE0, 0, 16];
        v.extend(b"JFIF\0\x01\x01\0\0\x01\0\x01\0\0");
        let mut app1 = b"Exif\0\0".to_vec();
        app1.extend_from_slice(tiff);
        v.extend([0xFF, 0xE1]);
        v.extend(((app1.len() + 2) as u16).to_be_bytes());
        v.extend(app1);
        v.extend([0xFF, 0xDA, 0, 2, 0xFF, 0xD9]);
        v
    }

    pub const CANON: &[(u16, &str)] = &[(0x010F, "Canon"), (0x0110, "Canon EOS R"), (0x0132, "2026:07:30 16:58:12")];
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;
    use super::*;

    fn file(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, bytes).unwrap();
        p
    }

    fn clock(s: &str) -> Option<Taken> {
        Some(Taken::Clock(NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()))
    }

    fn mdta(keys: &[(&str, &str)]) -> Vec<u8> {
        let mut k = vec![0, 0, 0, 0];
        k.extend((keys.len() as u32).to_be_bytes());
        let mut ilst = Vec::new();
        for (i, (name, value)) in keys.iter().enumerate() {
            k.extend(((8 + name.len()) as u32).to_be_bytes());
            k.extend(b"mdta");
            k.extend(name.as_bytes());
            let mut data = vec![0, 0, 0, 1, 0, 0, 0, 0];
            data.extend(value.as_bytes());
            ilst.extend(bx(&((i + 1) as u32).to_be_bytes(), &bx(b"data", &data)));
        }
        [bx(b"keys", &k), bx(b"ilst", &ilst)].concat()
    }

    fn mvhd(secs: u32) -> Vec<u8> {
        let mut p = vec![0, 0, 0, 0];
        p.extend(secs.to_be_bytes());
        p.extend([0; 92]);
        bx(b"mvhd", &p)
    }

    fn heic(tiff: &[u8]) -> Vec<u8> {
        let ftyp = bx(b"ftyp", b"heic\0\0\0\0mif1heic");
        let mut infe = vec![2, 0, 0, 0];
        infe.extend(1u16.to_be_bytes());
        infe.extend(0u16.to_be_bytes());
        infe.extend(b"hvc1\0");
        let mut infe2 = vec![2, 0, 0, 0];
        infe2.extend(7u16.to_be_bytes());
        infe2.extend(0u16.to_be_bytes());
        infe2.extend(b"Exif\0");
        let mut iinf = vec![0, 0, 0, 0];
        iinf.extend(2u16.to_be_bytes());
        iinf.extend(bx(b"infe", &infe));
        iinf.extend(bx(b"infe", &infe2));
        let mut payload = 6u32.to_be_bytes().to_vec();
        payload.extend(b"Exif\0\0");
        payload.extend_from_slice(tiff);
        let build = |offset: u32| {
            let mut iloc = vec![0, 0, 0, 0, 0x44, 0x00];
            iloc.extend(2u16.to_be_bytes());
            for (id, off) in [(1u16, 0u32), (7, offset)] {
                iloc.extend(id.to_be_bytes());
                iloc.extend(0u16.to_be_bytes());
                iloc.extend(1u16.to_be_bytes());
                iloc.extend(off.to_be_bytes());
                iloc.extend((payload.len() as u32).to_be_bytes());
            }
            let mut meta = vec![0, 0, 0, 0];
            meta.extend(bx(b"hdlr", &[0; 24]));
            meta.extend(bx(b"iinf", &iinf));
            meta.extend(bx(b"iloc", &iloc));
            [ftyp.clone(), bx(b"meta", &meta)].concat()
        };
        let head = build(0);
        let offset = (head.len() + 8) as u32;
        [build(offset), bx(b"mdat", &payload)].concat()
    }

    #[test]
    fn jpeg_tiff_and_the_fallbacks_of_the_date() {
        let d = tempfile::tempdir().unwrap();
        let both = tiff(true, CANON, &[(0x9003, "2026:07:30 10:00:01"), (0x9004, "2026:07:30 10:00:02")]);
        let c = read(&file(d.path(), "a.JPG", &jpeg_file(&both)));
        assert_eq!(c.date, clock("2026-07-30 10:00:01"));
        assert_eq!(c.camera.as_deref(), Some("Canon EOS R"));
        let digitized = tiff(false, CANON, &[(0x9004, "2026:07:30 10:00:02")]);
        assert_eq!(read(&file(d.path(), "b.NEF", &digitized)).date, clock("2026-07-30 10:00:02"));
        let only = tiff(true, CANON, &[]);
        assert_eq!(read(&file(d.path(), "c.dng", &only)).date, clock("2026-07-30 16:58:12"));
        // short make in the entry itself, year 0000, a TIFF that isn't a photo by its name
        let odd = tiff(true, &[(0x010F, "Ab"), (0x0110, "X1"), (0x0132, "0000:00:00 00:00:00")], &[]);
        let c = read(&file(d.path(), "d.tif", &odd));
        assert_eq!((c.date, c.camera.as_deref()), (None, Some("Ab X1")));
        assert_eq!(read(&file(d.path(), "e.bin", &only)), Capture::default());
    }

    #[test]
    fn png_psd_and_avi() {
        let d = tempfile::tempdir().unwrap();
        let chunk = |typ: &[u8; 4], data: &[u8]| {
            let mut v = (data.len() as u32).to_be_bytes().to_vec();
            v.extend(typ);
            v.extend(data);
            v.extend([0; 4]);
            v
        };
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        png.extend(chunk(b"IHDR", &[0; 13]));
        png.extend(chunk(b"iTXt", b"XML:com.adobe.xmp\0\0\0\0\0<x exif:DateTimeOriginal=\"2019-05-04T10:11:12\"/>"));
        png.extend(chunk(b"IDAT", &[0; 4]));
        assert_eq!(read(&file(d.path(), "s.PNG", &png)).date, clock("2019-05-04 10:11:12"));
        let mut png2 = b"\x89PNG\r\n\x1a\n".to_vec();
        png2.extend(chunk(b"eXIf", &tiff(false, CANON, &[])));
        assert_eq!(read(&file(d.path(), "t.png", &png2)).date, clock("2026-07-30 16:58:12"));

        let t = tiff(false, &[], &[(0x9003, "2011:02:03 04:05:06")]);
        let resource = |id: u16, data: &[u8]| {
            let mut v = b"8BIM".to_vec();
            v.extend(id.to_be_bytes());
            v.extend([0, 0]); // empty name, padded
            v.extend((data.len() as u32).to_be_bytes());
            v.extend(data);
            if data.len() % 2 == 1 {
                v.push(0);
            }
            v
        };
        let res = [resource(0x0424, b"<x xmp:CreateDate=\"2001-01-01T01:01:01\"/>"), resource(0x0422, &t)].concat();
        let mut psd = b"8BPS".to_vec();
        psd.resize(26, 0);
        psd.extend(0u32.to_be_bytes());
        psd.extend((res.len() as u32).to_be_bytes());
        psd.extend(res);
        assert_eq!(read(&file(d.path(), "p.psd", &psd)).date, clock("2011-02-03 04:05:06"));

        let mut avi = b"RIFF\0\0\0\0AVI LIST".to_vec();
        let idit = b"WED MAY 04 23:43:00 2005\n\0";
        avi.extend(b"IDIT");
        avi.extend((idit.len() as u32).to_le_bytes());
        avi.extend(idit);
        assert_eq!(read(&file(d.path(), "v.AVI", &avi)).date, clock("2005-05-04 23:43:00"));
    }

    #[test]
    fn cr3_heic_and_canon_mp4() {
        let d = tempfile::tempdir().unwrap();
        let uuid = |kids: &[u8]| bx(b"uuid", &[CR3_UUID.to_vec(), kids.to_vec()].concat());
        let cr3 = [bx(b"ftyp", b"crx \0\0\0\x01crx isom"), bx(b"moov", &uuid(&bx(b"CMT1", &tiff(true, CANON, &[]))))].concat();
        let c = read(&file(d.path(), "x.CR3", &cr3));
        assert_eq!((c.date, c.camera.as_deref()), (clock("2026-07-30 16:58:12"), Some("Canon EOS R")));

        let h = heic(&tiff(false, &[(0x010F, "Apple"), (0x0110, "iPhone 14 Pro")], &[(0x9003, "2026:08:02 13:34:23")]));
        let c = read(&file(d.path(), "y.HEIC", &h));
        assert_eq!((c.date, c.camera.as_deref()), (clock("2026-08-02 13:34:23"), Some("Apple iPhone 14 Pro")));

        let thumb = jpeg_file(&tiff(true, CANON, &[]));
        let mp4 = [bx(b"ftyp", b"isom\0\0\0\0isom"), bx(b"moov", &[mvhd(1), uuid(&bx(b"CNTH", &bx(b"CNDA", &thumb)))].concat())].concat();
        let c = read(&file(d.path(), "z.MP4", &mp4));
        assert_eq!((c.date, c.camera.as_deref()), (clock("2026-07-30 16:58:12"), Some("Canon EOS R")));
    }

    #[test]
    fn apple_keys_mvhd_and_gopro() {
        let d = tempfile::tempdir().unwrap();
        let keys = mdta(&[("com.apple.quicktime.make", "Apple"), ("com.apple.quicktime.model", "iPhone 14 Pro"), ("com.apple.quicktime.creationdate", "2025-12-13T21:42:54+0100")]);
        let mov = [bx(b"ftyp", b"qt  \0\0\0\0qt  "), bx(b"moov", &[mvhd(100), bx(b"meta", &[bx(b"hdlr", &[0; 24]), keys.clone()].concat())].concat())].concat();
        let c = read(&file(d.path(), "IMG_0001.MOV", &mov));
        let want = FixedOffset::east_opt(3600).unwrap().with_ymd_and_hms(2025, 12, 13, 21, 42, 54).unwrap();
        assert_eq!(c.date, Some(Taken::Instant(want)));
        assert_eq!(c.date.unwrap().clock().to_string(), "2025-12-13 21:42:54");
        assert_eq!(c.camera.as_deref(), Some("Apple iPhone 14 Pro"));
        // an MP4's meta is a full box
        let mp4 = [bx(b"ftyp", b"mp42\0\0\0\0mp42"), bx(b"moov", &bx(b"meta", &[vec![0, 0, 0, 0], keys].concat()))].concat();
        assert_eq!(read(&file(d.path(), "b.mp4", &mp4)).camera.as_deref(), Some("Apple iPhone 14 Pro"));

        // only mvhd: UTC seconds since 1904
        let secs = (2_082_844_800u64 + 1_785_000_000) as u32;
        let plain = [bx(b"ftyp", b"isom\0\0\0\0isom"), bx(b"moov", &mvhd(secs))].concat();
        let c = read(&file(d.path(), "c.MP4", &plain));
        assert_eq!(c.date, Some(Taken::Instant(DateTime::from_timestamp(1_785_000_000, 0).unwrap().fixed_offset())));
        assert_eq!(c.camera, None);
        let zero = [bx(b"ftyp", b"isom\0\0\0\0isom"), bx(b"moov", &mvhd(0))].concat();
        assert_eq!(read(&file(d.path(), "d.MP4", &zero)).date, None);

        let mut gpmf = b"DEVC\0\0\0\0MINF".to_vec();
        gpmf.extend([b'c', 1]);
        gpmf.extend(12u16.to_be_bytes());
        gpmf.extend(b"HERO7 Black\0");
        let gopro = [bx(b"ftyp", b"mp41\0\0\0\0mp41"), bx(b"moov", &[mvhd(secs), bx(b"udta", &bx(b"GPMF", &gpmf))].concat())].concat();
        assert_eq!(read(&file(d.path(), "GX010001.MP4", &gopro)).camera.as_deref(), Some("GoPro HERO7 Black"));
    }

    #[test]
    fn a_broken_file_gives_nothing_and_never_panics() {
        let d = tempfile::tempdir().unwrap();
        let keys = mdta(&[("com.apple.quicktime.creationdate", "2025-12-13T21:42:54+0100")]);
        let samples = [
            ("a.jpg", jpeg_file(&tiff(true, CANON, &[(0x9003, "2026:07:30 10:00:01")]))),
            ("b.heic", heic(&tiff(false, CANON, &[]))),
            ("c.mov", [bx(b"ftyp", b"qt  "), bx(b"moov", &[mvhd(5), bx(b"meta", &keys)].concat())].concat()),
            ("d.cr3", [bx(b"ftyp", b"crx "), bx(b"moov", &bx(b"uuid", &[CR3_UUID.to_vec(), bx(b"CMT1", &tiff(true, CANON, &[]))].concat()))].concat()),
            ("e.nef", tiff(false, CANON, &[(0x9003, "2026:07:30 10:00:01")])),
        ];
        for (name, bytes) in samples {
            for n in 0..bytes.len() {
                let _ = read(&file(d.path(), name, &bytes[..n]));
            }
            // garbage in every byte, one at a time
            for i in 0..bytes.len() {
                let mut b = bytes.clone();
                b[i] = b[i].wrapping_add(0x80);
                let _ = read(&file(d.path(), name, &b));
            }
        }
        assert_eq!(read(&d.path().join("missing.jpg")), Capture::default());
    }

    #[test]
    fn camera_names_as_the_nas_script() {
        let cases = [
            ("Canon", "Canon EOS R", Some("Canon EOS R")),
            ("Apple", "iPhone 13 Pro", Some("Apple iPhone 13 Pro")),
            ("NIKON CORPORATION", "NIKON D600", Some("NIKON D600")),
            ("  GoPro ", "HERO7  Black ", Some("GoPro HERO7 Black")),
            ("", "HERO7 Black", Some("HERO7 Black")),
            ("Apple", "", Some("Apple")),
            ("", "", None),
            ("EASTMAN KODAK COMPANY", "KODAK LS633 ZOOM DIGITAL CAMERA", Some("KODAK LS633")),
            ("Asahi Optical Co.,Ltd", "PENTAX Optio 430", Some("PENTAX Optio 430")),
            ("OLYMPUS OPTICAL CO.,LTD", "C5050Z", Some("OLYMPUS C5050Z")),
            ("CASIO COMPUTER CO.,LTD.", "EX-Z750", Some("CASIO EX-Z750")),
            ("OLYMPUS IMAGING CORP.", "E-M5", Some("OLYMPUS E-M5")),
            ("SAMSUNG TECHWIN CO., LTD.", "Digimax A6", Some("SAMSUNG TECHWIN Digimax A6")),
            ("Minolta Co., Ltd.", "DiMAGE Z1", Some("Minolta DiMAGE Z1")),
            ("Canon", "Canon EOS 350D DIGITAL", Some("Canon EOS 350D DIGITAL")),
            ("KONICA MINOLTA", "DiMAGE Z6", Some("KONICA MINOLTA DiMAGE Z6")),
            ("Acme Inc.", "Cam 1", Some("Acme Cam 1")),
            ("Telco", "T1", Some("Telco T1")), // `co` inside a word is not a company
        ];
        for (make, model, want) in cases {
            assert_eq!(camera_name(make, model).as_deref(), want, "{make} / {model}");
        }
    }

    #[test]
    fn the_order_of_the_contract() {
        let d = tempfile::tempdir().unwrap();
        let photo = file(d.path(), "IMG_0001.JPG", &jpeg_file(&tiff(true, CANON, &[])));
        let clip = file(d.path(), "IMG_0001.MOV", b"no date in here");
        let named = file(d.path(), "WhatsApp Video 2025-09-20 at 10.48.26.mp4", b"x");
        let plain = file(d.path(), "notes.txt", b"x");
        let r = resolve(&[photo.clone(), clip.clone(), named.clone(), plain.clone()]);
        assert_eq!(r[&photo], (clock("2026-07-30 16:58:12").unwrap(), "exif"));
        assert_eq!(r[&clip], (clock("2026-07-30 16:58:12").unwrap(), "partner"));
        assert_eq!(r[&named], (clock("2025-09-20 00:00:00").unwrap(), "name"));
        assert_eq!(r[&plain].1, "mtime");
    }

    #[test]
    fn dates_in_names() {
        let t = |n: &str| date_from_name(Path::new(n)).map(|d| d.date().to_string());
        assert_eq!(t("2026-08-02_13-34-23_IMG_6065.JPG").as_deref(), Some("2026-08-02"));
        assert_eq!(t("VID_20260803_120000.mp4").as_deref(), Some("2026-08-03"));
        assert_eq!(t("scan_2012_07_15.tif").as_deref(), Some("2012-07-15"));
        assert_eq!(t("IMG_20261340.jpg"), None); // month 13
        assert_eq!(t("120260803.jpg"), None); // a digit before
        assert_eq!(t("1999-05-05.jpg"), None); // before 2000
        assert_eq!(t("2026-02-30.jpg"), None); // no such day
        assert_eq!(t("IMG_0955.JPG"), None);
    }

    #[test]
    fn kinds_and_the_manifest_time() {
        let k = |n: &str| kind_of(Path::new(n));
        assert_eq!([k("a.CR3"), k("b.mp4"), k("c.xmp"), k("d.AAE"), k("e.txt"), k("f.LRV")], ["photo", "video", "sidecar", "sidecar", "other", "video"]);
        let t = clock("2026-07-30 16:58:12").unwrap().manifest_time();
        assert!(t.starts_with("2026-07-30T16:58:12") && (t.ends_with(":00") || t.ends_with(":30") || t.ends_with(":45")), "{t}");
        let i = Taken::Instant(DateTime::from_timestamp(0, 0).unwrap().fixed_offset()).manifest_time();
        assert_eq!(DateTime::parse_from_rfc3339(&i).unwrap().timestamp(), 0);
    }
}
