//! Prints what the app would do for the previews (F012/S02), to try the plan on real data:
//!
//!     cargo run --example previews-plan -- <index> <record> <previews> [<nas previews>]
//!
//! Adopted previews are written to `<record>`; point it at a scratch file to leave the app's own alone.

fn main() {
    use family_lightbox_lib::previews_plan::{Record, State, plan};
    use std::path::PathBuf;

    let args: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
    if args.len() < 3 {
        eprintln!("usage: previews-plan <index> <record> <previews> [<nas previews>]");
        std::process::exit(2);
    }
    let mut record = Record::open(&args[1]).unwrap_or_else(|e| panic!("{e}"));
    let p = plan(&args[0], &mut record, &args[2], args.get(3).map(|p| p.as_path()), None).unwrap_or_else(|e| panic!("{e}"));
    println!("{:?} in {} ms", p.counts, p.millis);
    let mut months: Vec<(String, usize)> = Vec::new();
    for w in p.todo.iter().filter(|w| w.state == State::Make) {
        match months.last_mut() {
            Some((m, n)) if *m == w.month => *n += 1,
            _ => months.push((w.month.clone(), 1)),
        }
    }
    println!("{} months to make, newest first: {:?}", months.len(), &months[..months.len().min(5)]);
}
