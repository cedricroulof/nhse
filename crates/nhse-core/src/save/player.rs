//! Accessors for a player's `personal.dat` (and `wherearen.dat`).
use super::horizon::HorizonSave;
use super::{EncryptedInt32, PersonalOffsets};
use crate::error::{Error, Result};
use crate::structs::misc::{Achievements, GDate, GDateMD, Reactions, RecipeBook};
use crate::structs::Item;
use crate::util::bytes::*;
use crate::util::strings::{get_string, set_string_if_changed};
use serde::{Deserialize, Serialize};

/// The player's core profile values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub index: usize,
    pub dir: String,
    pub name: String,
    pub town_name: String,
    pub player_id: u32,
    pub town_id: u32,
    pub wallet: u32,
    pub bank: u32,
    pub nook_miles: u32,
    pub total_nook_miles: u32,
    pub pocket_count: u32,
    pub bag_count: u32,
    pub storage_count: u32,
    pub storage_capacity: usize,
    pub hotel_tickets: Option<u32>,
    pub poki: Option<u32>,
    pub birthday: GDateMD,
    pub profile_birthday: GDateMD,
    pub profile_fruit: u16,
    pub profile_timestamp: GDate,
    pub profile_made_village: bool,
    pub has_photo: bool,
    /// Encrypted values that failed their checksum on load (shown as warnings).
    pub invalid_values: Vec<String>,
}

pub struct ItemContainer;

/// Which item list of a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlayerList {
    Pocket,
    Storage,
}

impl HorizonSave {
    fn pers(&self, p: usize) -> Result<(&[u8], &'static PersonalOffsets)> {
        let pl = self.players.get(p).ok_or(Error::OutOfRange { index: p, max: self.players.len() })?;
        Ok((&pl.personal.data, self.personal_ofs))
    }

    fn pers_mut(&mut self, p: usize) -> Result<(&mut Vec<u8>, &'static PersonalOffsets)> {
        let n = self.players.len();
        let ofs = self.personal_ofs;
        let pl = self.players.get_mut(p).ok_or(Error::OutOfRange { index: p, max: n })?;
        Ok((&mut pl.personal.data, ofs))
    }

    pub fn player_name(&self, p: usize) -> String {
        self.pers(p).map(|(d, o)| get_string(d, o.personal_id + 0x20, 10)).unwrap_or_default()
    }

    pub fn town_name(&self) -> String {
        self.pers(0).map(|(d, o)| get_string(d, o.personal_id + 4, 10)).unwrap_or_default()
    }

    /// Bytes identifying the player (id + name), used for identity replacement.
    pub fn player_identity(&self, p: usize) -> Vec<u8> {
        self.pers(p).map(|(d, o)| d[o.personal_id + 0x1C..o.personal_id + 0x1C + 24].to_vec()).unwrap_or_default()
    }

    pub fn town_identity(&self) -> Vec<u8> {
        self.pers(0).map(|(d, o)| d[o.personal_id..o.personal_id + 24].to_vec()).unwrap_or_default()
    }

    pub fn player_profile(&self, p: usize) -> Result<PlayerProfile> {
        let (d, o) = self.pers(p)?;
        let mut invalid = Vec::new();
        let mut enc = |name: &str, off: usize| -> u32 {
            if !EncryptedInt32::is_valid(d, off) {
                invalid.push(name.to_string());
            }
            EncryptedInt32::read(d, off).value
        };
        let wallet = enc("Wallet", o.wallet);
        let bank = enc("Bank", o.bank);
        let nook_miles = enc("Nook Miles", o.now_point);
        let total_nook_miles = enc("Total Nook Miles", o.total_point);
        let hotel_tickets = o.hotel.and_then(|(off, _)| {
            // the 3.0 block is zero until the player has visited the hotel area
            (u64le(d, off) != 0).then(|| enc("Hotel Tickets", off))
        });
        let pl = &self.players[p];
        let poki = pl.wherearen.as_ref().and_then(|w| {
            let off = super::offsets_gen::WHEREAREN_POKI;
            (off + 8 <= w.data.len()).then(|| EncryptedInt32::read(&w.data, off).value)
        });
        Ok(PlayerProfile {
            index: p,
            dir: pl.dir.clone(),
            name: get_string(d, o.personal_id + 0x20, 10),
            town_name: get_string(d, o.personal_id + 4, 10),
            player_id: u32le(d, o.personal_id + 0x1C),
            town_id: u32le(d, o.personal_id),
            wallet,
            bank,
            nook_miles,
            total_nook_miles,
            pocket_count: u32le(d, o.pockets2 + o.pockets2_count * 8),
            bag_count: u32le(d, o.pockets1 + o.pockets1_count * 8),
            storage_count: u32le(d, o.item_chest + o.item_chest_count * 8),
            storage_capacity: o.item_chest_count,
            hotel_tickets,
            poki,
            birthday: GDateMD::read(d, o.birthday),
            profile_birthday: GDateMD::read(d, o.profile_birthday),
            profile_fruit: u16le(d, o.profile_fruit),
            profile_timestamp: GDate::read(d, o.profile_timestamp),
            profile_made_village: d[o.profile_is_make_village] != 0,
            has_photo: !self.player_photo(p).is_empty(),
            invalid_values: invalid,
        })
    }

