//! Prints the merged lists (F003/S01), to try the reading on real data:
//!
//!     cargo run --example lists -- <Lists folder>

fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        eprintln!("usage: lists <Lists folder>");
        std::process::exit(2);
    };
    let (state, _) = family_lightbox_lib::lists::load(Some(std::path::Path::new(&dir)), std::path::Path::new("/nonexistent"));
    println!("{}", serde_json::to_string_pretty(&state).unwrap());
}
