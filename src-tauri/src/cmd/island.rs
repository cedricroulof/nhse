//! Island-wide commands (`main.dat`).
use crate::error::{err, CmdResult};
use crate::state::AppState;
use crate::{with_save, with_save_mut};
use nhse_core::editing::formats;
use nhse_core::map::render::{decode_png, encode_png};
use nhse_core::save::island::{DesignKind, IslandSettings};
use nhse_core::structs::bbs::Board;
use nhse_core::structs::building::Building;
use nhse_core::structs::design::{self, DesignInfo};
use nhse_core::structs::house::{PlayerHouse, PlayerRoom, VillagerHouse};
use nhse_core::structs::misc::{GDate, Museum, Turnips, Visitors};
use nhse_core::structs::villager::Villager;
use nhse_core::Item;
use serde::Serialize;
use tauri::State;

#[tauri::command]
pub fn island_settings(state: State<AppState>) -> CmdResult<IslandSettings> {
    with_save!(state, |s| Ok(s.island_settings()))
}

#[tauri::command]
pub fn set_island_settings(state: State<AppState>, settings: IslandSettings) -> CmdResult<()> {
    with_save_mut!(state, |s| {
        s.set_island_settings(&settings);
        Ok(())
    })
}

// --- villagers ----------------------------------------------------------------------

#[tauri::command]
pub fn villagers(state: State<AppState>) -> CmdResult<Vec<Villager>> {
    with_save!(state, |s| Ok(s.villagers()))
}

#[tauri::command]
pub fn set_villager(state: State<AppState>, villager: Villager) -> CmdResult<Vec<Villager>> {
    with_save_mut!(state, |s| {
        s.set_villager(&villager)?;
        Ok(s.villagers())
    })
}

#[tauri::command]
pub fn villager_templates() -> CmdResult<Vec<String>> {
    Ok(nhse_core::editing::villagers::available_templates().to_vec())
}

#[tauri::command]
pub fn replace_villager(state: State<AppState>, index: usize, internal: String, move_out: bool) -> CmdResult<Vec<Villager>> {
    with_save_mut!(state, |s| {
        nhse_core::editing::villagers::replace_villager(s, index, &internal, move_out)?;
        Ok(s.villagers())
    })
}

#[tauri::command]
pub fn export_villager(state: State<AppState>, index: usize, path: String) -> CmdResult<()> {
    with_save!(state, |s| {
        std::fs::write(path, s.villager_raw(index)?)?;
        Ok(())
    })
}

#[derive(Serialize)]
pub struct ImportResult {
    pub foreign: bool,
    pub owner: String,
}

/// Imports a villager dump (`.nhv`/`.nhv2`). If it came from another island, the town/player
/// identities can be rewritten to the host's (`adopt`).
#[tauri::command]
pub fn import_villager(state: State<AppState>, index: usize, path: String, adopt: bool) -> CmdResult<ImportResult> {
    let raw = std::fs::read(&path)?;
    with_save_mut!(state, |s| {
        let expect = s.main_ofs.villager_size;
        let Some(mut data) = formats::villager_compatible(&raw, expect) else {
            return err(format!("This file is {} bytes; a villager for this game version is {} bytes.", raw.len(), expect));
        };
        let town = s.town_identity();
        let player = s.player_identity(0);
        let mem_town = data[4..4 + 24].to_vec();
        let mem_player = data[4 + 0x1C..4 + 0x1C + 24].to_vec();
        let foreign = mem_town != town || mem_player != player;
        let owner = nhse_core::util::strings::get_string(&data, 4 + 0x20, 10);
        if foreign && adopt {
            nhse_core::util::bytes::replace_occurrences(&mut data, &mem_town, &town);
            nhse_core::util::bytes::replace_occurrences(&mut data, &mem_player, &player);
        }
        s.set_villager_raw(index, &data)?;
        Ok(ImportResult { foreign, owner })
    })
}

#[tauri::command]
pub fn max_friendship_all(state: State<AppState>) -> CmdResult<Vec<Villager>> {
    with_save_mut!(state, |s| {
        for mut v in s.villagers() {
            for m in v.memories.iter_mut() {
                if !m.player.player_name.is_empty() {
                    m.friendship = 255;
                }
            }
            s.set_villager(&v)?;
        }
        Ok(s.villagers())
    })
}