    /// Applies profile edits. Renaming the player or town propagates the identity everywhere.
    pub fn set_player_profile(&mut self, p: usize, v: &PlayerProfile) -> Result<()> {
        let current = self.player_profile(p)?;
        if current.name != v.name {
            let orig = self.player_identity(p);
            {
                let (d, o) = self.pers_mut(p)?;
                set_string_if_changed(d, o.personal_id + 0x20, 10, &v.name);
            }
            let upd = self.player_identity(p);
            self.change_identity(&orig, &upd);
        }
        if current.town_name != v.town_name {
            let (d, o) = self.pers(p)?;
            let orig = d[o.personal_id..o.personal_id + 24].to_vec();
            {
                let (d, o) = self.pers_mut(p)?;
                set_string_if_changed(d, o.personal_id + 4, 10, &v.town_name);
            }
            let (d, o) = self.pers(p)?;
            let upd = d[o.personal_id..o.personal_id + 24].to_vec();
            self.change_identity(&orig, &upd);
        }
        let hotel_present = current.hotel_tickets.is_some();
        let (d, o) = self.pers_mut(p)?;
        EncryptedInt32::set_value(d, o.wallet, v.wallet);
        EncryptedInt32::set_value(d, o.bank, v.bank);
        EncryptedInt32::set_value(d, o.now_point, v.nook_miles);
        EncryptedInt32::set_value(d, o.total_point, v.total_nook_miles);
        put_u32(d, o.pockets2 + o.pockets2_count * 8, v.pocket_count);
        put_u32(d, o.pockets1 + o.pockets1_count * 8, v.bag_count);
        put_u32(d, o.item_chest + o.item_chest_count * 8, v.storage_count);
        if let (Some((off, _)), Some(t), true) = (o.hotel, v.hotel_tickets, hotel_present) {
            EncryptedInt32::set_value(d, off, t);
        }
        v.birthday.write(d, o.birthday);
        v.profile_birthday.write(d, o.profile_birthday);
        put_u16(d, o.profile_fruit, v.profile_fruit);
        v.profile_timestamp.write(d, o.profile_timestamp);
        put_bool(d, o.profile_is_make_village, v.profile_made_village);
        if let (Some(poki), Some(w)) = (v.poki, self.players[p].wherearen.as_mut()) {
            let off = super::offsets_gen::WHEREAREN_POKI;
            if off + 8 <= w.data.len() {
                EncryptedInt32::set_value(&mut w.data, off, poki);
            }
        }
        Ok(())
    }

    /// Profile photo (JPEG bytes), empty if absent.
    pub fn player_photo(&self, p: usize) -> Vec<u8> {
        let Ok((d, o)) = self.pers(p) else { return Vec::new() };
        let off = o.profile_photo;
        if off + 2 > d.len() || u16le(d, off) != 0xD8FF {
            return Vec::new();
        }
        let len = i32le(d, off - 4);
        if len <= 0 || off + len as usize > d.len() {
            return Vec::new();
        }
        d[off..off + len as usize].to_vec()
    }

    // --- items -------------------------------------------------------------

    /// Pocket (20 slots) followed by bag (20 slots), in in-game order.
    pub fn player_items(&self, p: usize, list: PlayerList) -> Result<Vec<Item>> {
        let (d, o) = self.pers(p)?;
        Ok(match list {
            PlayerList::Pocket => {
                let mut v = Item::read_array(d, o.pockets2, o.pockets2_count);
                v.extend(Item::read_array(d, o.pockets1, o.pockets1_count));
                v
            }
            PlayerList::Storage => Item::read_array(d, o.item_chest, o.item_chest_count),
        })
    }

