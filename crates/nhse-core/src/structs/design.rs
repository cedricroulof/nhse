//! Custom designs ("My Designs") — normal 32×32 and PRO (4 sheets).
use crate::util::bytes::*;
use crate::util::strings::{get_string, set_string_if_changed};
use serde::{Deserialize, Serialize};

pub const NORMAL_SIZE: usize = 0x2A8;
pub const PRO_SIZE: usize = 0x8A8;
pub const DIM: usize = 32;
pub const PALETTE_COLORS: usize = 15;
const PALETTE_START: usize = 0x78;
const PIXEL_START: usize = PALETTE_START + PALETTE_COLORS * 3; // 0xA5
const SHEET_BYTES: usize = DIM * DIM / 2;
const PERSONAL: usize = 0x38;

/// Summary of a design for the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignInfo {
    pub index: usize,
    pub name: String,
    pub town_name: String,
    pub player_name: String,
    pub town_id: u32,
    pub player_id: u32,
    pub usage: u32,
    pub pro: bool,
    pub empty: bool,
}

pub fn sheets(pro: bool) -> usize {
    if pro {
        4
    } else {
        1
    }
}

pub fn size(pro: bool) -> usize {
    if pro {
        PRO_SIZE
    } else {
        NORMAL_SIZE
    }
}

pub fn info(d: &[u8], index: usize, pro: bool) -> DesignInfo {
    let name = get_string(d, 0x10, 20);
    DesignInfo {
        index,
        empty: name.is_empty() && d[PIXEL_START..PIXEL_START + 64].iter().all(|b| *b == 0),
        name,
        town_id: u32le(d, PERSONAL),
        town_name: get_string(d, PERSONAL + 4, 10),
        player_id: u32le(d, PERSONAL + 0x1C),
        player_name: get_string(d, PERSONAL + 0x20, 10),
        usage: u32le(d, 0x70),
        pro,
    }
}

pub fn set_name(d: &mut [u8], name: &str) {
    set_string_if_changed(d, 0x10, 20, name);
}

pub fn set_usage(d: &mut [u8], usage: u32) {
    put_u32(d, 0x70, usage);
}

/// Sets the author (town + player identity) of a design.
pub fn set_author(d: &mut [u8], town_id: u32, town_name: &str, player_id: u32, player_name: &str) {
    put_u32(d, PERSONAL, town_id);
    set_string_if_changed(d, PERSONAL + 4, 10, town_name);
    put_u32(d, PERSONAL + 0x1C, player_id);
    set_string_if_changed(d, PERSONAL + 0x20, 10, player_name);
}

pub fn palette(d: &[u8]) -> [[u8; 3]; PALETTE_COLORS] {
    std::array::from_fn(|i| {
        let o = PALETTE_START + i * 3;
        [d[o], d[o + 1], d[o + 2]]
    })
}

pub fn set_palette(d: &mut [u8], pal: &[[u8; 3]]) {
    for (i, c) in pal.iter().enumerate().take(PALETTE_COLORS) {
        let o = PALETTE_START + i * 3;
        d[o..o + 3].copy_from_slice(c);
    }
}

/// Palette index of a pixel (15 = transparent).
pub fn pixel(d: &[u8], sheet: usize, x: usize, y: usize) -> u8 {
    let index = y * DIM + x;
    let v = d[PIXEL_START + sheet * SHEET_BYTES + index / 2];
    if index & 1 == 0 {
        v & 0xF
    } else {
        v >> 4
    }
}

pub fn set_pixel(d: &mut [u8], sheet: usize, x: usize, y: usize, value: u8) {
    let index = y * DIM + x;
    let o = PIXEL_START + sheet * SHEET_BYTES + index / 2;
    d[o] = if index & 1 == 0 { (d[o] & 0xF0) | (value & 0xF) } else { ((value & 0xF) << 4) | (d[o] & 0xF) };
}

/// Renders one sheet to RGBA (transparent pixels have alpha 0).
pub fn render_rgba(d: &[u8], sheet: usize) -> Vec<u8> {
    let pal = palette(d);
    let mut out = vec![0u8; DIM * DIM * 4];
    for y in 0..DIM {
        for x in 0..DIM {
            let p = pixel(d, sheet, x, y) as usize;
            if p >= PALETTE_COLORS {
                continue;
            }
            let o = (y * DIM + x) * 4;
            out[o..o + 3].copy_from_slice(&pal[p]);
            out[o + 3] = 255;
        }
    }
    out
}