#[tauri::command]
pub fn villager_houses(state: State<AppState>) -> CmdResult<Vec<VillagerHouse>> {
    with_save!(state, |s| Ok(s.villager_houses()))
}

#[tauri::command]
pub fn set_villager_house(state: State<AppState>, house: VillagerHouse) -> CmdResult<()> {
    with_save_mut!(state, |s| {
        s.set_villager_house(&house);
        Ok(())
    })
}

#[tauri::command]
pub fn export_villager_house(state: State<AppState>, index: usize, path: String) -> CmdResult<()> {
    with_save!(state, |s| {
        std::fs::write(path, s.villager_house_raw(index))?;
        Ok(())
    })
}

#[tauri::command]
pub fn import_villager_house(state: State<AppState>, index: usize, path: String) -> CmdResult<()> {
    let raw = std::fs::read(&path)?;
    with_save_mut!(state, |s| {
        let expect = s.main_ofs.villager_house_size;
        let Some(data) = formats::villager_house_compatible(&raw, expect) else {
            return err(format!("This file is {} bytes; expected {}.", raw.len(), expect));
        };
        // keep links to the current resident
        let cur = s.villager_houses()[index].clone();
        s.set_villager_house_raw(index, &data)?;
        let mut h = s.villager_houses()[index].clone();
        h.npc1 = cur.npc1;
        h.npc2 = cur.npc2;
        h.build_player = cur.build_player;
        s.set_villager_house(&h);
        Ok(())
    })
}

// --- player houses ---------------------------------------------------------------------

#[tauri::command]
pub fn player_houses(state: State<AppState>) -> CmdResult<Vec<PlayerHouse>> {
    with_save!(state, |s| Ok(s.player_houses()))
}

#[tauri::command]
pub fn set_player_house(state: State<AppState>, house: PlayerHouse) -> CmdResult<()> {
    with_save_mut!(state, |s| {
        s.set_player_house(&house);
        Ok(())
    })
}

#[tauri::command]
pub fn player_room(state: State<AppState>, house: usize, room: usize) -> CmdResult<PlayerRoom> {
    with_save!(state, |s| Ok(s.player_room(house, room)?))
}

#[tauri::command]
pub fn set_player_room(state: State<AppState>, house: usize, room: usize, data: PlayerRoom) -> CmdResult<()> {
    with_save_mut!(state, |s| Ok(s.set_player_room(house, room, &data)?))
}

#[tauri::command]
pub fn export_player_house(state: State<AppState>, index: usize, path: String) -> CmdResult<()> {
    with_save!(state, |s| {
        std::fs::write(path, s.player_house_raw(index))?;
        Ok(())
    })
}

#[tauri::command]
pub fn import_player_house(state: State<AppState>, index: usize, path: String) -> CmdResult<()> {
    let raw = std::fs::read(&path)?;
    with_save_mut!(state, |s| {
        let expect = s.main_ofs.player_house_size;
        let Some(data) = formats::player_house_compatible(&raw, expect) else {
            return err(format!("This file is {} bytes; expected {}.", raw.len(), expect));
        };
        Ok(s.set_player_house_raw(index, &data)?)
    })
}

#[tauri::command]
pub fn export_player_room(state: State<AppState>, house: usize, room: usize, path: String) -> CmdResult<()> {
    with_save!(state, |s| {
        std::fs::write(path, s.player_room_raw(house, room))?;
        Ok(())
    })
}

#[tauri::command]
pub fn import_player_room(state: State<AppState>, house: usize, room: usize, path: String) -> CmdResult<()> {
    let raw = std::fs::read(&path)?;
    with_save_mut!(state, |s| {
        let expect = s.player_room_raw(house, room).len();
        let Some(data) = formats::room_compatible(&raw, expect) else {
            return err(format!("This file is {} bytes; expected {}.", raw.len(), expect));
        };
        Ok(s.set_player_room_raw(house, room, &data)?)
    })
}

// --- designs -----------------------------------------------------------------------------

