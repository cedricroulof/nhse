//! Villager houses, player houses and their rooms.
use crate::save::EncryptedInt32;
use crate::structs::misc::RoomFloorWall;
use crate::structs::Item;
use crate::util::bits::{get_flag, set_flag};
use crate::util::bytes::*;
use serde::{Deserialize, Serialize};

pub const VILLAGER_HOUSE1_SIZE: usize = 0x1D4;
pub const VILLAGER_HOUSE2_SIZE: usize = 0x12E8;
pub const PLAYER_HOUSE1_SIZE: usize = 0x26400;
pub const PLAYER_HOUSE2_SIZE: usize = 0x28A28;
pub const PLAYER_ROOM1_SIZE: usize = 0x65C8;
pub const PLAYER_ROOM2_SIZE: usize = 0x6C24;
pub const ROOM_COUNT: usize = 6;
pub const ROOM_START: usize = 0x120;
pub const ROOM_LAYERS: usize = 8;
pub const ROOM_DIM: usize = 20;
pub const ROOM_LAYER_SIZE: usize = ROOM_DIM * ROOM_DIM * 8;

/// Exterior appearance shared by villager and player houses.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct HouseExterior {
    pub level: u32,
    pub wall: u16,
    pub roof: u16,
    pub door: u16,
    pub order_wall: u16,
    pub order_roof: u16,
    pub order_door: u16,
    pub door_deco: Item,
}

/// Villager house (`GSaveNpcHouse`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VillagerHouse {
    pub index: usize,
    pub format: u8,
    pub exterior: HouseExterior,
    pub status: u32,
    pub npc1: i8,
    pub npc2: i8,
    pub build_player: i8,
    /// Interior room surfaces (2.0+ only).
    pub room: Option<RoomFloorWall>,
}

impl VillagerHouse {
    pub fn read(d: &[u8], format: u8, index: usize) -> Self {
        Self {
            index,
            format,
            exterior: HouseExterior {
                level: u32le(d, 0),
                wall: u16le(d, 0x08),
                roof: u16le(d, 0x0A),
                door: u16le(d, 0x0C),
                order_wall: u16le(d, 0x0E),
                order_roof: u16le(d, 0x10),
                order_door: u16le(d, 0x12),
                door_deco: Item::read(d, 0x1C8),
            },
            status: u32le(d, 4),
            npc1: d[0x1C4] as i8,
            npc2: d[0x1C5] as i8,
            build_player: d[0x1D0] as i8,
            room: (format >= 2).then(|| RoomFloorWall::read(d, 0x1D4 + 0x109C)),
        }
    }

    pub fn write(&self, d: &mut [u8]) {
        let e = &self.exterior;
        put_u32(d, 0, e.level);
        put_u32(d, 4, self.status);
        put_u16(d, 0x08, e.wall);
        put_u16(d, 0x0A, e.roof);
        put_u16(d, 0x0C, e.door);
        put_u16(d, 0x0E, e.order_wall);
        put_u16(d, 0x10, e.order_roof);
        put_u16(d, 0x12, e.order_door);
        e.door_deco.write(d, 0x1C8);
        d[0x1C4] = self.npc1 as u8;
        d[0x1C5] = self.npc2 as u8;
        d[0x1D0] = self.build_player as u8;
        if let (Some(r), true) = (&self.room, self.format >= 2) {
            r.write(d, 0x1D4 + 0x109C);
        }
    }
}

/// Offsets inside a player house that differ by format.
struct PlayerHouseLayout {
    room_size: usize,
    npc: usize,
}

fn ph_layout(format: u8) -> PlayerHouseLayout {
    if format >= 2 {
        PlayerHouseLayout { room_size: PLAYER_ROOM2_SIZE, npc: 0x289F8 }
    } else {
        PlayerHouseLayout { room_size: PLAYER_ROOM1_SIZE, npc: 0x263D0 }
    }
}

/// Player house (`GSavePlayerHouse`).
///
/// Note: the original editor read NPC/door-deco/mailbox fields of 2.0+ houses at the 1.x
/// offsets (interface dispatch hid the overrides). Offsets here are format-correct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerHouse {
    pub index: usize,
    pub format: u8,
    pub exterior: HouseExterior,
    pub loan: u32,
    pub npc1: i8,
    pub npc2: i8,
    pub player_house_flag: bool,
    pub mailbox: Item,
    pub order_mailbox: Item,
    pub event_flags: Vec<i16>,
}

