//! Reads the NAS's layout as the app does, into a temporary config folder, and prints the roots
//! (F019): cargo run --example layout-check -- <nas host> <home share>

use family_lightbox_lib::{config, layout, mounts};

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let tmp = std::env::temp_dir().join(format!("layout-check-{}", std::process::id()));
    let raw = mounts::system_mounts();
    println!("source: {:?}", layout::refresh(&tmp, &raw, &a[0], &a[1]));
    let settings = config::Settings { nas_host: a[0].clone(), layout: layout::read_copy(&tmp), ..Default::default() };
    let l = settings.effective_layout();
    println!("name: {:?}", l.name);
    let found = mounts::find_shares(&raw, &settings);
    for (what, list) in [("library", &found.library), ("previews", &found.previews), ("lists", &found.lists), ("exports", &found.exports)] {
        println!("{what:9} → {:?}", mounts::NasMounts::chosen(list).map(|m| m.mount_point.clone()));
    }
    let _ = std::fs::remove_dir_all(&tmp);
}
