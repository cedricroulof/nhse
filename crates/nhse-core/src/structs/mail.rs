//! Mail stored in `postbox.dat`.
use crate::structs::misc::GDate;
use crate::structs::Item;
use crate::util::bytes::*;
use crate::util::strings::{get_string, set_string_if_changed};
use serde::{Deserialize, Serialize};

pub const SIZE: usize = 0x3C8;
pub const SLOTS: usize = 300;
pub const TO_LEN: usize = 47;
pub const MESSAGE_LEN: usize = 239;
pub const MESSAGE_LEN_OFFICIAL: usize = 168;
pub const FROM_LEN: usize = 47;

/// One letter / parcel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mail {
    pub slot: usize,
    /// 0 = empty, 3 = unread, 4 = read.
    pub status: u8,
    pub favorite: bool,
    /// 0 = player, 1 = villager, 2 = NPC.
    pub sender_type: u8,
    pub sender_island_id: u32,
    pub sender_island_name: String,
    pub sender_species: u8,
    pub sender_variant: u8,
    pub sender_id: u32,
    pub sender_name: String,
    pub receiver_island_id: u32,
    pub receiver_island_name: String,
    pub receiver_player_id: u32,
    pub receiver_player_name: String,
    pub date: GDate,
    pub item: Item,
    pub wrapping: u8,
    pub is_letter: bool,
    pub to_line: String,
    pub message: String,
    pub from_line: String,
    pub stationery: u16,
    pub sort_index: u16,
    pub language: String,
}

impl Mail {
    pub fn is_used(&self) -> bool {
        self.status == 3 || self.status == 4
    }

    pub fn read(d: &[u8], o: usize, slot: usize) -> Self {
        let sender_type = d[o + 0x5C];
        let player = sender_type == 0;
        let lang_bytes = &d[o + 0x390..o + 0x395];
        Self {
            slot,
            status: d[o],
            favorite: d[o + 1] != 0,
            sender_type,
            sender_island_id: if player {
                u32le(d, o + 4)
            } else {
                u32le(d, o + 8)
            },
            sender_island_name: if player {
                get_string(d, o + 8, 10)
            } else {
                get_string(d, o + 0xC, 10)
            },
            sender_species: d[o + 4],
            sender_variant: d[o + 5],
            sender_id: u32le(d, o + 0x20),
            sender_name: get_string(d, o + 0x24, 10),
            receiver_island_id: u32le(d, o + 0x60),
            receiver_island_name: get_string(d, o + 0x64, 10),
            receiver_player_id: u32le(d, o + 0x7C),
            receiver_player_name: get_string(d, o + 0x80, 10),
            date: GDate::read(d, o + 0xBC),
            item: Item::read(d, o + 0xC0),
            wrapping: d[o + 0xC3],
            is_letter: d[o + 0xC8] == 0 && d[o + 0xCC] == 1,
            to_line: get_string(d, o + 0xCE, TO_LEN),
            message: get_string(d, o + 0x12E, MESSAGE_LEN),
            from_line: get_string(d, o + 0x30E, FROM_LEN),
            stationery: u16le(d, o + 0x36E),
            sort_index: u16le(d, o + 0x374),
            language: lang_bytes
                .iter()
                .take_while(|b| **b != 0)
                .map(|b| *b as char)
                .collect(),
        }
    }

