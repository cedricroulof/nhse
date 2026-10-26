//! Applies the same edits as the C# harness "modify" mode and saves with seed 0xBEEF.
use nhse_core::save::player::PlayerList;
use nhse_core::{HorizonSave, Item};

fn copy_dir(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap() {
        let e = e.unwrap();
        let to = dst.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &to);
        } else {
            std::fs::copy(e.path(), to).unwrap();
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (input, output) = (
        std::path::Path::new(&args[1]),
        std::path::Path::new(&args[2]),
    );
    copy_dir(input, output);
    let mut s = HorizonSave::open(output).unwrap();
    let mut prof = s.player_profile(0).unwrap();
    prof.wallet = 55555;
    prof.name = "Renamed".into();
    s.set_player_profile(0, &prof).unwrap();
    let mut is = s.island_settings();
    is.hemisphere = 1;
    s.set_island_settings(&is);
    let mut v = s.villagers()[0].clone();
    v.catchphrase = "hello".into();
    s.set_villager(&v).unwrap();
    let mut items = s.player_items(0, PlayerList::Pocket).unwrap();
    let mut it = Item::new(0x1234);
    it.set_count(5);
    items[1] = it;
    s.set_player_items(0, PlayerList::Pocket, &items).unwrap();
    let mut t = s.turnips();
    t.buy_price = 111;
    s.set_turnips(&t);
    s.save(0xBEEF).unwrap();
    println!("saved={}", output.display());
}
