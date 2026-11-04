//! The dry run of a rename (F022/S01), read only: `cargo run --example rename-plan -- <Library>
//! <Previews> <to> <from>...`.
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() < 4 {
        eprintln!("gebruik: rename-plan <Library> <Previews> <nieuwe naam> <bron>...");
        std::process::exit(2);
    }
    let p = family_lightbox_lib::rename::plan(a[0].as_ref(), a[1].as_ref(), &a[3..], &a[2]).unwrap_or_else(|e| panic!("{e}"));
    for m in &p.months {
        let days: std::collections::BTreeSet<String> = m.moves.iter().filter_map(|x| x.to.rsplit_once('/').map(|(d, _)| d.to_string())).collect();
        println!("{}: {} te verplaatsen ({} sidecars, {} previews), {} dubbel, {} naamconflict · naar {:?}", m.month, m.moves.len(), m.moves.iter().filter(|x| x.sidecar).count(), m.moves.iter().filter(|x| x.preview).count(), m.duplicates.len(), m.conflicts, days);
    }
    println!("samen: {} originelen, {} sidecars, {} previews, {:.1} GB, waarvan {:.1} GB opnieuw naar Drive; {} dubbel, {} naamconflicten", p.files, p.sidecars, p.previews, p.bytes as f64 / 1e9, p.drive_bytes as f64 / 1e9, p.duplicates, p.conflicts);
}