/// Replaces the pixels/palette of `sheet` from a 32×32 RGBA image, quantizing to 15 colors.
/// For PRO designs the palette is shared by all sheets, so pass every sheet at once.
pub fn import_rgba(d: &mut [u8], sheets_rgba: &[Vec<u8>]) {
    let mut pixels: Vec<[u8; 3]> = Vec::new();
    for img in sheets_rgba {
        for px in img.chunks_exact(4) {
            if px[3] >= 128 {
                pixels.push([px[0], px[1], px[2]]);
            }
        }
    }
    let pal = quantize(&pixels, PALETTE_COLORS);
    let mut full = [[0u8; 3]; PALETTE_COLORS];
    for (i, c) in pal.iter().enumerate() {
        full[i] = *c;
    }
    set_palette(d, &full);
    for (s, img) in sheets_rgba.iter().enumerate() {
        for y in 0..DIM {
            for x in 0..DIM {
                let o = (y * DIM + x) * 4;
                let v = if img[o + 3] < 128 { 15 } else { nearest(&pal, [img[o], img[o + 1], img[o + 2]]) as u8 };
                set_pixel(d, s, x, y, v);
            }
        }
    }
}

fn dist(a: [u8; 3], b: [u8; 3]) -> u32 {
    let dr = a[0] as i32 - b[0] as i32;
    let dg = a[1] as i32 - b[1] as i32;
    let db = a[2] as i32 - b[2] as i32;
    // weighted for perceived brightness
    (2 * dr * dr + 4 * dg * dg + 3 * db * db) as u32
}

fn nearest(pal: &[[u8; 3]], c: [u8; 3]) -> usize {
    pal.iter().enumerate().min_by_key(|(_, p)| dist(**p, c)).map(|(i, _)| i).unwrap_or(0)
}

/// Median-cut quantization followed by a few k-means refinement passes.
pub fn quantize(pixels: &[[u8; 3]], k: usize) -> Vec<[u8; 3]> {
    if pixels.is_empty() {
        return vec![[255, 255, 255]];
    }
    let mut unique: Vec<[u8; 3]> = pixels.to_vec();
    unique.sort();
    unique.dedup();
    if unique.len() <= k {
        return unique;
    }
    let mut boxes: Vec<Vec<[u8; 3]>> = vec![pixels.to_vec()];
    while boxes.len() < k {
        // split the box with the widest channel range
        let (bi, ch, _) = boxes
            .iter()
            .enumerate()
            .filter(|(_, b)| b.len() > 1)
            .map(|(i, b)| {
                let (ch, r) = (0..3)
                    .map(|c| {
                        let mn = b.iter().map(|p| p[c]).min().unwrap();
                        let mx = b.iter().map(|p| p[c]).max().unwrap();
                        (c, mx - mn)
                    })
                    .max_by_key(|(_, r)| *r)
                    .unwrap();
                (i, ch, r as usize * b.len())
            })
            .max_by_key(|(_, _, score)| *score)
            .unwrap_or((0, 0, 0));
        if boxes[bi].len() <= 1 {
            break;
        }
        let mut b = boxes.swap_remove(bi);
        b.sort_by_key(|p| p[ch]);
        let tail = b.split_off(b.len() / 2);
        boxes.push(b);
        boxes.push(tail);
    }
    let mut centers: Vec<[u8; 3]> = boxes
        .iter()
        .filter(|b| !b.is_empty())
        .map(|b| {
            let n = b.len() as u64;
            let s = b.iter().fold([0u64; 3], |a, p| [a[0] + p[0] as u64, a[1] + p[1] as u64, a[2] + p[2] as u64]);
            [(s[0] / n) as u8, (s[1] / n) as u8, (s[2] / n) as u8]
        })
        .collect();
    for _ in 0..6 {
        let mut sums = vec![[0u64; 4]; centers.len()];
        for p in pixels {
            let i = nearest(&centers, *p);
            sums[i][0] += p[0] as u64;
            sums[i][1] += p[1] as u64;
            sums[i][2] += p[2] as u64;
            sums[i][3] += 1;
        }
        for (c, s) in centers.iter_mut().zip(&sums) {
            if s[3] > 0 {
                *c = [(s[0] / s[3]) as u8, (s[1] / s[3]) as u8, (s[2] / s[3]) as u8];
            }
        }
    }
    centers
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pixels_roundtrip() {
        let mut d = vec![0u8; PRO_SIZE];
        set_pixel(&mut d, 2, 5, 7, 9);
        set_pixel(&mut d, 2, 6, 7, 3);
        assert_eq!(pixel(&d, 2, 5, 7), 9);
        assert_eq!(pixel(&d, 2, 6, 7), 3);
        assert_eq!(pixel(&d, 1, 5, 7), 0);
    }
    #[test]
    fn import_export() {
        let mut d = vec![0u8; NORMAL_SIZE];
        let mut img = vec![0u8; DIM * DIM * 4];
        for (i, px) in img.chunks_exact_mut(4).enumerate() {
            let c = (i % 4) as u8 * 60;
            px.copy_from_slice(&[c, 255 - c, 100, if i % 9 == 0 { 0 } else { 255 }]);
        }
        import_rgba(&mut d, &[img.clone()]);
        let out = render_rgba(&d, 0);
        for (a, b) in img.chunks_exact(4).zip(out.chunks_exact(4)) {
            if a[3] == 0 {
                assert_eq!(b[3], 0);
            } else {
                assert_eq!(&a[..3], &b[..3]);
            }
        }
    }
}
