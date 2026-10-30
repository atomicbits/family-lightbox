//! Times a filter over a real index (F002/S02):  cargo run --release --example filter-speed -- <index>

fn main() {
    let index = std::env::args().nth(1).expect("usage: filter-speed <index>");
    let db = family_lightbox_lib::db::Db::open(index.as_ref()).unwrap();
    let f = family_lightbox_lib::filter::Filter { kinds: vec!["live".into()], ..Default::default() };
    let t = std::time::Instant::now();
    let months = db.filtered_months(&f).unwrap();
    let n: i64 = months.iter().map(|(_, c)| c).sum();
    println!("{} months, {n} Live Photos, in {} ms", months.len(), t.elapsed().as_millis());
}
