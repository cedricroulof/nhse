//! The 8-byte item record used everywhere in the save.
use crate::data::items::{item_kind, kinds};
use crate::data::strings::GameStrings;
use crate::data::tables::tables;
use serde::{Deserialize, Serialize};

/// An item: `id:u16 | system_param:u8 | additional_param:u8 | free_param:u32` (little-endian).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Item {
    pub id: u16,
    pub system: u8,
    pub additional: u8,
    pub free: u32,
}

impl Default for Item {
    fn default() -> Self {
        Self::NONE
    }
}

impl Item {
    pub const SIZE: usize = 8;
    pub const NONE_ID: u16 = 0xFFFE;
    pub const EXTENSION_ID: u16 = 0xFFFD;
    pub const FIELD_ITEM_MIN: u16 = 60_000;
    pub const MESSAGE_BOTTLE: u16 = 0x16A1;
    pub const DIY_RECIPE: u16 = 0x16A2;
    pub const MESSAGE_BOTTLE_EGG: u16 = 0x3100;

    pub const NONE: Item = Item { id: Self::NONE_ID, system: 0, additional: 0, free: 0 };

    pub fn new(id: u16) -> Self {
        Self { id, system: 0, additional: 0, free: 0 }
    }

    pub fn from_raw(raw: u64) -> Self {
        Self { id: raw as u16, system: (raw >> 16) as u8, additional: (raw >> 24) as u8, free: (raw >> 32) as u32 }
    }

    pub fn raw(&self) -> u64 {
        self.id as u64 | (self.system as u64) << 16 | (self.additional as u64) << 24 | (self.free as u64) << 32
    }

    pub fn read(d: &[u8], o: usize) -> Self {
        let mut b = [0u8; 8];
        b.copy_from_slice(&d[o..o + 8]);
        Self::from_raw(u64::from_le_bytes(b))
    }

    pub fn write(&self, d: &mut [u8], o: usize) {
        d[o..o + 8].copy_from_slice(&self.raw().to_le_bytes());
    }

    pub fn read_array(d: &[u8], o: usize, count: usize) -> Vec<Item> {
        (0..count).map(|i| Self::read(d, o + i * 8)).collect()
    }

    pub fn write_array(d: &mut [u8], o: usize, items: &[Item]) {
        for (i, it) in items.iter().enumerate() {
            it.write(d, o + i * 8);
        }
    }

    // --- basic predicates -------------------------------------------------
    pub fn is_none(&self) -> bool {
        self.id == Self::NONE_ID
    }
    pub fn is_extension(&self) -> bool {
        self.id == Self::EXTENSION_ID
    }
    pub fn is_root(&self) -> bool {
        self.id < Self::EXTENSION_ID
    }
    pub fn is_field_item(&self) -> bool {
        self.is_root() && self.id >= Self::FIELD_ITEM_MIN
    }
    pub fn display_id(&self) -> u16 {
        if self.is_extension() {
            self.extension_item_id()
        } else {
            self.id
        }
    }
    pub fn clear(&mut self) {
        *self = Self::NONE;
    }

    // --- system param ------------------------------------------------------
    pub fn rotation(&self) -> u8 {
        self.system & 3
    }
    pub fn set_rotation(&mut self, r: u8) {
        self.system = (self.system & !3) | (r & 3);
    }
    fn sys_flag(&self, m: u8) -> bool {
        self.system & m != 0
    }
    fn set_sys_flag(&mut self, m: u8, v: bool) {
        if v {
            self.system |= m
        } else {
            self.system &= !m
        }
    }
    pub fn is_buried(&self) -> bool {
        self.sys_flag(0x04)
    }
    pub fn set_buried(&mut self, v: bool) {
        self.set_sys_flag(0x04, v)
    }
    pub fn is_dropped(&self) -> bool {
        self.sys_flag(0x20)
    }
    pub fn set_dropped(&mut self, v: bool) {
        self.set_sys_flag(0x20, v)
    }

    // --- stackables ---------------------------------------------------------
    pub fn count(&self) -> u16 {
        self.free as u16
    }
    pub fn set_count(&mut self, c: u16) {
        self.free = (self.free & 0xFFFF_0000) | c as u32;
    }
    pub fn use_count(&self) -> u16 {
        (self.free >> 16) as u16
    }
    pub fn set_use_count(&mut self, c: u16) {
        self.free = (self.free & 0xFFFF) | (c as u32) << 16;
    }

