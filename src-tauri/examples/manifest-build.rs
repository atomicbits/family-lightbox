//! Prints the manifest the app would build for a year subfolder, without writing anything
//! (F018/S02), to compare with the one the CLI or the NAS made:
//!
//!     cargo run --release --example manifest-build -- <library> <folder> [<existing manifest>]
//!
//! With `MANIFEST_VERIFY=<share>` it reads the share and only says what a push would find there.

use family_lightbox_lib::manifest;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() < 2 {
        eprintln!("usage: manifest-build <library> <folder> [<existing manifest>]");
        std::process::exit(2);
    }
    let existing = a.get(2).and_then(|p| manifest::read(p.as_ref()));
    let m = manifest::build(a[0].as_ref(), &a[1], "example", existing.as_ref()).unwrap_or_else(|e| panic!("{e}"));
    if let Ok(share) = std::env::var("MANIFEST_VERIFY") {
        let share = std::path::Path::new(&share);
        let sizes = manifest::sizes(share, &a[1]);
        let remote = manifest::read(&share.join(&a[1]).join(manifest::MANIFEST_NAME));
        let v = manifest::verify(&m, remote.as_ref(), &sizes);
        let merged = manifest::merge(&m, remote.as_ref(), &v.ok, &sizes);
        println!("on the share {}, verified {}, missing {}, conflicts {}, merged rows {}", sizes.len(), v.ok.len(), v.missing.len(), v.conflicts.len(), merged["files"].as_array().map_or(0, Vec::len));
        return;
    }
    print!("{}", manifest::to_text(&m));
}
