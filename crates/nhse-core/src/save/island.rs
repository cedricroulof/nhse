//! Accessors for `main.dat` (island-wide data).
use super::horizon::HorizonSave;
use crate::error::{Error, Result};
use crate::structs::bbs::{self, Board};
use crate::structs::building::{self, Building};
use crate::structs::design;
use crate::structs::house::{self, PlayerHouse, PlayerRoom, VillagerHouse};
use crate::structs::misc::{GDate, GTime, Museum, Turnips, Visitors};
use crate::structs::villager::Villager;
use crate::structs::Item;
use crate::util::bytes::*;
use serde::{Deserialize, Serialize};

pub const VILLAGER_COUNT: usize = 10;
pub const PLAYER_COUNT: usize = 8;
pub const RECYCLE_BIN_COUNT: usize = 40;
pub const TAILOR_COUNT: usize = 8;
pub const LAND_FLAG_COUNT: usize = 0x400;

/// Island-wide settings shown on the overview page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IslandSettings {
    pub hemisphere: u8,
    pub tour_hemisphere: u8,
    pub airport_color: u8,
    pub weather_seed: u32,
    pub tour_weather_seed: u32,
    pub specialty_fruit: u16,
    pub sister_fruit: u16,
    pub specialty_flower: u8,
    pub sister_flower: u8,
    pub fruit_flags: Vec<u8>,
    pub last_saved: String,
    pub camp_status: u8,
    pub camp_species: u8,
    pub camp_variant: u8,
    pub camp_last_visit: GDate,
}

/// Which list of designs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DesignKind {
    Normal,
    Pro,
    Flag,
    Tailor,
}

impl HorizonSave {
    fn m(&self) -> &[u8] {
        &self.main.data
    }
    fn mm(&mut self) -> &mut [u8] {
        &mut self.main.data
    }

    pub fn island_settings(&self) -> IslandSettings {
        let o = self.main_ofs;
        let d = self.m();
        IslandSettings {
            hemisphere: d[o.weather_area],
            tour_hemisphere: d[o.tour_hemisphere],
            airport_color: d[o.airport_theme_color],
            weather_seed: u32le(d, o.weather_rand_seed),
            tour_weather_seed: u32le(d, o.tour_weather_rand_seed),
            specialty_fruit: u16le(d, o.specialty_fruit),
            sister_fruit: u16le(d, o.sister_fruit),
            specialty_flower: d[o.specialty_flower],
            sister_flower: d[o.sister_flower],
            fruit_flags: d[o.fruit_flags..o.fruit_flags + 5].to_vec(),
            last_saved: GTime::read(d, o.last_saved_time).timestamp(),
            camp_status: d[o.g_save_camp_site],
            camp_variant: d[o.g_save_npc_camp],
            camp_species: d[o.g_save_npc_camp + 1],
            camp_last_visit: GDate::read(d, o.camp_last_visit_time),
        }
    }

    pub fn set_island_settings(&mut self, s: &IslandSettings) {
        let o = self.main_ofs;
        let d = self.mm();
        d[o.weather_area] = s.hemisphere;
        d[o.tour_hemisphere] = s.tour_hemisphere;
        d[o.airport_theme_color] = s.airport_color;
        put_u32(d, o.weather_rand_seed, s.weather_seed);
        put_u32(d, o.tour_weather_rand_seed, s.tour_weather_seed);
        put_u16(d, o.specialty_fruit, s.specialty_fruit);
        put_u16(d, o.sister_fruit, s.sister_fruit);
        d[o.specialty_flower] = s.specialty_flower;
        d[o.sister_flower] = s.sister_flower;
        if s.fruit_flags.len() == 5 {
            d[o.fruit_flags..o.fruit_flags + 5].copy_from_slice(&s.fruit_flags);
        }
        d[o.g_save_camp_site] = s.camp_status;
        d[o.g_save_npc_camp] = s.camp_variant;
        d[o.g_save_npc_camp + 1] = s.camp_species;
        s.camp_last_visit.write(d, o.camp_last_visit_time);
    }

    // --- villagers ----------------------------------------------------------

    fn villager_range(&self, i: usize) -> std::ops::Range<usize> {
        let o = self.main_ofs;
        let s = o.animal + i * o.villager_size;
        s..s + o.villager_size
    }

    pub fn villager_raw(&self, i: usize) -> Result<&[u8]> {
        if i >= VILLAGER_COUNT {
            return Err(Error::OutOfRange { index: i, max: VILLAGER_COUNT });
        }
        Ok(&self.main.data[self.villager_range(i)])
    }

    pub fn set_villager_raw(&mut self, i: usize, data: &[u8]) -> Result<()> {
        let r = self.villager_range(i);
        if i >= VILLAGER_COUNT || data.len() != r.len() {
            return Err(Error::Invalid(format!("villager data must be {} bytes (got {})", r.len(), data.len())));
        }
        self.main.data[r].copy_from_slice(data);
        Ok(())
    }