    // --- customization -------------------------------------------------------
    pub fn body_type(&self) -> u32 {
        self.free & 7
    }
    pub fn set_body_type(&mut self, v: u32) {
        self.free = (self.free & !7) | (v & 7);
    }
    pub fn pattern_source(&self) -> u32 {
        (self.free >> 3) & 3
    }
    pub fn set_pattern_source(&mut self, v: u32) {
        self.free = (self.free & !0x18) | ((v & 3) << 3);
    }
    pub fn pattern_choice(&self) -> u32 {
        self.free >> 5
    }
    pub fn set_pattern_choice(&mut self, v: u32) {
        self.free = (self.free & 0x1F) | ((v & 0x7FF) << 5);
    }

    // --- wrapping -----------------------------------------------------------
    pub fn is_wrapped(&self) -> bool {
        self.additional != 0 && !matches!(self.display_id(), Self::MESSAGE_BOTTLE | Self::MESSAGE_BOTTLE_EGG)
    }
    /// 0 nothing, 1 wrapping paper, 2 present, 3 delivery, 4 festive (special-cased).
    pub fn wrapping_type(&self) -> u8 {
        let v = self.additional & 3;
        if v == 3 && self.wrapping_paper() == 15 {
            4
        } else {
            v
        }
    }
    pub fn wrapping_paper(&self) -> u8 {
        (self.additional >> 2) & 0xF
    }
    pub fn wrapping_show_item(&self) -> bool {
        self.additional & 0x40 != 0
    }
    pub fn wrapping_80(&self) -> bool {
        self.additional & 0x80 != 0
    }
    pub fn set_wrapping(&mut self, wrap: u8, paper: u8, show_item: bool, item80: bool) {
        if wrap == 0 || wrap > 4 {
            self.additional = 0;
            return;
        }
        let (w, p) = match wrap {
            1 => (1, paper & 0xF),
            4 => (3, 15),
            w => (w, 0),
        };
        self.additional = (w & 3) | (p << 2) | if show_item { 0x40 } else { 0 } | if item80 { 0x80 } else { 0 };
    }

    // --- flowers --------------------------------------------------------------
    pub fn genes(&self) -> u8 {
        self.free as u8
    }
    pub fn set_genes(&mut self, g: u8) {
        self.free = (self.free & !0xFF) | g as u32;
    }
    fn watered(&self) -> u16 {
        (self.free >> 8) as u16
    }
    fn set_watered(&mut self, w: u16) {
        self.free = (self.free & 0xFF0000FF) | (w as u32) << 8;
    }
    pub fn days_watered(&self) -> u8 {
        (self.watered() & 0x1F) as u8
    }
    pub fn set_days_watered(&mut self, d: u8) {
        let w = (self.watered() & 0xFFE0) | (d as u16 & 0x1F);
        self.set_watered(w);
    }
    pub fn is_watered_by_visitor(&self, v: usize) -> bool {
        v < 10 && self.watered() & (1 << (5 + v)) != 0
    }
    pub fn set_watered_by_visitor(&mut self, v: usize, val: bool) {
        if v >= 10 {
            return;
        }
        let bit = 1u16 << (5 + v);
        let w = if val { self.watered() | bit } else { self.watered() & !bit };
        self.set_watered(w);
    }
    pub fn is_watered(&self) -> bool {
        self.watered() & 0x8000 != 0
    }
    pub fn set_is_watered(&mut self, v: bool) {
        let w = (self.watered() & 0x7FFF) | if v { 0x8000 } else { 0 };
        self.set_watered(w);
    }
    pub fn is_watered_gold(&self) -> bool {
        (self.free >> 24) & 1 == 1
    }
    pub fn set_watered_gold(&mut self, v: bool) {
        self.free = (self.free & !0x0100_0000) | if v { 0x0100_0000 } else { 0 };
    }
    pub fn water(&mut self, all: bool) {
        if all {
            self.set_watered(0xFFFF);
            self.free = (self.free & 0x00FF_FFFF) | 0x0100_0000;
            return;
        }
        self.set_days_watered(31);
        self.set_is_watered(true);
    }

    // --- extensions ------------------------------------------------------------
    pub fn extension_item_id(&self) -> u16 {
        self.free as u16
    }
    pub fn extension_x(&self) -> u8 {
        (self.free >> 16) as u8
    }
    pub fn extension_y(&self) -> u8 {
        (self.free >> 24) as u8
    }
    pub fn extension(root_id: u16, x: u8, y: u8) -> Self {
        Self { id: Self::EXTENSION_ID, system: 0, additional: 0, free: root_id as u32 | (x as u32) << 16 | (y as u32) << 24 }
    }

