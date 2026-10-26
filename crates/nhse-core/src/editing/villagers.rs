//! Villager replacement from bundled templates (2.0+ saves).
use crate::error::{Error, Result};
use crate::save::HorizonSave;
use crate::structs::house::VILLAGER_HOUSE2_SIZE;
use crate::structs::villager::{LAYOUT_V2, MEMORY_COUNT, MEMORY_SIZE};
use std::io::Read;
use std::sync::OnceLock;

static TEMPLATES: &[u8] = include_bytes!("../../resources/villagers.zip");

fn template_names() -> &'static Vec<String> {
    static N: OnceLock<Vec<String>> = OnceLock::new();
    N.get_or_init(|| {
        let mut z =
            zip::ZipArchive::new(std::io::Cursor::new(TEMPLATES)).expect("embedded templates");
        let mut names: Vec<String> = (0..z.len())
            .filter_map(|i| z.by_index(i).ok().map(|f| f.name().to_string()))
            .filter_map(|n| {
                n.strip_prefix("Villagers/")
                    .and_then(|s| s.strip_suffix("V.bin"))
                    .map(str::to_string)
            })
            .collect();
        names.sort();
        names
    })
}

/// Internal names (`cat00`, …) of villagers that can be swapped in.
pub fn available_templates() -> &'static [String] {
    template_names()
}

fn read_template(path: &str) -> Option<Vec<u8>> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(TEMPLATES)).ok()?;
    let mut f = z.by_name(path).ok()?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).ok()?;
    Some(buf)
}

/// Replaces villager `index` with a template, keeping the player relationships' identities and
/// the house's links. The original editor copied the template's player id into the old memory,
/// which left the new villager pointing at a stranger; here the real player ids are preserved.
pub fn replace_villager(
    s: &mut HorizonSave,
    index: usize,
    internal: &str,
    move_out: bool,
) -> Result<()> {
    if s.main_ofs.villager_format < 2 || s.main_ofs.villager_house_format < 2 {
        return Err(Error::Invalid(
            "Villager replacement requires a save from game version 2.0 or newer.".into(),
        ));
    }
    let v = read_template(&format!("Villagers/{internal}V.bin"))
        .ok_or_else(|| Error::Invalid(format!("No template for villager {internal}")))?;
    let h = read_template(&format!("Houses/{internal}H.bin"))
        .ok_or_else(|| Error::Invalid(format!("No house template for villager {internal}")))?;
    if v.len() != LAYOUT_V2.size || h.len() != VILLAGER_HOUSE2_SIZE {
        return Err(Error::Invalid(
            "Template data has an unexpected size.".into(),
        ));
    }
    let old = s.villager_raw(index)?.to_vec();
    let mut new = v;
    for m in 0..MEMORY_COUNT {
        let o = 4 + m * MEMORY_SIZE;
        new[o..o + 0x38].copy_from_slice(&old[o..o + 0x38]);
        // clear the nickname and greetings learned from the template author
        new[o + 0x138..o + 0x14C].fill(0);
        for g in 0..11 {
            let go = o + 0x14C + 0x60 * g;
            new[go..go + 0x60].fill(0);
        }
    }
    if move_out {
        let mt = LAYOUT_V2.birth_type + 2;
        new[mt] |= 2;
    }
    s.set_villager_raw(index, &new)?;

    // link the template house to this villager
    let houses = s.villager_houses();
    let house_index = houses
        .iter()
        .position(|x| x.npc1 as i64 == index as i64)
        .unwrap_or(index);
    let old_house = &houses[house_index];
    let mut nh = h;
    nh[0x1C4] = old_house.npc1 as u8;
    nh[0x1C5] = old_house.npc2 as u8;
    nh[0x1D0] = old_house.build_player as u8;
    s.set_villager_house_raw(house_index, &nh)?;
    Ok(())
}
