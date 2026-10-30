//! Builds the app's index into a file of choice, to try the builder on real data:
//!
//!     cargo run --release --example index-build -- <working copy> <previews> <target.sqlite>

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() != 3 {
        eprintln!("usage: index-build <working copy> <previews> <target.sqlite>");
        std::process::exit(2);
    }
    let out = family_lightbox_lib::index::build(a[0].as_ref(), a[1].as_ref(), a[2].as_ref()).unwrap_or_else(|e| panic!("{e}"));
    println!("{} items from {} manifests in {} ms", out.items, out.manifests, out.millis);
}
