//! Pushes one year subfolder to the Library share as the import will (F018/S02). It writes to
//! the share: run it only on purpose.
//!
//!     cargo run --release --example push-folder -- <library> <share> <folder> <device>

use family_lightbox_lib::{manifest, push};
use std::path::Path;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() != 4 {
        eprintln!("usage: push-folder <library> <share> <folder> <device>");
        std::process::exit(2);
    }
    let (local, share, folder, device) = (Path::new(&a[0]), Path::new(&a[1]), &a[2], &a[3]);
    let existing = manifest::read(&share.join(folder).join(manifest::MANIFEST_NAME));
    let m = manifest::build(local, folder, device, existing.as_ref()).unwrap_or_else(|e| panic!("{e}"));
    let t = std::time::Instant::now();
    let out = push::push_folder(local, share, &m).unwrap_or_else(|e| panic!("{e}"));
    println!(
        "{folder}: copied {}, there {}, verified {}, missing {}, conflicts {}, failed {}, manifest rows {}, in {} ms",
        out.copied, out.there, out.ok, out.missing.len(), out.conflicts.len(), out.failed.len(), out.rows, t.elapsed().as_millis()
    );
    for p in out.missing.iter().chain(&out.conflicts).chain(&out.failed).take(20) {
        println!("  {p}");
    }
}
