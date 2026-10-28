//! Makes the preview and thumbnail of chosen originals with the app's own code (F012/S01), to
//! compare them with `media previews`:
//!
//!     cargo run --example make-preview -- <folder> <original> [--taken 2026-07-30T16:58:12+02:00] ...
//!
//! Writes `<folder>/<name>.jpg` and `<folder>/<name>.t.jpg` per original.

#[cfg(target_os = "macos")]
fn main() {
    use family_lightbox_lib::preview::{Kind, make};
    use std::path::{Path, PathBuf};

    const VIDEO: [&str; 8] = ["mp4", "mov", "m4v", "avi", "mts", "3gp", "360", "insv"];
    let mut args = std::env::args().skip(1);
    let Some(folder) = args.next().map(PathBuf::from) else {
        eprintln!("usage: make-preview <folder> <original> [--taken <taken>] ...");
        std::process::exit(2);
    };
    let mut taken: Option<String> = None;
    let mut failed = false;
    while let Some(arg) = args.next() {
        if arg == "--taken" {
            taken = args.next();
            continue;
        }
        let src = Path::new(&arg);
        let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let kind = if VIDEO.contains(&ext.as_str()) { Kind::Video } else { Kind::Photo };
        let name = src.file_name().unwrap().to_string_lossy();
        let (out, thumb) = (folder.join(format!("{name}.jpg")), folder.join(format!("{name}.t.jpg")));
        let t0 = std::time::Instant::now();
        match make(src, kind, taken.as_deref(), &out, &thumb) {
            Ok(()) => println!("{name}: ok in {:.0} ms", t0.elapsed().as_secs_f64() * 1000.0),
            Err(e) => {
                println!("{name}: {e}");
                failed = true;
            }
        }
    }
    std::process::exit(failed as i32);
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("make-preview is macOS only");
}
