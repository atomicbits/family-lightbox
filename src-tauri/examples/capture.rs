//! Prints what the import reads from files (F018/S01), to compare with exiftool or the NAS script:
//!
//!     cargo run --release --example capture -- <file>...
//!
//! One line per file: path, the date as the manifest writes it, the clock time, the camera.

use family_lightbox_lib::capture;

fn main() {
    for a in std::env::args().skip(1) {
        let c = capture::read(a.as_ref());
        let date = c.date.map(|d| format!("{}\t{}", d.manifest_time(), d.clock().format("%Y:%m:%d %H:%M:%S"))).unwrap_or_else(|| "-\t-".into());
        println!("{a}\t{date}\t{}", c.camera.unwrap_or_else(|| "-".into()));
    }
}
