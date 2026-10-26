//! Conversions between data formats of different game versions (for imports of dumps).
use crate::structs::house::*;
use crate::structs::villager::{LAYOUT_V1, LAYOUT_V2};
use crate::structs::Item;

const LIGHT_MEMORY_START: usize = 0x2F84;
const LIGHT_MEMORY_OLD: usize = 0x14C;
const LIGHT_MEMORY_NEW: usize = 0x158;
const LIGHT_MEMORY_COUNT: usize = 160;

/// Converts a villager dump to `expect` bytes (v1 ↔ v2) if needed.
pub fn villager_compatible(data: &[u8], expect: usize) -> Option<Vec<u8>> {
    let (v1, v2) = (LAYOUT_V1.size, LAYOUT_V2.size);
    if data.len() == expect {
        return Some(data.to_vec());
    }
    if expect == v2 && data.len() == v1 {
        let mut out = vec![0u8; v2];
        out[..LIGHT_MEMORY_START].copy_from_slice(&data[..LIGHT_MEMORY_START]);
        for i in 0..LIGHT_MEMORY_COUNT {
            let s = LIGHT_MEMORY_START + LIGHT_MEMORY_OLD * i;
            let d = LIGHT_MEMORY_START + LIGHT_MEMORY_NEW * i;
            out[d..d + LIGHT_MEMORY_OLD].copy_from_slice(&data[s..s + LIGHT_MEMORY_OLD]);
        }
        out[0x10684..].copy_from_slice(&data[0xFF04..]);
        return Some(out);
    }
    if expect == v1 && data.len() == v2 {
        let mut out = vec![0u8; v1];
        out[..LIGHT_MEMORY_START].copy_from_slice(&data[..LIGHT_MEMORY_START]);
        for i in 0..LIGHT_MEMORY_COUNT {
            let s = LIGHT_MEMORY_START + LIGHT_MEMORY_OLD * i;
            let d = LIGHT_MEMORY_START + LIGHT_MEMORY_NEW * i;
            out[s..s + LIGHT_MEMORY_OLD].copy_from_slice(&data[d..d + LIGHT_MEMORY_OLD]);
        }
        out[0xFF04..].copy_from_slice(&data[0x10684..]);
        return Some(out);
    }
    None
}

const VH2_FOOTER: [u8; 120] = [
    0x4B, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x4B, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x9B, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF8, 0x00, 0xF8, 0x00, 0xF8, 0x00, 0x40,
    0x10, 0x00, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFE, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xFE, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFE, 0xFF, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Converts a villager house dump to `expect` bytes (v1 ↔ v2).
pub fn villager_house_compatible(data: &[u8], expect: usize) -> Option<Vec<u8>> {
    if data.len() == expect {
        return Some(data.to_vec());
    }
    if expect == VILLAGER_HOUSE2_SIZE && data.len() == VILLAGER_HOUSE1_SIZE {
        let mut out = vec![0u8; VILLAGER_HOUSE2_SIZE];
        out[..VILLAGER_HOUSE1_SIZE].copy_from_slice(data);
        for i in 0..236 {
            Item::NONE.write(&mut out, 0x1D8 + i * 0xC);
        }
        out[0x1270..0x1270 + VH2_FOOTER.len()].copy_from_slice(&VH2_FOOTER);
        return Some(out);
    }
    if expect == VILLAGER_HOUSE1_SIZE && data.len() == VILLAGER_HOUSE2_SIZE {
        return Some(data[..VILLAGER_HOUSE1_SIZE].to_vec());
    }
    None
}

fn room12(r1: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; PLAYER_ROOM2_SIZE];
    out[..PLAYER_ROOM1_SIZE].copy_from_slice(r1);
    out
}

/// Converts a player room dump to `expect` bytes.
pub fn room_compatible(data: &[u8], expect: usize) -> Option<Vec<u8>> {
    if data.len() == expect {
        return Some(data.to_vec());
    }
    if expect == PLAYER_ROOM2_SIZE && data.len() == PLAYER_ROOM1_SIZE {
        return Some(room12(data));
    }
    if expect == PLAYER_ROOM1_SIZE && data.len() == PLAYER_ROOM2_SIZE {
        return Some(data[..PLAYER_ROOM1_SIZE].to_vec());
    }
    None
}

