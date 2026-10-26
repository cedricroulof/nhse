//! `cargo run -p nhse-core --example synth -- <dir> <revision-index> [players]`
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = std::path::PathBuf::from(&args[1]);
    let rev: usize = args[2].parse().unwrap();
    let players: usize = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(2);
    nhse_core::testing::write_synthetic_save(&dir, rev, players, 0xC0FFEE);
    println!("wrote synthetic save rev {} to {}", rev, dir.display());
}
