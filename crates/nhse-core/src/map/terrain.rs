//! Terrain tiles (16 per acre) and their appearance.
use crate::util::bytes::*;
use serde::{Deserialize, Serialize};

pub const TILE_SIZE: usize = 0xE;
pub const TILES_W: usize = 7 * 16; // 112
pub const TILES_H: usize = 6 * 16; // 96

/// `[model, variation, angle]` for the ground and the road layer, plus elevation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainTile {
    pub model: u16,
    pub variation: u16,
    pub angle: u16,
    pub road: u16,
    pub road_variation: u16,
    pub road_angle: u16,
    pub elevation: u16,
}

impl TerrainTile {
    pub fn read(d: &[u8], o: usize) -> Self {
        Self {
            model: u16le(d, o),
            variation: u16le(d, o + 2),
            angle: u16le(d, o + 4),
            road: u16le(d, o + 6),
            road_variation: u16le(d, o + 8),
            road_angle: u16le(d, o + 10),
            elevation: u16le(d, o + 12),
        }
    }
    pub fn write(&self, d: &mut [u8], o: usize) {
        put_u16(d, o, self.model);
        put_u16(d, o + 2, self.variation);
        put_u16(d, o + 4, self.angle);
        put_u16(d, o + 6, self.road);
        put_u16(d, o + 8, self.road_variation);
        put_u16(d, o + 10, self.road_angle);
        put_u16(d, o + 12, self.elevation);
    }
}

// --- model classification -----------------------------------------------------

pub fn is_river(m: u16) -> bool {
    (0x01..=0x15).contains(&m)
}
pub fn is_cliff(m: u16) -> bool {
    (0x16..=0x29).contains(&m) || m == 0x93
}
pub fn is_fall(m: u16) -> bool {
    (0x3A..=0x46).contains(&m) || (0x71..=0x92).contains(&m)
}
pub fn is_road(m: u16) -> bool {
    m >= 0x94 || (0x47..=0x70).contains(&m)
}

/// Road family used for coloring and for the road brush.
pub fn road_family(m: u16) -> Option<&'static str> {
    Some(match m {
        0x47..=0x5B => "soil",
        0x5C..=0x70 => "stone",
        0x94..=0xA8 => "brick",
        0xA9..=0xBD => "darksoil",
        0xBE..=0xD2 => "pattern",
        0xD3..=0xE7 => "sand",
        0xE8..=0xFC => "tile",
        0xFD..=0x111 => "wood",
        _ => return None,
    })
}

pub type Rgb = [u8; 3];

const GRASS: [Rgb; 4] = [
    [76, 154, 74],
    [92, 170, 84],
    [108, 184, 96],
    [124, 197, 110],
];
const WATER: Rgb = [104, 196, 214];
const WATER_DEEP: Rgb = [84, 176, 200];
const SHORE: Rgb = [214, 199, 140];
const CLIFF: [Rgb; 4] = [[66, 104, 62], [80, 118, 70], [94, 132, 80], [108, 146, 92]];
const CLIFF_EDGE: Rgb = [52, 72, 48];
const FALL: Rgb = [134, 214, 236];
const FOAM: Rgb = [235, 250, 255];

fn road_color(m: u16) -> Rgb {
    match road_family(m) {
        Some("brick") => [178, 70, 60],
        Some("darksoil") => [120, 82, 52],
        Some("soil") => [196, 150, 98],
        Some("stone") => [158, 158, 160],
        Some("pattern") => [236, 230, 210],
        Some("tile") => [94, 132, 178],
        Some("sand") => [226, 196, 140],
        Some("wood") => [170, 120, 74],
        _ => [200, 170, 120],
    }
}