    pub fn set_player_items(&mut self, p: usize, list: PlayerList, items: &[Item]) -> Result<()> {
        let (d, o) = self.pers_mut(p)?;
        match list {
            PlayerList::Pocket => {
                if items.len() != o.pockets1_count + o.pockets2_count {
                    return Err(Error::Invalid("pocket must have 40 slots".into()));
                }
                Item::write_array(d, o.pockets2, &items[..o.pockets2_count]);
                Item::write_array(d, o.pockets1, &items[o.pockets2_count..]);
            }
            PlayerList::Storage => {
                let n = items.len().min(o.item_chest_count);
                Item::write_array(d, o.item_chest, &items[..n]);
            }
        }
        Ok(())
    }

    // --- recipes -----------------------------------------------------------

    /// Returns `[known, made, new, favorite]` bit arrays.
    pub fn recipes(&self, p: usize) -> Result<Vec<[bool; 4]>> {
        let (d, o) = self.pers(p)?;
        Ok((0..RecipeBook::COUNT)
            .map(|r| std::array::from_fn(|f| RecipeBook::get(d, o.recipes, f, r)))
            .collect())
    }

    pub fn set_recipes(&mut self, p: usize, flags: &[[bool; 4]]) -> Result<()> {
        let (d, o) = self.pers_mut(p)?;
        for (r, f) in flags.iter().enumerate().take(RecipeBook::COUNT) {
            for (i, v) in f.iter().enumerate() {
                RecipeBook::set(d, o.recipes, i, r, *v);
            }
        }
        Ok(())
    }

    // --- reactions, achievements, flags --------------------------------------

    pub fn reactions(&self, p: usize) -> Result<Reactions> {
        let (d, o) = self.pers(p)?;
        Ok(Reactions::read(d, o.manpu, o.manpu_format))
    }

    pub fn set_reactions(&mut self, p: usize, r: &Reactions) -> Result<()> {
        let (d, o) = self.pers_mut(p)?;
        r.write(d, o.manpu, o.manpu_format);
        Ok(())
    }

    pub fn achievements(&self, p: usize) -> Result<Achievements> {
        let (d, o) = self.pers(p)?;
        Ok(Achievements::read(d, o.count_achievement))
    }

    pub fn set_achievements(&mut self, p: usize, a: &Achievements) -> Result<()> {
        let (d, o) = self.pers_mut(p)?;
        a.write(d, o.count_achievement);
        Ok(())
    }

    pub fn player_flags(&self, p: usize) -> Result<Vec<i16>> {
        let (d, o) = self.pers(p)?;
        Ok(read_i16_array(&d[o.event_flags_player..o.event_flags_player + o.event_flags_player_length]))
    }

    pub fn set_player_flags(&mut self, p: usize, flags: &[i16]) -> Result<()> {
        let (d, o) = self.pers_mut(p)?;
        let n = flags.len().min(o.event_flags_player_length / 2);
        write_i16_array(&mut d[o.event_flags_player..o.event_flags_player + n * 2], &flags[..n]);
        Ok(())
    }

    // --- catalog (received items) -------------------------------------------

    pub fn received_items(&self, p: usize) -> Result<(Vec<bool>, Vec<bool>)> {
        let (d, o) = self.pers(p)?;
        let n = crate::data::items::item_count();
        let items = (0..n).map(|i| crate::util::bits::get_flag(d, o.item_collect_bit, i)).collect();
        let remake = (0..o.max_remake_bit_flag).map(|i| crate::util::bits::get_flag(d, o.item_remake_collect_bit, i)).collect();
        Ok((items, remake))
    }

    pub fn set_received_items(&mut self, p: usize, items: &[bool], remake: &[bool]) -> Result<()> {
        let (d, o) = self.pers_mut(p)?;
        let n = items.len().min(crate::data::items::item_count());
        for (i, v) in items.iter().enumerate().take(n) {
            crate::util::bits::set_flag(d, o.item_collect_bit, i, *v);
        }
        for (i, v) in remake.iter().enumerate().take(o.max_remake_bit_flag) {
            crate::util::bits::set_flag(d, o.item_remake_collect_bit, i, *v);
        }
        Ok(())
    }
}
