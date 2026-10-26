//! Synthetic save generation for tests and cross-validation (not used by the app).
use crate::save::crypto;
use crate::save::hash;
use crate::save::offsets_gen;
use crate::save::revision::{expected_size, REVISIONS};
use crate::save::EncryptedInt32;
use crate::util::strings::set_string;
use std::path::Path;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u8 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 24) as u8
    }
    fn fill(&mut self, d: &mut [u8]) {
        for b in d {
            *b = self.next();
        }
    }
}

fn header_for(rev: usize, rng: &mut Rng) -> Vec<u8> {
    let r = &REVISIONS[rev];
    let mut h = vec![0u8; 0x300];
    rng.fill(&mut h);
    h[0..4].copy_from_slice(&r.major.to_le_bytes());
    h[4..8].copy_from_slice(&r.minor.to_le_bytes());
    h[8..10].copy_from_slice(&2u16.to_le_bytes());
    h[10..12].copy_from_slice(&0u16.to_le_bytes());
    h[12..14].copy_from_slice(&2u16.to_le_bytes());
    h[14..16].copy_from_slice(&r.save_revision.to_le_bytes());
    h
}

fn write_pair(dir: &Path, name: &str, data: &mut [u8], rev: usize, seed: u32, rng: &mut Rng) {
    if let Some(d) = hash::details_for(REVISIONS[rev].hash, &format!("{name}.dat")) {
        hash::update_hashes(data, d);
    }
    let header = header_for(rev, rng);
    let (enc, hdr) = crypto::encrypt(data, seed, &header);
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join(format!("{name}.dat")), enc).unwrap();
    std::fs::write(dir.join(format!("{name}Header.dat")), hdr).unwrap();
}

fn enc_int(d: &mut [u8], o: usize, v: u32, rng: &mut Rng) {
    EncryptedInt32 {
        value: v,
        adjust: rng.next() as u16 * 3,
        shift: rng.next() % 8,
        checksum: 0,
    }
    .write(d, o);
}

/// Writes an encrypted synthetic save for revision index `rev` with `players` players.
pub fn write_synthetic_save(dir: &Path, rev: usize, players: usize, seed: u32) {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ rev as u64);
    let mo = offsets_gen::main_offsets(rev).unwrap();
    let po = offsets_gen::personal_offsets(rev).unwrap();

    let mut main = vec![0u8; expected_size(rev, "main.dat").unwrap()];
    rng.fill(&mut main);
    // villagers: valid species / variants, empty item lists
    for i in 0..10 {
        let o = mo.animal + i * mo.villager_size;
        main[o] = (i * 3) as u8;
        main[o + 1] = i as u8;
        main[o + 2] = (i % 8) as u8;
    }
    // make the field item layers empty so map tools have a clean slate
    let tiles = mo.field_item_acre_width * 32 * 6 * 32;
    for i in 0..tiles * 2 {
        let o = mo.field_item + i * 8;
        main[o..o + 8].copy_from_slice(&0xFFFEu64.to_le_bytes());
    }
    write_pair(dir, "main", &mut main, rev, seed, &mut rng);

    for p in 0..players {
        let pdir = dir.join(format!("Villager{p}"));
        let mut pers = vec![0u8; expected_size(rev, "personal.dat").unwrap()];
        rng.fill(&mut pers);
        pers[po.personal_id..po.personal_id + 4].copy_from_slice(&0x1234_5678u32.to_le_bytes());
        set_string(&mut pers, po.personal_id + 4, 10, "Testland");
        pers[po.personal_id + 0x1C..po.personal_id + 0x20]
            .copy_from_slice(&(0xABCD_0000u32 + p as u32).to_le_bytes());
        set_string(&mut pers, po.personal_id + 0x20, 10, &format!("Player{p}"));
        enc_int(&mut pers, po.wallet, 12_345 + p as u32, &mut rng);
        enc_int(&mut pers, po.bank, 999_999, &mut rng);
        enc_int(&mut pers, po.now_point, 4_321, &mut rng);
        enc_int(&mut pers, po.total_point, 50_000, &mut rng);
        if let Some((o, _)) = po.hotel {
            enc_int(&mut pers, o, 77, &mut rng);
        }
        for i in 0..po.pockets1_count {
            pers[po.pockets1 + i * 8..po.pockets1 + i * 8 + 8]
                .copy_from_slice(&0xFFFEu64.to_le_bytes());
        }
        for i in 0..po.pockets2_count {
            let v: u64 = if i == 0 {
                0x0001_0000_0000_0A54
            } else {
                0xFFFE
            };
            pers[po.pockets2 + i * 8..po.pockets2 + i * 8 + 8].copy_from_slice(&v.to_le_bytes());
        }
        // no JPEG marker for the photo
        pers[po.profile_photo] = 0;
        write_pair(&pdir, "personal", &mut pers, rev, seed, &mut rng);
        for (name, file) in [
            ("photo_studio_island", "photo_studio_island.dat"),
            ("postbox", "postbox.dat"),
            ("profile", "profile.dat"),
        ] {
            let mut d = vec![0u8; expected_size(rev, file).unwrap()];
            rng.fill(&mut d);
            write_pair(&pdir, name, &mut d, rev, seed, &mut rng);
        }
        if offsets_gen::wherearen_supported(rev) {
            let mut d = vec![0u8; expected_size(rev, "wherearen.dat").unwrap()];
            rng.fill(&mut d);
            enc_int(&mut d, offsets_gen::WHEREAREN_POKI, 600, &mut rng);
            write_pair(&pdir, "wherearen", &mut d, rev, seed, &mut rng);
        }
    }
}