fn is_pro(kind: DesignKind) -> bool {
    matches!(kind, DesignKind::Pro | DesignKind::Tailor)
}

#[tauri::command]
pub fn designs(state: State<AppState>, kind: DesignKind) -> CmdResult<Vec<DesignInfo>> {
    with_save!(state, |s| {
        let n = s.design_count(kind);
        (0..n).map(|i| Ok(design::info(s.design_raw(kind, i)?, i, is_pro(kind)))).collect()
    })
}

#[derive(Serialize)]
pub struct DesignPixels {
    pub info: DesignInfo,
    pub palette: Vec<[u8; 3]>,
    /// Palette index per pixel, `sheets × 32 × 32` (15 = transparent).
    pub pixels: Vec<u8>,
}

#[tauri::command]
pub fn design_pixels(state: State<AppState>, kind: DesignKind, index: usize) -> CmdResult<DesignPixels> {
    with_save!(state, |s| {
        let d = s.design_raw(kind, index)?;
        let pro = is_pro(kind);
        let mut pixels = Vec::with_capacity(design::sheets(pro) * 1024);
        for sheet in 0..design::sheets(pro) {
            for y in 0..32 {
                for x in 0..32 {
                    pixels.push(design::pixel(d, sheet, x, y));
                }
            }
        }
        Ok(DesignPixels { info: design::info(d, index, pro), palette: design::palette(d).to_vec(), pixels })
    })
}

#[tauri::command]
pub fn set_design_pixels(state: State<AppState>, kind: DesignKind, index: usize, name: String, palette: Vec<[u8; 3]>, pixels: Vec<u8>) -> CmdResult<()> {
    with_save_mut!(state, |s| {
        let mut d = s.design_raw(kind, index)?.to_vec();
        let pro = is_pro(kind);
        design::set_name(&mut d, &name);
        design::set_palette(&mut d, &palette);
        for (i, p) in pixels.iter().enumerate().take(design::sheets(pro) * 1024) {
            design::set_pixel(&mut d, i / 1024, i % 32, (i % 1024) / 32, *p);
        }
        Ok(s.set_design_raw(kind, index, &d)?)
    })
}

#[tauri::command]
pub fn design_png(state: State<AppState>, kind: DesignKind, index: usize, path: String, scale: Option<usize>) -> CmdResult<()> {
    with_save!(state, |s| {
        let d = s.design_raw(kind, index)?;
        let pro = is_pro(kind);
        let sc = scale.unwrap_or(8).clamp(1, 32);
        let sheets = design::sheets(pro);
        let (w, h) = (32 * sc * if sheets == 4 { 2 } else { 1 }, 32 * sc * if sheets == 4 { 2 } else { 1 });
        let mut out = vec![0u8; w * h * 4];
        for sheet in 0..sheets {
            let rgba = design::render_rgba(d, sheet);
            let (ox, oy) = ((sheet % 2) * 32 * sc, (sheet / 2) * 32 * sc);
            for y in 0..32 * sc {
                for x in 0..32 * sc {
                    let si = ((y / sc) * 32 + x / sc) * 4;
                    let di = ((oy + y) * w + ox + x) * 4;
                    out[di..di + 4].copy_from_slice(&rgba[si..si + 4]);
                }
            }
        }
        std::fs::write(path, encode_png(w, h, &out))?;
        Ok(())
    })
}

