use nhse_core::save::player::PlayerList;
use nhse_core::save::revision::REVISIONS;
use nhse_core::{HorizonSave, Item};

fn tmp(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("nhse-test-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
fn open_edit_save_reopen_all_revisions() {
    for rev in [0usize, 1, 6, 8, 10, 13, 15, 16, 17, 18, 19, 20, 22, 31, REVISIONS.len() - 1] {
        let dir = tmp(&format!("rt{rev}"));
        nhse_core::testing::write_synthetic_save(&dir, rev, 2, 42);
        let mut s = HorizonSave::open(&dir).unwrap();
        assert_eq!(s.revision.index, rev);
        assert!(!s.revision.compatibility_mode);
        assert!(s.invalid_hashes().is_empty(), "rev {rev}: hashes invalid on load");
        assert_eq!(s.players.len(), 2);

        let mut p = s.player_profile(1).unwrap();
        assert_eq!(p.name, "Player1");
        assert_eq!(p.wallet, 12_346);
        p.wallet = 99_999;
        p.name = "Zoë".into();
        s.set_player_profile(1, &p).unwrap();

        let mut items = s.player_items(0, PlayerList::Pocket).unwrap();
        items[5] = Item::new(0x0A54);
        s.set_player_items(0, PlayerList::Pocket, &items).unwrap();
        s.save(7).unwrap();

        let s2 = HorizonSave::open(&dir).unwrap();
        assert!(s2.invalid_hashes().is_empty(), "rev {rev}: hashes invalid after save");
        let p2 = s2.player_profile(1).unwrap();
        assert_eq!(p2.wallet, 99_999);
        assert_eq!(p2.name, "Zoë");
        assert_eq!(s2.player_items(0, PlayerList::Pocket).unwrap()[5].id, 0x0A54);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[test]
fn zip_with_nested_folder_roundtrip() {
    use std::io::Write;
    let dir = tmp("zipsrc");
    nhse_core::testing::write_synthetic_save(&dir, REVISIONS.len() - 1, 1, 9);
    let zip_path = std::env::temp_dir().join(format!("nhse-test-{}.zip", std::process::id()));
    {
        let f = std::fs::File::create(&zip_path).unwrap();
        let mut z = zip::ZipWriter::new(f);
        let opts = zip::write::SimpleFileOptions::default();
        for entry in walk(&dir) {
            let rel = entry.strip_prefix(&dir).unwrap().to_string_lossy().replace('\\', "/");
            z.start_file(format!("My Save/{rel}"), opts).unwrap();
            z.write_all(&std::fs::read(&entry).unwrap()).unwrap();
        }
        z.finish().unwrap();
    }
    let mut s = HorizonSave::open(&zip_path).unwrap();
    let mut t = s.turnips();
    t.buy_price = 105;
    s.set_turnips(&t);
    s.save(1).unwrap();
    let s2 = HorizonSave::open(&zip_path).unwrap();
    assert_eq!(s2.turnips().buy_price, 105);
    assert!(s2.invalid_hashes().is_empty());
    let _ = std::fs::remove_file(&zip_path);
    let _ = std::fs::remove_dir_all(&dir);
}

fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = vec![];
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            out.extend(walk(&p));
        } else {
            out.push(p);
        }
    }
    out
}
