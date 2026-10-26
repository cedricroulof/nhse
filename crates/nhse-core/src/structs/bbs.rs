//! Bulletin board posts (`GSaveBulletinBoard`).
use crate::structs::misc::{GDate, PlayerId};
use crate::util::bytes::*;
use crate::util::strings::{get_string, set_string_if_changed};
use serde::{Deserialize, Serialize};

pub const BOARD_SIZE: usize = 0xE0AE8;
pub const POST_SIZE: usize = 0x77D0;
pub const POST_COUNT: usize = 30;
pub const BODY_LEN: usize = 240;
pub const FOOTER_LEN: usize = 48;
const STOCK: usize = 8;
const INDEX_TABLE: usize = STOCK + POST_SIZE * POST_COUNT;
pub const HANDWRITING_BYTES: usize = 30_000;
/// Handwriting canvas: 2 bits per pixel, 400×300 (the game's message board canvas).
pub const HANDWRITING_W: usize = 400;
pub const HANDWRITING_H: usize = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub slot: usize,
    pub date: GDate,
    pub body: String,
    pub footer: String,
    pub author: PlayerId,
    pub design_id: u16,
    pub unique_id: u32,
    pub has_handwriting: bool,
    pub index_table: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub built: GDate,
    pub latest_unique_id: u32,
    pub posts: Vec<Post>,
}

fn post_offset(base: usize, slot: usize) -> usize {
    base + STOCK + slot * POST_SIZE
}

impl Board {
    pub fn read(d: &[u8], o: usize) -> Self {
        let posts = (0..POST_COUNT)
            .map(|i| {
                let p = post_offset(o, i);
                Post {
                    slot: i,
                    date: GDate::read(d, p),
                    body: get_string(d, p + 4, BODY_LEN),
                    footer: get_string(d, p + 0x1E4, FOOTER_LEN),
                    author: PlayerId::read(d, p + 0x777C),
                    design_id: u16le(d, p + 0x77B6),
                    unique_id: u32le(d, p + 0x77C8),
                    has_handwriting: d[p + 0x244..p + 0x244 + HANDWRITING_BYTES]
                        .iter()
                        .any(|b| *b != 0),
                    index_table: i32le(d, o + INDEX_TABLE + i * 4),
                }
            })
            .collect();
        Self {
            built: GDate::read(d, o),
            latest_unique_id: u32le(d, o + 0xE0AE0),
            posts,
        }
    }

    /// Updates text fields of posts (the drawing, ids and author are preserved).
    pub fn write_text(&self, d: &mut [u8], o: usize) {
        for p in &self.posts {
            let po = post_offset(o, p.slot);
            p.date.write(d, po);
            set_string_if_changed(d, po + 4, BODY_LEN, &p.body);
            set_string_if_changed(d, po + 0x1E4, FOOTER_LEN, &p.footer);
        }
    }

    pub fn post_bytes(d: &[u8], o: usize, slot: usize) -> &[u8] {
        let p = post_offset(o, slot);
        &d[p..p + POST_SIZE]
    }

    pub fn set_post_bytes(d: &mut [u8], o: usize, slot: usize, bytes: &[u8]) {
        let p = post_offset(o, slot);
        d[p..p + POST_SIZE].copy_from_slice(&bytes[..POST_SIZE]);
    }

    /// Renders a post's handwriting layer as RGBA using its 4-entry palette (index 0 transparent).
    pub fn handwriting_rgba(d: &[u8], o: usize, slot: usize) -> Vec<u8> {
        let p = post_offset(o, slot) + 0x244;
        let img = &d[p..p + HANDWRITING_BYTES];
        let pal = &d[p + HANDWRITING_BYTES..p + HANDWRITING_BYTES + 4];
        let mut out = vec![0u8; HANDWRITING_W * HANDWRITING_H * 4];
        for i in 0..HANDWRITING_W * HANDWRITING_H {
            let v = (img[i / 4] >> ((i % 4) * 2)) & 3;
            if v == 0 {
                continue;
            }
            let c = ink_color(pal[v as usize]);
            out[i * 4..i * 4 + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
        }
        out
    }
}

/// Bulletin board ink colors (palette ids used by the game's pen tool).
fn ink_color(id: u8) -> [u8; 3] {
    const INKS: [[u8; 3]; 8] = [
        [60, 50, 40],
        [220, 60, 60],
        [60, 110, 220],
        [60, 170, 90],
        [240, 170, 40],
        [170, 90, 200],
        [120, 80, 50],
        [250, 250, 250],
    ];
    INKS[(id as usize) % INKS.len()]
}