impl PlayerHouse {
    pub fn read(d: &[u8], format: u8, index: usize) -> Self {
        let l = ph_layout(format);
        Self {
            index,
            format,
            exterior: HouseExterior {
                level: u32le(d, 0),
                wall: u16le(d, 0x04),
                roof: u16le(d, 0x06),
                door: u16le(d, 0x08),
                order_wall: u16le(d, 0x0A),
                order_roof: u16le(d, 0x0C),
                order_door: u16le(d, 0x0E),
                door_deco: Item::read(d, l.npc + 4),
            },
            loan: EncryptedInt32::read(d, 0x10).value,
            npc1: d[l.npc] as i8,
            npc2: d[l.npc + 1] as i8,
            player_house_flag: d[l.npc + 0xC] != 0,
            mailbox: Item::read(d, l.npc + 0x10),
            order_mailbox: Item::read(d, l.npc + 0x18),
            event_flags: read_i16_array(&d[0x20..0x120]),
        }
    }

    pub fn write(&self, d: &mut [u8]) {
        let l = ph_layout(self.format);
        let e = &self.exterior;
        put_u32(d, 0, e.level);
        put_u16(d, 0x04, e.wall);
        put_u16(d, 0x06, e.roof);
        put_u16(d, 0x08, e.door);
        put_u16(d, 0x0A, e.order_wall);
        put_u16(d, 0x0C, e.order_roof);
        put_u16(d, 0x0E, e.order_door);
        EncryptedInt32::set_value(d, 0x10, self.loan);
        d[l.npc] = self.npc1 as u8;
        d[l.npc + 1] = self.npc2 as u8;
        e.door_deco.write(d, l.npc + 4);
        put_bool(d, l.npc + 0xC, self.player_house_flag);
        self.mailbox.write(d, l.npc + 0x10);
        self.order_mailbox.write(d, l.npc + 0x18);
        if self.event_flags.len() == 0x80 {
            write_i16_array(&mut d[0x20..0x120], &self.event_flags);
        }
    }

    pub fn room_range(format: u8, room: usize) -> std::ops::Range<usize> {
        let l = ph_layout(format);
        let o = ROOM_START + room * l.room_size;
        o..o + l.room_size
    }
}

/// A room of a player house: 8 item layers of 20×20 plus active flags and surfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRoom {
    /// `layers[layer][x * 20 + y]` (column major, like the field item grid).
    pub layers: Vec<Vec<Item>>,
    pub surfaces: RoomFloorWall,
    pub music: i16,
    pub music_shuffle: bool,
    pub ambient: Option<u8>,
}

impl PlayerRoom {
    pub fn read(d: &[u8], format: u8) -> Self {
        Self {
            layers: (0..ROOM_LAYERS).map(|l| Item::read_array(d, l * ROOM_LAYER_SIZE, ROOM_DIM * ROOM_DIM)).collect(),
            surfaces: RoomFloorWall::read(d, 0x65A4),
            music: i16le(d, 0x65A0),
            music_shuffle: d[0x65A3] != 0,
            ambient: (format >= 2).then(|| d[0x6C0C]),
        }
    }

    pub fn write(&self, d: &mut [u8], format: u8) {
        for (l, layer) in self.layers.iter().enumerate().take(ROOM_LAYERS) {
            Item::write_array(d, l * ROOM_LAYER_SIZE, layer);
        }
        self.surfaces.write(d, 0x65A4);
        put_i16(d, 0x65A0, self.music);
        put_bool(d, 0x65A3, self.music_shuffle);
        if let (Some(a), true) = (self.ambient, format >= 2) {
            d[0x6C0C] = a;
        }
    }

    pub fn is_active(d: &[u8], layer: usize, x: usize, y: usize) -> bool {
        get_flag(d, 0x6400 + layer * 0x34, y * ROOM_DIM + x)
    }

    pub fn set_active(d: &mut [u8], layer: usize, x: usize, y: usize, v: bool) {
        set_flag(d, 0x6400 + layer * 0x34, y * ROOM_DIM + x, v)
    }
}