/// Converts a player house dump to `expect` bytes.
pub fn player_house_compatible(data: &[u8], expect: usize) -> Option<Vec<u8>> {
    if data.len() == expect {
        return Some(data.to_vec());
    }
    if expect == PLAYER_HOUSE2_SIZE && data.len() == PLAYER_HOUSE1_SIZE {
        let mut out = vec![0u8; PLAYER_HOUSE2_SIZE];
        out[..0x120].copy_from_slice(&data[..0x120]);
        for i in 0..ROOM_COUNT {
            let s = ROOM_START + i * PLAYER_ROOM1_SIZE;
            let d = ROOM_START + i * PLAYER_ROOM2_SIZE;
            out[d..d + PLAYER_ROOM2_SIZE].copy_from_slice(&room12(&data[s..s + PLAYER_ROOM1_SIZE]));
        }
        out[0x289F8..0x289F8 + 0x30].copy_from_slice(&data[0x263D0..0x263D0 + 0x30]);
        return Some(out);
    }
    if expect == PLAYER_HOUSE1_SIZE && data.len() == PLAYER_HOUSE2_SIZE {
        let mut out = vec![0u8; PLAYER_HOUSE1_SIZE];
        out[..0x120].copy_from_slice(&data[..0x120]);
        for i in 0..ROOM_COUNT {
            let s = ROOM_START + i * PLAYER_ROOM2_SIZE;
            let d = ROOM_START + i * PLAYER_ROOM1_SIZE;
            out[d..d + PLAYER_ROOM1_SIZE].copy_from_slice(&data[s..s + PLAYER_ROOM1_SIZE]);
        }
        out[0x263D0..0x263D0 + 0x30].copy_from_slice(&data[0x289F8..0x289F8 + 0x30]);
        return Some(out);
    }
    None
}

const ACRE_COLUMN_ITEMS: usize = 6 * 32 * 32;

/// Converts a whole field item layer dump between the 7- and 9-acre wide layouts (column-major).
pub fn field_layer_compatible(data: &[u8], expect: usize) -> Option<Vec<u8>> {
    let col = ACRE_COLUMN_ITEMS * 8;
    let (old, new) = (7 * col, 9 * col);
    if data.len() == expect {
        return Some(data.to_vec());
    }
    if expect == new && data.len() == old {
        let mut out = Vec::with_capacity(new);
        let none = Item::NONE.raw().to_le_bytes();
        for _ in 0..ACRE_COLUMN_ITEMS {
            out.extend_from_slice(&none);
        }
        out.extend_from_slice(data);
        for _ in 0..ACRE_COLUMN_ITEMS {
            out.extend_from_slice(&none);
        }
        return Some(out);
    }
    if expect == old && data.len() == new {
        return Some(data[col..col + old].to_vec());
    }
    None
}

/// Converts a field item flag dump (row-major bits) between 7- and 9-acre layouts.
pub fn field_flags_compatible(data: &[u8], expect: usize) -> Option<Vec<u8>> {
    const ROWS: usize = 6 * 32;
    const OLD_ROW: usize = 7 * 32 / 8;
    const NEW_ROW: usize = 9 * 32 / 8;
    if data.len() == expect {
        return Some(data.to_vec());
    }
    if expect == ROWS * NEW_ROW && data.len() == ROWS * OLD_ROW {
        let mut out = vec![0u8; ROWS * NEW_ROW];
        for r in 0..ROWS {
            out[r * NEW_ROW + 4..r * NEW_ROW + 4 + OLD_ROW]
                .copy_from_slice(&data[r * OLD_ROW..(r + 1) * OLD_ROW]);
        }
        return Some(out);
    }
    if expect == ROWS * OLD_ROW && data.len() == ROWS * NEW_ROW {
        let mut out = vec![0u8; ROWS * OLD_ROW];
        for r in 0..ROWS {
            out[r * OLD_ROW..(r + 1) * OLD_ROW]
                .copy_from_slice(&data[r * NEW_ROW + 4..r * NEW_ROW + 4 + OLD_ROW]);
        }
        return Some(out);
    }
    None
}

/// Cheat code text (`04100000 OFFSET VALUE` lines) for a list of items.
pub fn items_to_cheat(items: &[Item], offset: u32) -> String {
    let mut out = String::new();
    for (i, it) in items.iter().enumerate() {
        let raw = it.raw();
        out.push_str(&format!(
            "04100000 {:08X} {:08X}\n",
            offset + (i as u32) * 8,
            raw as u32
        ));
        out.push_str(&format!(
            "04100000 {:08X} {:08X}\n",
            offset + (i as u32) * 8 + 4,
            (raw >> 32) as u32
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn villager_roundtrip() {
        let mut v1 = vec![0u8; LAYOUT_V1.size];
        for (i, b) in v1.iter_mut().enumerate() {
            *b = (i % 251) as u8;
        }
        let v2 = villager_compatible(&v1, LAYOUT_V2.size).unwrap();
        assert_eq!(villager_compatible(&v2, LAYOUT_V1.size).unwrap(), v1);
    }
    #[test]
    fn field_roundtrip() {
        let old: Vec<u8> = (0..7 * ACRE_COLUMN_ITEMS * 8)
            .map(|i| (i % 7) as u8)
            .collect();
        let new = field_layer_compatible(&old, 9 * ACRE_COLUMN_ITEMS * 8).unwrap();
        assert_eq!(field_layer_compatible(&new, old.len()).unwrap(), old);
        let flags: Vec<u8> = (0..192 * 28).map(|i| i as u8).collect();
        let nf = field_flags_compatible(&flags, 192 * 36).unwrap();
        assert_eq!(field_flags_compatible(&nf, flags.len()).unwrap(), flags);
    }
}
