//! Island residents (`GSaveNpc`), formats 1 (≤1.x) and 2 (2.0+).
use crate::structs::misc::{GDate, PlayerId, RoomFloorWall};
use crate::structs::Item;
use crate::util::bytes::*;
use crate::util::strings::{get_string, set_string_if_changed};
use serde::{Deserialize, Serialize};

pub const MEMORY_COUNT: usize = 8;
pub const MEMORY_SIZE: usize = 0x5F0;
pub const VILLAGER_ITEM_SIZE: usize = 0x2C;
pub const WEAR_COUNT: usize = 24;
pub const FTR_COUNT: usize = 32;
pub const EVENT_FLAG_COUNT: usize = 0x100;
pub const GREETING_COUNT: usize = 11;
/// Greeting buffers are `char16[48]` (0x60 bytes); keep one unit for the terminator.
pub const GREETING_MAX: usize = 47;
/// Approximate number of characters the in-game speech bubble renders on one line.
pub const GREETING_RENDER_HINT: usize = 24;

/// Layout constants that differ between villager formats.
#[derive(Debug, Clone, Copy)]
pub struct VillagerLayout {
    pub size: usize,
    pub catchphrase: usize,
    pub wear: usize,
    pub ftr: usize,
    pub birth_type: usize,
    pub event_flags: usize,
    pub room: usize,
    pub design: usize,
    pub diy_end: usize,
    pub extension: &'static str,
}

pub const LAYOUT_V1: VillagerLayout = VillagerLayout {
    size: 0x12AB0,
    catchphrase: 0xFFE8 + 0x2C,
    wear: 0x101CC,
    ftr: 0x105EC,
    birth_type: 0x11EF8,
    event_flags: 0x11EFC,
    room: 0x12100,
    design: 0x12128,
    diy_end: 0x129D1,
    extension: "nhv",
};

pub const LAYOUT_V2: VillagerLayout = VillagerLayout {
    size: 0x13230,
    catchphrase: 0x10768 + 0x2C,
    wear: 0x1094C,
    ftr: 0x10D6C,
    birth_type: 0x12678,
    event_flags: 0x1267C,
    room: 0x12880,
    design: 0x128A8,
    diy_end: 0x13151,
    extension: "nhv2",
};

pub fn layout(format: u8) -> &'static VillagerLayout {
    if format >= 2 {
        &LAYOUT_V2
    } else {
        &LAYOUT_V1
    }
}

/// Internal villager name, e.g. species 4 variant 0 → `cat00`.
pub fn internal_name(species: u8, variant: u8) -> String {
    let sp = crate::data::tables::enum_name("VillagerSpecies", species as i64).unwrap_or("non");
    format!("{sp}{variant:02}")
}

/// Parses an internal name back into (species, variant).
pub fn parse_internal_name(name: &str) -> Option<(u8, u8)> {
    if name.len() < 4 {
        return None;
    }
    let (sp, var) = name.split_at(3);
    let species = crate::data::tables::enum_value("VillagerSpecies", sp)? as u8;
    let variant: u8 = var.parse().ok()?;
    Some((species, variant))
}

/// A villager's memory of one player (`GSaveMemory`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub player: PlayerId,
    pub friendship: u8,
    pub nickname: String,
    pub greetings: Vec<String>,
    pub greeting_set_date: GDate,
    /// Raw event flag bytes (0x100).
    pub flags: Vec<u8>,
}

impl Memory {
    fn read(d: &[u8], o: usize) -> Self {
        Self {
            player: PlayerId::read(d, o),
            friendship: d[o + 0x38 + 10],
            nickname: get_string(d, o + 0x138, 10),
            greetings: (0..GREETING_COUNT)
                .map(|i| get_string(d, o + 0x14C + 0x60 * i, GREETING_MAX))
                .collect(),
            greeting_set_date: GDate::read(d, o + 0x56C),
            flags: d[o + 0x38..o + 0x138].to_vec(),
        }
    }

    fn write(&self, d: &mut [u8], o: usize) {
        self.player.write(d, o);
        if self.flags.len() == 0x100 {
            d[o + 0x38..o + 0x138].copy_from_slice(&self.flags);
        }
        d[o + 0x38 + 10] = self.friendship;
        set_string_if_changed(d, o + 0x138, 10, &self.nickname);
        for (i, g) in self.greetings.iter().enumerate().take(GREETING_COUNT) {
            let off = o + 0x14C + 0x60 * i;
            let g = truncate_units(g, GREETING_MAX);
            if get_string(d, off, GREETING_MAX) != g {
                // 47 units + the zero terminator (the field holds 48).
                crate::util::strings::set_string(d, off, GREETING_MAX + 1, &g);
            }
        }
        self.greeting_set_date.write(d, o + 0x56C);
    }
}

