//! Prints differing byte ranges between two saves (decrypted).
use nhse_core::HorizonSave;
fn ranges(a: &[u8], b: &[u8]) -> Vec<(usize, usize)> {
    let mut out = vec![];
    let mut i = 0;
    while i < a.len().min(b.len()) {
        if a[i] != b[i] {
            let s = i;
            while i < a.len() && a[i] != b[i] {
                i += 1;
            }
            out.push((s, i));
        }
        i += 1;
    }
    out
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let a = HorizonSave::open(std::path::Path::new(&args[1])).unwrap();
    let b = HorizonSave::open(std::path::Path::new(&args[2])).unwrap();
    for (s, e) in ranges(&a.main.data, &b.main.data).iter().take(20) {
        println!(
            "main {:#X}..{:#X}: {:02X?} vs {:02X?}",
            s,
            e,
            &a.main.data[*s..(*e).min(s + 16)],
            &b.main.data[*s..(*e).min(s + 16)]
        );
    }
    let (pa, pb) = (&a.players[0].personal.data, &b.players[0].personal.data);
    for (s, e) in ranges(pa, pb).iter().take(20) {
        println!(
            "pers {:#X}..{:#X}: {:02X?} vs {:02X?}",
            s,
            e,
            &pa[*s..(*e).min(s + 16)],
            &pb[*s..(*e).min(s + 16)]
        );
    }
    println!(
        "personal_id={:#X} wallet={:#X}",
        a.personal_ofs.personal_id, a.personal_ofs.wallet
    );
}