    pub fn villagers(&self) -> Vec<Villager> {
        let f = self.main_ofs.villager_format;
        (0..VILLAGER_COUNT).map(|i| Villager::read(&self.main.data[self.villager_range(i)], f, i)).collect()
    }

    pub fn set_villager(&mut self, v: &Villager) -> Result<()> {
        if v.index >= VILLAGER_COUNT {
            return Err(Error::OutOfRange { index: v.index, max: VILLAGER_COUNT });
        }
        let r = self.villager_range(v.index);
        v.write(&mut self.main.data[r]);
        Ok(())
    }

    fn villager_house_range(&self, i: usize) -> std::ops::Range<usize> {
        let o = self.main_ofs;
        let s = o.npc_house_list + i * o.villager_house_size;
        s..s + o.villager_house_size
    }

    pub fn villager_house_raw(&self, i: usize) -> &[u8] {
        &self.main.data[self.villager_house_range(i)]
    }

    pub fn set_villager_house_raw(&mut self, i: usize, data: &[u8]) -> Result<()> {
        let r = self.villager_house_range(i);
        if data.len() != r.len() {
            return Err(Error::Invalid(format!("villager house data must be {} bytes (got {})", r.len(), data.len())));
        }
        self.main.data[r].copy_from_slice(data);
        Ok(())
    }

    pub fn villager_houses(&self) -> Vec<VillagerHouse> {
        let f = self.main_ofs.villager_house_format;
        (0..VILLAGER_COUNT).map(|i| VillagerHouse::read(self.villager_house_raw(i), f, i)).collect()
    }

    pub fn set_villager_house(&mut self, h: &VillagerHouse) {
        let r = self.villager_house_range(h.index);
        h.write(&mut self.main.data[r]);
    }

    // --- player houses -------------------------------------------------------

    fn player_house_range(&self, i: usize) -> std::ops::Range<usize> {
        let o = self.main_ofs;
        let s = o.player_house_list + i * o.player_house_size;
        s..s + o.player_house_size
    }

    pub fn player_houses(&self) -> Vec<PlayerHouse> {
        let f = self.main_ofs.player_house_format;
        (0..PLAYER_COUNT).map(|i| PlayerHouse::read(&self.main.data[self.player_house_range(i)], f, i)).collect()
    }

    pub fn set_player_house(&mut self, h: &PlayerHouse) {
        let r = self.player_house_range(h.index);
        h.write(&mut self.main.data[r]);
    }

    pub fn player_house_raw(&self, i: usize) -> &[u8] {
        &self.main.data[self.player_house_range(i)]
    }

    pub fn set_player_house_raw(&mut self, i: usize, data: &[u8]) -> Result<()> {
        let r = self.player_house_range(i);
        if data.len() != r.len() {
            return Err(Error::Invalid(format!("player house data must be {} bytes (got {})", r.len(), data.len())));
        }
        self.main.data[r].copy_from_slice(data);
        Ok(())
    }

    fn room_abs_range(&self, house: usize, room: usize) -> std::ops::Range<usize> {
        let base = self.player_house_range(house).start;
        let r = PlayerHouse::room_range(self.main_ofs.player_house_format, room);
        base + r.start..base + r.end
    }

    pub fn player_room(&self, house: usize, room: usize) -> Result<PlayerRoom> {
        if house >= PLAYER_COUNT || room >= house::ROOM_COUNT {
            return Err(Error::OutOfRange { index: room, max: house::ROOM_COUNT });
        }
        Ok(PlayerRoom::read(&self.main.data[self.room_abs_range(house, room)], self.main_ofs.player_house_format))
    }

    pub fn set_player_room(&mut self, house: usize, room: usize, r: &PlayerRoom) -> Result<()> {
        if house >= PLAYER_COUNT || room >= house::ROOM_COUNT {
            return Err(Error::OutOfRange { index: room, max: house::ROOM_COUNT });
        }
        let rr = self.room_abs_range(house, room);
        let f = self.main_ofs.player_house_format;
        r.write(&mut self.main.data[rr], f);
        Ok(())
    }

    pub fn player_room_raw(&self, house: usize, room: usize) -> &[u8] {
        &self.main.data[self.room_abs_range(house, room)]
    }

    pub fn set_player_room_raw(&mut self, house: usize, room: usize, data: &[u8]) -> Result<()> {
        let r = self.room_abs_range(house, room);
        if data.len() != r.len() {
            return Err(Error::Invalid(format!("room data must be {} bytes (got {})", r.len(), data.len())));
        }
        self.main.data[r].copy_from_slice(data);
        Ok(())
    }

    // --- designs ---------------------------------------------------------------

    pub fn design_count(&self, kind: DesignKind) -> usize {
        match kind {
            DesignKind::Normal | DesignKind::Pro => self.main_ofs.pattern_count,
            DesignKind::Flag => 1,
            DesignKind::Tailor => TAILOR_COUNT,
        }
    }