fn tri_area2(a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> i32 {
    (a.0 * (b.1 - c.1) + b.0 * (c.1 - a.1) + c.0 * (a.1 - b.1)).abs()
}

fn in_tri(px: i32, py: i32, a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> bool {
    let p = (px, py);
    tri_area2(a, b, c) == tri_area2(p, b, c) + tri_area2(a, p, c) + tri_area2(a, b, p)
}

fn in_multi(
    px: i32,
    py: i32,
    a: (i32, i32),
    b: (i32, i32),
    c: (i32, i32),
    va: (i32, i32),
    vb: (i32, i32),
) -> bool {
    in_tri(px, py, a, va, b) || in_tri(px, py, a, b, c) || in_tri(px, py, c, b, vb)
}

fn nub_tl(x: i32, y: i32) -> bool {
    in_tri(x, y, (0, 4), (0, 0), (4, 0))
}
fn nub_tr(x: i32, y: i32) -> bool {
    in_tri(x, y, (12, 0), (15, 0), (15, 4))
}
fn nub_bl(x: i32, y: i32) -> bool {
    in_tri(x, y, (0, 12), (0, 15), (4, 15))
}
fn nub_br(x: i32, y: i32) -> bool {
    in_tri(x, y, (12, 15), (15, 15), (15, 12))
}

/// True if pixel (x, y) in 0..16 of a river tile is land (bank).
/// Shapes follow the original editor's river tile atlas.
pub fn river_is_land(m: u16, angle: u16, x: i32, y: i32) -> bool {
    let a = angle & 3;
    let edge = |lo: bool, hi: bool| lo || hi;
    match m {
        0x01 => edge(!(4..12).contains(&x), !(4..12).contains(&y)),
        0x04 => match a {
            0 => !(4..12).contains(&x) || y < 4,
            1 => x < 4 || !(4..12).contains(&y),
            2 => !(4..12).contains(&x) || y >= 12,
            _ => !(4..12).contains(&y) || x >= 12,
        },
        0x07 => match a {
            0 | 2 => !(4..12).contains(&x),
            _ => !(4..12).contains(&y),
        },
        0x08 => match a {
            0 => {
                in_multi(x, y, (4, 15), (0, 0), (15, 4), (0, 15), (15, 0))
                    || nub_br(x, y)
                    || x < 4
                    || y < 4
            }
            1 => {
                in_multi(x, y, (4, 0), (0, 15), (15, 12), (0, 0), (15, 15))
                    || nub_tr(x, y)
                    || x < 4
                    || y >= 12
            }
            2 => {
                in_multi(x, y, (0, 12), (15, 15), (12, 0), (0, 15), (15, 0))
                    || nub_tl(x, y)
                    || x >= 12
                    || y >= 12
            }
            _ => {
                in_multi(x, y, (0, 4), (15, 0), (12, 15), (0, 0), (15, 15))
                    || nub_bl(x, y)
                    || x >= 12
                    || y < 4
            }
        },
        0x09 => match a {
            0 => x < 4 || y < 4 || nub_br(x, y),
            1 => x < 4 || y >= 12 || nub_tr(x, y),
            2 => x >= 12 || y >= 12 || nub_tl(x, y),
            _ => x >= 12 || y < 4 || nub_bl(x, y),
        },
        0x0A => match a {
            0 => x < 4 || nub_tr(x, y) || nub_br(x, y),
            1 => y >= 12 || nub_tl(x, y) || nub_tr(x, y),
            2 => x >= 12 || nub_bl(x, y) || nub_tl(x, y),
            _ => y < 4 || nub_br(x, y) || nub_bl(x, y),
        },
        0x0B => match a {
            0 => in_multi(x, y, (4, 15), (0, 0), (15, 4), (0, 15), (15, 0)),
            1 => in_multi(x, y, (4, 0), (0, 15), (15, 12), (0, 0), (15, 15)),
            2 => in_multi(x, y, (0, 12), (15, 15), (12, 0), (0, 15), (15, 0)),
            _ => in_multi(x, y, (0, 4), (15, 0), (12, 15), (0, 0), (15, 15)),
        },
        0x0C => match a {
            0 => x < 4 || y < 4,
            1 => x < 4 || y >= 12,
            2 => x >= 12 || y >= 12,
            _ => x >= 12 || y < 4,
        },
        0x0D => match a {
            0 => x < 4 || nub_tr(x, y),
            1 => y >= 12 || nub_tl(x, y),
            2 => x >= 12 || nub_bl(x, y),
            _ => y < 4 || nub_br(x, y),
        },
        0x0E => match a {
            0 => x < 4 || nub_br(x, y),
            1 => y >= 12 || nub_tr(x, y),
            2 => x >= 12 || nub_tl(x, y),
            _ => y < 4 || nub_bl(x, y),
        },
        0x0F => nub_bl(x, y) || nub_br(x, y) || nub_tl(x, y) || nub_tr(x, y),
        0x10 => match a {
            0 => nub_tl(x, y) || nub_tr(x, y) || nub_br(x, y),
            1 => nub_bl(x, y) || nub_tl(x, y) || nub_tr(x, y),
            2 => nub_bl(x, y) || nub_br(x, y) || nub_tl(x, y),
            _ => nub_bl(x, y) || nub_br(x, y) || nub_tr(x, y),
        },
        0x11 => match a {
            0 => x < 4,
            1 => y >= 12,
            2 => x >= 12,
            _ => y < 4,
        },
        0x12 => match a {
            0 | 2 => nub_tl(x, y) || nub_br(x, y),
            _ => nub_bl(x, y) || nub_tr(x, y),
        },
        0x13 => match a {
            0 => nub_bl(x, y) || nub_br(x, y),
            1 => nub_br(x, y) || nub_tr(x, y),
            2 => nub_tr(x, y) || nub_tl(x, y),
            _ => nub_tl(x, y) || nub_bl(x, y),
        },
        0x14 => match a {
            0 => nub_bl(x, y),
            1 => nub_br(x, y),
            2 => nub_tr(x, y),
            _ => nub_tl(x, y),
        },
        _ => false,
    }
}

fn grass(elev: u16) -> Rgb {
    GRASS[(elev as usize).min(3)]
}

fn shade(c: Rgb, f: f32) -> Rgb {
    [
        (c[0] as f32 * f).min(255.0) as u8,
        (c[1] as f32 * f).min(255.0) as u8,
        (c[2] as f32 * f).min(255.0) as u8,
    ]
}

/// Color of pixel (x, y) (0..16) inside a terrain tile.
pub fn tile_pixel(t: &TerrainTile, x: i32, y: i32) -> Rgb {
    if is_road(t.road) {
        let c = road_color(t.road);
        // subtle texture so road patches read as paths
        return if (x + y) % 8 == 0 { shade(c, 0.93) } else { c };
    }
    let m = t.model;
    if is_river(m) {
        let land = |px: i32, py: i32| -> bool {
            if !(0..16).contains(&px) || !(0..16).contains(&py) {
                return false;
            }
            river_is_land(m, t.angle, px, py)
        };
        if land(x, y) {
            // a narrow sandy band where the bank meets the water
            let near_water = [(-1, 0), (1, 0), (0, -1), (0, 1)].iter().any(|(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                (0..16).contains(&nx) && (0..16).contains(&ny) && !land(nx, ny)
            });
            return if near_water {
                SHORE
            } else {
                grass(t.elevation)
            };
        }
        return if (x * 3 + y * 5) % 11 == 0 {
            WATER_DEEP
        } else {
            WATER
        };
    }
    if is_fall(m) {
        return if (y + (x % 4)) % 5 == 0 { FOAM } else { FALL };
    }
    if is_cliff(m) {
        let base = CLIFF[(t.elevation as usize).min(3)];
        return if x == 0 || y == 0 || x == 15 || y == 15 {
            CLIFF_EDGE
        } else {
            base
        };
    }
    grass(t.elevation)
}

/// Short label for a terrain model (e.g. `River2B`).
pub fn model_name(m: u16) -> &'static str {
    crate::data::tables::enum_name("TerrainUnitModel", m as i64).unwrap_or("?")
}