/// Imports a PNG (32×32, or 64×64 = 4 sheets for PRO), scaling down by sampling if larger.
#[tauri::command]
pub fn import_design_png(state: State<AppState>, kind: DesignKind, index: usize, path: String, name: Option<String>) -> CmdResult<()> {
    let bytes = std::fs::read(&path)?;
    let Some((w, h, rgba)) = decode_png(&bytes) else {
        return err("Could not read this PNG (indexed PNGs are not supported, please re-save as RGB/RGBA).");
    };
    with_save_mut!(state, |s| {
        let pro = is_pro(kind);
        let sheets = design::sheets(pro);
        let grid = if sheets == 4 { 2 } else { 1 };
        let mut sheet_imgs = Vec::new();
        for sh in 0..sheets {
            let mut img = vec![0u8; 32 * 32 * 4];
            for y in 0..32 {
                for x in 0..32 {
                    let sx = ((sh % grid) * 32 + x) * w / (32 * grid);
                    let sy = ((sh / grid) * 32 + y) * h / (32 * grid);
                    let si = (sy.min(h - 1) * w + sx.min(w - 1)) * 4;
                    img[(y * 32 + x) * 4..(y * 32 + x) * 4 + 4].copy_from_slice(&rgba[si..si + 4]);
                }
            }
            sheet_imgs.push(img);
        }
        let mut d = s.design_raw(kind, index)?.to_vec();
        design::import_rgba(&mut d, &sheet_imgs);
        let default_name = std::path::Path::new(&path).file_stem().map(|x| x.to_string_lossy().chars().take(20).collect()).unwrap_or_default();
        design::set_name(&mut d, &name.unwrap_or(default_name));
        let p = s.player_profile(0)?;
        design::set_author(&mut d, p.town_id, &p.town_name, p.player_id, &p.name);
        Ok(s.set_design_raw(kind, index, &d)?)
    })
}

#[tauri::command]
pub fn export_design(state: State<AppState>, kind: DesignKind, index: usize, path: String) -> CmdResult<()> {
    with_save!(state, |s| {
        std::fs::write(path, s.design_raw(kind, index)?)?;
        Ok(())
    })
}

/// Imports a raw design dump (`.nhd` / `.nhpd`). `adopt` rewrites the author to player 1.
#[tauri::command]
pub fn import_design(state: State<AppState>, kind: DesignKind, index: usize, path: String, adopt: bool) -> CmdResult<()> {
    let data = std::fs::read(&path)?;
    with_save_mut!(state, |s| {
        let expect = s.design_raw(kind, index)?.len();
        if data.len() != expect {
            return err(format!("This file is {} bytes; a {} design is {} bytes.", data.len(), if is_pro(kind) { "PRO" } else { "normal" }, expect));
        }
        let mut d = data.clone();
        if adopt {
            let p = s.player_profile(0)?;
            design::set_author(&mut d, p.town_id, &p.town_name, p.player_id, &p.name);
        }
        Ok(s.set_design_raw(kind, index, &d)?)
    })
}

#[tauri::command]
pub fn set_design_author_all(state: State<AppState>, kind: DesignKind, player: usize) -> CmdResult<usize> {
    with_save_mut!(state, |s| {
        let p = s.player_profile(player)?;
        let n = s.design_count(kind);
        for i in 0..n {
            let mut d = s.design_raw(kind, i)?.to_vec();
            design::set_author(&mut d, p.town_id, &p.town_name, p.player_id, &p.name);
            s.set_design_raw(kind, i, &d)?;
        }
        Ok(n)
    })
}

// --- misc lists ------------------------------------------------------------------------------

#[tauri::command]
pub fn recycle_bin(state: State<AppState>) -> CmdResult<Vec<Item>> {
    with_save!(state, |s| Ok(s.recycle_bin()))
}

#[tauri::command]
pub fn set_recycle_bin(state: State<AppState>, items: Vec<Item>) -> CmdResult<()> {
    with_save_mut!(state, |s| {
        s.set_recycle_bin(&items);
        Ok(())
    })
}

#[tauri::command]
pub fn buildings(state: State<AppState>) -> CmdResult<Vec<Building>> {
    with_save!(state, |s| Ok(s.buildings()))
}

#[tauri::command]
pub fn set_buildings(state: State<AppState>, buildings: Vec<Building>) -> CmdResult<()> {
    with_save_mut!(state, |s| {
        s.set_buildings(&buildings);
        Ok(())
    })
}

#[tauri::command]
pub fn land_flags(state: State<AppState>) -> CmdResult<Vec<i16>> {
    with_save!(state, |s| Ok(s.land_flags()))
}

#[tauri::command]
pub fn set_land_flags(state: State<AppState>, flags: Vec<i16>) -> CmdResult<()> {
    with_save_mut!(state, |s| {
        s.set_land_flags(&flags);
        Ok(())
    })
}

#[tauri::command]
pub fn turnips(state: State<AppState>) -> CmdResult<Turnips> {
    with_save!(state, |s| Ok(s.turnips()))
}