    /// Writes all editable fields (sender/receiver/identity region is rebuilt from scratch).
    pub fn write(&self, d: &mut [u8], o: usize) {
        // clear transmit/receive region (0x00..0x60) like the original editor
        d[o..o + 0x60].fill(0);
        d[o] = self.status;
        put_bool(d, o + 1, self.favorite);
        d[o + 0x5C] = self.sender_type;
        match self.sender_type {
            0 => {
                put_u32(d, o + 4, self.sender_island_id);
                set_string_if_changed(d, o + 8, 10, &self.sender_island_name);
            }
            _ => {
                d[o + 4] = self.sender_species;
                d[o + 5] = self.sender_variant;
                if self.sender_type == 1 {
                    put_u32(d, o + 8, self.sender_island_id);
                    set_string_if_changed(d, o + 0xC, 10, &self.sender_island_name);
                }
            }
        }
        put_u32(d, o + 0x20, self.sender_id);
        set_string_if_changed(d, o + 0x24, 10, &self.sender_name);
        put_u32(d, o + 0x60, self.receiver_island_id);
        set_string_if_changed(d, o + 0x64, 10, &self.receiver_island_name);
        put_u32(d, o + 0x7C, self.receiver_player_id);
        set_string_if_changed(d, o + 0x80, 10, &self.receiver_player_name);
        self.date.write(d, o + 0xBC);
        self.item.write(d, o + 0xC0);
        d[o + 0xC3] = self.wrapping;
        if self.is_letter {
            d[o + 0xC8] = 0;
            d[o + 0xCC] = 1;
        } else {
            d[o + 0xC8] = 1;
            d[o + 0xCC] = 0;
        }
        set_string_if_changed(d, o + 0xCE, TO_LEN, &self.to_line.replace(['\r', '\n'], ""));
        set_string_if_changed(
            d,
            o + 0x12E,
            MESSAGE_LEN,
            &self.message.replace("\r\n", "\n"),
        );
        set_string_if_changed(
            d,
            o + 0x30E,
            FROM_LEN,
            &self.from_line.replace(['\r', '\n'], ""),
        );
        put_u16(
            d,
            o + 0x36E,
            if self.is_letter { self.stationery } else { 0 },
        );
        put_u16(d, o + 0x374, self.sort_index);
        let lang = &mut d[o + 0x390..o + 0x395];
        lang.fill(0);
        if self.sender_type == 0 {
            for (i, b) in self.language.bytes().take(5).enumerate() {
                lang[i] = b;
            }
        }
    }

    /// Writes an empty slot (matches what the game uses for unused entries).
    pub fn clear(d: &mut [u8], o: usize) {
        d[o..o + SIZE].fill(0);
        put_u16(d, o + 0xC0, 0xFFFE);
        put_u16(d, o + 0x370, 0xFFFE);
    }
}

/// Known NPC senders: (species byte value, image key, display name).
pub const NPC_SENDERS: &[(u8, &str, &str)] = &[
    (22, "rco", "Tom Nook"),
    (25, "rco", "Nook Inc."),
    (40, "bey", "C.J."),
    (41, "chy", "Flick"),
    (42, "pyn", "Zipper"),
    (43, "fox", "Redd"),
    (44, "alp", "Reese & Cyrus"),
    (45, "gulB", "Gullivarrr"),
    (46, "tap", "Luna"),
    (47, "pkn", "Jack"),
    (48, "xct", "Rover"),
    (50, "bpt", "Katrina"),
    (51, "slo", "Leif"),
    (52, "spn", "Harvey"),
    (53, "man", "Wardell"),
    (54, "OneroomCardboard", "Paradise Planning"),
    (55, "otg", "Lottie"),
    (56, "mnc", "Niko"),
    (57, "kpg", "Hotel"),
    (60, "gul", "Gulliver"),
    (61, "gstA", "Wisp"),
    (62, "hgc", "Label"),
    (63, "tkkA", "K.K."),
    (64, "boc", "Daisy Mae"),
    (65, "SnowCrystal", "Snowboy"),
    (68, "mol", "Resetti"),
    (80, "ott", "Happy Home Academy"),
    (81, "Leaf", "Nintendo"),
    (82, "MoneyBag010", "Bank of Nook"),
    (84, "Cardboard", "Nook Shopping"),
    (85, "PlaneTicket", "Nook Mileage Program"),
    (86, "Post", "Mom"),
    (87, "dod", "Dodo Airlines"),
    (88, "owl", "The Farway Museum"),
    (89, "Leaf", "NookLink Admins"),
];

/// Stationery skin codes in in-game order (index = `mailNN` string key).
pub const STATIONERY: &[u16] = &[
    0, 356, 357, 358, 359, 360, 361, 362, 363, 364, 365, 366, 367, 368, 370, 371, 372, 373, 374,
    375, 376, 377, 378, 379, 380, 381, 382, 383, 384, 385, 386, 387, 388, 389, 390, 391, 392, 393,
    394, 395, 396, 397, 398, 399, 402, 403, 404, 405, 406, 407, 408, 409, 410, 411, 412, 413, 414,
    416, 427, 428, 429, 430, 431, 432, 433, 434, 435, 437, 438, 439, 440, 443, 444, 445, 448,
];

/// Parcel box skins (wrapping byte, name).
pub const BOX_SKINS: &[(u8, &str)] = &[
    (0x00, "Standard box"),
    (0x43, "Nook Shopping box"),
    (0x6B, "Hotel box"),
    (0x6F, "Paradise Planning box"),
];

pub const LANGUAGES: &[&str] = &[
    "en-US", "en-GB", "ja-JP", "de-DE", "es-ES", "fr-FR", "it-IT", "ko-KR", "zh-CN", "zh-TW",
];
