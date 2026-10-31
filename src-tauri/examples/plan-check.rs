//! The plan of an import, without copying anything (F018/S04), to compare with the CLI's dry run:
//!
//!     cargo run --release --example plan-check -- <index> <cache.sqlite> <folder> <source>

use family_lightbox_lib::{db::Db, import_plan, sources};
use std::path::Path;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let db = Db::open(a[0].as_ref()).unwrap();
    let layouts = sources::layouts(&db.source_paths().unwrap());
    let layout = layouts.get(&a[3]).copied().unwrap_or(sources::NEW);
    let known = db.path_hashes().unwrap();
    let cache = import_plan::HashCache::open(a[1].as_ref()).unwrap();
    let t = std::time::Instant::now();
    let p = import_plan::plan(Path::new(&a[2]), &a[3], layout, &[], &known, &cache, &mut |_| {});
    println!(
        "{}: {} to copy ({:.1} GB), {} already in the library, {} conflicts, {} left out, in {:.1} s",
        a[3], p.copy_files, p.copy_bytes as f64 / 1e9, p.duplicates, p.conflicts, p.excluded, t.elapsed().as_secs_f64()
    );
    println!("  months {:?}", p.months);
    println!("  dates not from the file {:?}", p.not_from_file);
    for f in p.files.iter().take(3) {
        println!("  {:?} {} → {}", f.action, f.src.rsplit('/').next().unwrap_or(""), f.target);
    }
    for e in p.problems.iter().take(3) {
        println!("  problem: {e}");
    }
}