#[tauri::command]
pub fn set_turnips(state: State<AppState>, turnips: Turnips) -> CmdResult<()> {
    with_save_mut!(state, |s| {
        s.set_turnips(&turnips);
        Ok(())
    })
}

#[tauri::command]
pub fn museum(state: State<AppState>) -> CmdResult<Museum> {
    with_save!(state, |s| Ok(s.museum()))
}

#[tauri::command]
pub fn set_museum(state: State<AppState>, museum: Museum) -> CmdResult<()> {
    with_save_mut!(state, |s| {
        s.set_museum(&museum);
        Ok(())
    })
}

/// Donates everything of the given categories (fish, bugs, sea, fossils, art) not yet donated.
#[tauri::command]
pub fn museum_donate_all(state: State<AppState>, categories: Vec<String>, player: u8, per_day: usize) -> CmdResult<(Museum, usize)> {
    with_save_mut!(state, |s| {
        let t = nhse_core::data::tables();
        let en = nhse_core::data::strings("en");
        let mut ids: Vec<u16> = Vec::new();
        for c in &categories {
            match c.as_str() {
                "fish" => ids.extend(t.list("Fish")),
                "bugs" => ids.extend(t.list("Bugs")),
                "sea" => ids.extend(t.list("Dive")),
                "fossils" => ids.extend(t.list("Fossils")),
                // genuine pieces only (forgeries have a parenthesised suffix)
                "art" => ids.extend(t.list("Art").iter().filter(|i| !en.items.get(**i as usize).is_some_and(|n| n.contains('(')))),
                _ => {}
            }
        }
        let mut m = s.museum();
        let n = m.donate_all(&ids, player, per_day.max(1));
        s.set_museum(&m);
        Ok((m, n))
    })
}

#[tauri::command]
pub fn museum_sort(state: State<AppState>) -> CmdResult<Museum> {
    with_save_mut!(state, |s| {
        let mut m = s.museum();
        m.sort();
        s.set_museum(&m);
        Ok(m)
    })
}

#[tauri::command]
pub fn visitors(state: State<AppState>) -> CmdResult<Visitors> {
    with_save!(state, |s| Ok(s.visitors()))
}

#[tauri::command]
pub fn set_visitors(state: State<AppState>, visitors: Visitors) -> CmdResult<()> {
    with_save_mut!(state, |s| {
        s.set_visitors(&visitors);
        Ok(())
    })
}

#[tauri::command]
pub fn bulletin(state: State<AppState>) -> CmdResult<Board> {
    with_save!(state, |s| Ok(s.bulletin()))
}

#[tauri::command]
pub fn set_bulletin(state: State<AppState>, board: Board) -> CmdResult<()> {
    with_save_mut!(state, |s| {
        s.set_bulletin_text(&board);
        Ok(())
    })
}

#[tauri::command]
pub fn export_bulletin_post(state: State<AppState>, slot: usize, path: String) -> CmdResult<()> {
    with_save!(state, |s| {
        std::fs::write(path, s.bulletin_post_raw(slot))?;
        Ok(())
    })
}

#[tauri::command]
pub fn import_bulletin_post(state: State<AppState>, slot: usize, path: String) -> CmdResult<()> {
    let d = std::fs::read(path)?;
    with_save_mut!(state, |s| Ok(s.set_bulletin_post_raw(slot, &d)?))
}

/// Exports every post's text as a readable text file (issue #344).
#[tauri::command]
pub fn export_bulletin_text(state: State<AppState>, path: String) -> CmdResult<usize> {
    with_save!(state, |s| {
        let b = s.bulletin();
        let mut out = String::new();
        let mut n = 0;
        for p in b.posts.iter().filter(|p| !p.body.is_empty() || p.has_handwriting) {
            n += 1;
            out.push_str(&format!("── #{} · {} · {} ({})\n{}\n{}\n\n", p.slot + 1, p.date, p.author.player_name, p.author.town_name, p.body, p.footer));
        }
        std::fs::write(path, out)?;
        Ok(n)
    })
}

#[tauri::command]
pub fn today() -> GDate {
    GDate::today()
}