    pub fn design_range(&self, kind: DesignKind, i: usize) -> Result<std::ops::Range<usize>> {
        let o = self.main_ofs;
        if i >= self.design_count(kind) {
            return Err(Error::OutOfRange { index: i, max: self.design_count(kind) });
        }
        let (start, size) = match kind {
            DesignKind::Normal => (o.land_my_design + i * design::NORMAL_SIZE, design::NORMAL_SIZE),
            DesignKind::Pro => (o.patterns_pro + i * design::PRO_SIZE, design::PRO_SIZE),
            DesignKind::Flag => (o.pattern_flag, design::NORMAL_SIZE),
            DesignKind::Tailor => (o.pattern_tailor + i * design::PRO_SIZE, design::PRO_SIZE),
        };
        Ok(start..start + size)
    }

    pub fn design_raw(&self, kind: DesignKind, i: usize) -> Result<&[u8]> {
        Ok(&self.main.data[self.design_range(kind, i)?])
    }

    /// Writes a design. Normal/PRO slots also reset the "edited" flag so the game picks up the name.
    pub fn set_design_raw(&mut self, kind: DesignKind, i: usize, data: &[u8]) -> Result<()> {
        let r = self.design_range(kind, i)?;
        if data.len() != r.len() {
            return Err(Error::Invalid(format!("design must be {} bytes (got {})", r.len(), data.len())));
        }
        self.main.data[r].copy_from_slice(data);
        let o = self.main_ofs;
        match kind {
            DesignKind::Normal => self.main.data[o.patterns_edit_flag_start + i] = 0,
            DesignKind::Pro => self.main.data[o.patterns_pro_edit_flag_start + i] = 0,
            _ => {}
        }
        Ok(())
    }

    // --- misc lists -------------------------------------------------------------

    pub fn recycle_bin(&self) -> Vec<Item> {
        Item::read_array(self.m(), self.main_ofs.lost_item_box, RECYCLE_BIN_COUNT)
    }

    pub fn set_recycle_bin(&mut self, items: &[Item]) {
        let o = self.main_ofs.lost_item_box;
        Item::write_array(self.mm(), o, &items[..items.len().min(RECYCLE_BIN_COUNT)]);
    }

    pub fn buildings(&self) -> Vec<Building> {
        let o = self.main_ofs.main_field_structure;
        (0..building::COUNT).map(|i| Building::read(self.m(), o + i * building::SIZE)).collect()
    }

    pub fn set_buildings(&mut self, b: &[Building]) {
        let o = self.main_ofs.main_field_structure;
        for (i, x) in b.iter().enumerate().take(building::COUNT) {
            x.write(self.mm(), o + i * building::SIZE);
        }
    }

    pub fn land_flags(&self) -> Vec<i16> {
        let o = self.main_ofs.event_flag_land;
        read_i16_array(&self.m()[o..o + LAND_FLAG_COUNT * 2])
    }

    pub fn set_land_flags(&mut self, f: &[i16]) {
        let o = self.main_ofs.event_flag_land;
        let n = f.len().min(LAND_FLAG_COUNT);
        write_i16_array(&mut self.mm()[o..o + n * 2], &f[..n]);
    }

    pub fn turnips(&self) -> Turnips {
        Turnips::read(self.m(), self.main_ofs.shop_kabu)
    }

    pub fn set_turnips(&mut self, t: &Turnips) {
        let o = self.main_ofs.shop_kabu;
        t.write(self.mm(), o);
    }

    pub fn museum(&self) -> Museum {
        Museum::read(self.m(), self.main_ofs.museum)
    }

    pub fn set_museum(&mut self, m: &Museum) {
        let o = self.main_ofs.museum;
        m.write(self.mm(), o);
    }

    pub fn visitors(&self) -> Visitors {
        Visitors::read(self.m(), self.main_ofs.visitor)
    }

    pub fn set_visitors(&mut self, v: &Visitors) {
        let o = self.main_ofs.visitor;
        v.write(self.mm(), o);
    }

    pub fn bulletin(&self) -> Board {
        Board::read(self.m(), self.main_ofs.bulletin_board)
    }

    pub fn set_bulletin_text(&mut self, b: &Board) {
        let o = self.main_ofs.bulletin_board;
        b.write_text(self.mm(), o);
    }

    pub fn bulletin_post_raw(&self, slot: usize) -> Vec<u8> {
        Board::post_bytes(self.m(), self.main_ofs.bulletin_board, slot.min(bbs::POST_COUNT - 1)).to_vec()
    }

    pub fn set_bulletin_post_raw(&mut self, slot: usize, data: &[u8]) -> Result<()> {
        if slot >= bbs::POST_COUNT || data.len() != bbs::POST_SIZE {
            return Err(Error::Invalid(format!("bulletin post must be {} bytes", bbs::POST_SIZE)));
        }
        let o = self.main_ofs.bulletin_board;
        Board::set_post_bytes(self.mm(), o, slot, data);
        Ok(())
    }

    pub fn bulletin_handwriting(&self, slot: usize) -> Vec<u8> {
        Board::handwriting_rgba(self.m(), self.main_ofs.bulletin_board, slot.min(bbs::POST_COUNT - 1))
    }
}