    /// Item id displayed for a wrapped present (the wrapper itself).
    pub fn wrapped_item_name_id(&self) -> u16 {
        match self.wrapping_type() {
            1 => 0x1E13 + self.wrapping_paper() as u16,
            2 => 0x1180,
            3 => 0x1225,
            _ => self.id,
        }
    }

    pub fn kind(&self) -> u8 {
        item_kind(self.display_id())
    }

    /// Descriptive name (adds recipe / fossil / tree-drop / gene details where relevant).
    pub fn name(&self, s: &GameStrings) -> String {
        if self.is_none() {
            return s.item_name(Self::NONE_ID);
        }
        if self.is_extension() {
            return s.item_name(self.extension_item_id());
        }
        let id = self.id;
        let kd = kinds();
        let kind = item_kind(id);
        if kd.is_flower_gene(kind, id) && self.genes() != 0 {
            return format!("{} - {}", s.item_name(id), gene_text(self.genes()));
        }
        if kind == kd.diy_recipe || kind == kd.message_bottle {
            let recipe = self.free as u16;
            let makes = tables().recipe_map.get(&recipe).map(|r| s.item_name(*r)).unwrap_or_else(|| format!("{recipe:03}"));
            return format!("{} - {}", s.item_name(id), makes);
        }
        if kind == kd.fossil_unknown {
            return format!("{} - {}", s.item_name(id), s.item_name(self.free as u16));
        }
        if kind == kd.tree && self.count() != 0 {
            return format!("{} - {}", s.item_name(id), s.item_name(self.count()));
        }
        s.item_name(id)
    }

    /// Checks whether the player character could drop this item.
    pub fn is_droppable(&self) -> bool {
        if self.is_field_item() || self.is_extension() || self.is_none() || self.system > 3 {
            return false;
        }
        let kd = kinds();
        let k = self.kind();
        !(k == kd.insect || k == kd.dummy_presentbox || k == kd.fish || k == kd.dive_fish || k == kd.flower_bud || k == kd.bush || k == kd.tree)
    }
}

/// Human readable flower genes (e.g. `R1 R2 Y1 w s`).
pub fn gene_text(g: u8) -> String {
    let mut parts = Vec::new();
    let pair = |b0: u8, b1: u8, up: &str| -> String {
        let n = ((g & b0 != 0) as u8) + ((g & b1 != 0) as u8);
        match n {
            0 => up.to_lowercase().repeat(2),
            1 => format!("{}{}", up, up.to_lowercase()),
            _ => up.repeat(2),
        }
    };
    parts.push(pair(0x01, 0x02, "R"));
    parts.push(pair(0x04, 0x08, "Y"));
    // white gene is inverted: both bits set = no white (WW)
    parts.push(pair(0x10, 0x20, "W"));
    parts.push(pair(0x40, 0x80, "S"));
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_roundtrip() {
        let it = Item { id: 0x1234, system: 0x56, additional: 0x78, free: 0x9ABC_DEF0 };
        assert_eq!(Item::from_raw(it.raw()), it);
        let mut buf = [0u8; 8];
        it.write(&mut buf, 0);
        assert_eq!(buf, [0x34, 0x12, 0x56, 0x78, 0xF0, 0xDE, 0xBC, 0x9A]);
    }

    #[test]
    fn wrapping() {
        let mut it = Item::new(100);
        it.set_wrapping(4, 0, false, false);
        assert_eq!(it.wrapping_type(), 4);
        assert_eq!(it.additional, 3 | (15 << 2));
        it.set_wrapping(1, 7, true, false);
        assert_eq!(it.wrapping_type(), 1);
        assert_eq!(it.wrapping_paper(), 7);
        assert!(it.wrapping_show_item());
    }

    #[test]
    fn flowers() {
        let mut it = Item::new(100);
        it.set_genes(0b1010_0101);
        it.water(false);
        assert_eq!(it.genes(), 0b1010_0101);
        assert_eq!(it.days_watered(), 31);
        assert!(it.is_watered());
        it.set_watered_by_visitor(3, true);
        assert!(it.is_watered_by_visitor(3));
        it.water(true);
        assert!(it.is_watered_gold());
        assert_eq!(it.genes(), 0b1010_0101);
    }
}
