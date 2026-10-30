//! Times the content hash of one file (F018/S01):  cargo run --release --example hash-speed -- <file>

fn main() {
    let p = std::env::args().nth(1).expect("usage: hash-speed <file>");
    let t = std::time::Instant::now();
    let h = family_lightbox_lib::hash::sha256_file(p.as_ref()).unwrap();
    let secs = t.elapsed().as_secs_f64();
    let mb = std::fs::metadata(&p).unwrap().len() as f64 / 1e6;
    println!("{h}  {:.0} MB/s", mb / secs);
}
