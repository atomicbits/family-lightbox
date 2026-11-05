//! Renders one original at a long side (F024/S01): `cargo run --example render-one -- <src> <out> <long|0>`.
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let long: usize = a[2].parse().unwrap();
    let t = std::time::Instant::now();
    family_lightbox_lib::preview::render(a[0].as_ref(), (long > 0).then_some(long), 0.92, Some(300), Some("2026-07-30T16:58:12+02:00"), Some("test"), a[1].as_ref()).unwrap_or_else(|e| panic!("{e}"));
    println!("{} in {:?}", a[1], t.elapsed());
}
