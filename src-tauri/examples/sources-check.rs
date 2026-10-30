//! The layouts the import reads from an index, and the source it would propose for a camera
//! (F018/S03); reads only:  cargo run --example sources-check -- <index> [<camera>…]

use family_lightbox_lib::{db::Db, sources};

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let db = Db::open(a[0].as_ref()).unwrap();
    let paths = db.source_paths().unwrap();
    let layouts = sources::layouts(&paths);
    let mut counts: std::collections::HashMap<&str, usize> = Default::default();
    for (s, _) in &paths {
        *counts.entry(s).or_default() += 1;
    }
    let mut list: Vec<_> = layouts.iter().collect();
    list.sort_by_key(|(s, _)| std::cmp::Reverse(counts[s.as_str()]));
    for (s, l) in list.iter().take(8) {
        println!("{:>6}  split={:<5} day={:<5} {s}", counts[s.as_str()], l.split, l.day);
    }
    for camera in &a[1..] {
        println!("{camera} → {}", sources::propose(camera, &db.sources_of_camera(camera).unwrap(), &layouts));
    }
}
