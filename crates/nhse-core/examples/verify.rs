//! `cargo run -p nhse-core --example verify -- <save dir>`: opens a save and reports hash problems.
fn main() {
    let dir = std::env::args().nth(1).expect("usage: verify <save dir>");
    let s = nhse_core::HorizonSave::open(std::path::Path::new(&dir)).expect("open");
    let bad = s.invalid_hashes();
    println!(
        "revision {} · {} players · {} invalid hash regions",
        s.revision.index,
        s.players.len(),
        bad.len()
    );
    for b in &bad {
        println!("  {b:?}");
    }
    let l0 = s.field_layer(0);
    println!(
        "layer0 items: {}",
        l0.tiles.iter().filter(|t| !t.is_none()).count()
    );
    std::process::exit(if bad.is_empty() { 0 } else { 1 });
}