fn truncate_units(s: &str, max: usize) -> String {
    let mut out = String::new();
    let mut n = 0;
    for c in s.chars() {
        let l = c.len_utf16();
        if n + l > max {
            break;
        }
        n += l;
        out.push(c);
    }
    out
}

/// Editable villager summary. Raw data is kept separately in the save.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Villager {
    pub index: usize,
    pub format: u8,
    pub species: u8,
    pub variant: u8,
    pub internal_name: String,
    pub personality: u8,
    pub catchphrase: String,
    pub birth_type: u8,
    pub inducement_type: u8,
    pub move_type: u8,
    pub moving_out: bool,
    pub diy_end: [u8; 3],
    pub diy_recipe: u16,
    pub room: RoomFloorWall,
    pub wear: Vec<Item>,
    pub furniture: Vec<Item>,
    pub memories: Vec<Memory>,
    pub event_flags: Vec<u16>,
}

impl Villager {
    pub fn read(d: &[u8], format: u8, index: usize) -> Self {
        let l = layout(format);
        let species = d[0];
        let variant = d[1];
        let move_type = d[l.birth_type + 2];
        Self {
            index,
            format,
            species,
            variant,
            internal_name: internal_name(species, variant),
            personality: d[2],
            catchphrase: get_string(d, l.catchphrase, 24),
            birth_type: d[l.birth_type],
            inducement_type: d[l.birth_type + 1],
            move_type,
            moving_out: move_type & 2 == 2,
            diy_end: [d[l.diy_end], d[l.diy_end + 1], d[l.diy_end + 2]],
            diy_recipe: u16le(d, l.diy_end + 3),
            room: RoomFloorWall::read(d, l.room),
            wear: (0..WEAR_COUNT)
                .map(|i| Item::read(d, l.wear + i * VILLAGER_ITEM_SIZE))
                .collect(),
            furniture: (0..FTR_COUNT)
                .map(|i| Item::read(d, l.ftr + i * VILLAGER_ITEM_SIZE))
                .collect(),
            memories: (0..MEMORY_COUNT)
                .map(|i| Memory::read(d, 4 + i * MEMORY_SIZE))
                .collect(),
            event_flags: read_u16_array(&d[l.event_flags..l.event_flags + EVENT_FLAG_COUNT * 2]),
        }
    }

    pub fn write(&self, d: &mut [u8]) {
        let l = layout(self.format);
        d[0] = self.species;
        d[1] = self.variant;
        d[2] = self.personality;
        set_string_if_changed(d, l.catchphrase, 24, &self.catchphrase);
        d[l.birth_type] = self.birth_type;
        d[l.birth_type + 1] = self.inducement_type;
        d[l.birth_type + 2] = (self.move_type & !2) | if self.moving_out { 2 } else { 0 };
        d[l.diy_end..l.diy_end + 3].copy_from_slice(&self.diy_end);
        put_u16(d, l.diy_end + 3, self.diy_recipe);
        self.room.write(d, l.room);
        for (i, it) in self.wear.iter().enumerate().take(WEAR_COUNT) {
            write_villager_item(d, l.wear + i * VILLAGER_ITEM_SIZE, it);
        }
        for (i, it) in self.furniture.iter().enumerate().take(FTR_COUNT) {
            write_villager_item(d, l.ftr + i * VILLAGER_ITEM_SIZE, it);
        }
        for (i, m) in self.memories.iter().enumerate().take(MEMORY_COUNT) {
            m.write(d, 4 + i * MEMORY_SIZE);
        }
        if self.event_flags.len() == EVENT_FLAG_COUNT {
            write_u16_array(
                &mut d[l.event_flags..l.event_flags + EVENT_FLAG_COUNT * 2],
                &self.event_flags,
            );
        }
    }

    /// 0 = male, 1 = female.
    pub fn gender(&self) -> u8 {
        (self.personality / 4) & 1
    }

    pub fn is_empty(&self) -> bool {
        // species "non" (35) marks an empty slot
        self.species == 35 || self.species == 0xFF
    }
}

/// Writes the 8-byte item head of a 0x2C `VillagerItem`; clearing an item also clears its tail.
fn write_villager_item(d: &mut [u8], o: usize, it: &Item) {
    let prev = Item::read(d, o);
    it.write(d, o);
    if it.is_none() || prev.id != it.id {
        d[o + 8..o + VILLAGER_ITEM_SIZE].fill(0);
    }
}

/// Byte range of player identity (id + name) inside a memory, for identity replacement.
pub fn memory_player_identity(d: &[u8], memory: usize) -> &[u8] {
    let o = 4 + memory * MEMORY_SIZE + 0x1C;
    &d[o..o + 24]
}

/// Raw design pattern block (PRO) stored in the villager.
pub fn design_range(format: u8) -> std::ops::Range<usize> {
    let l = layout(format);
    l.design..l.design + crate::structs::design::PRO_SIZE
}
