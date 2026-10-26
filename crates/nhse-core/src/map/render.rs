//! Rasterization of the island map and item color tables.
use super::terrain::{tile_pixel, Rgb, TerrainTile, TILES_H, TILES_W};
use crate::data::items::{item_count, item_kind, KIND_UNKNOWN};
use crate::data::tables::{enum_value, tables};

static OUTSIDE: &[u8] = include_bytes!("../../resources/byte/outside.bin");
const ACRE_TEMPLATE_MAX: u16 = 317; // FldOutNGardenRFront00

/// Collision type → color for pre-defined (exterior) acres.
fn collision_color(c: u8) -> Option<Rgb> {
    Some(match c {
        0 => [76, 154, 74],             // grass
        1 | 12 | 15 => [104, 196, 214], // river / river mouth
        3 => [192, 192, 192],           // stone
        4 => [238, 222, 160],           // sand
        5 => [86, 176, 210],            // sea
        6 => [190, 140, 100],           // wood
        8 => [60, 60, 70],              // building
        10 => [70, 70, 80],             // door
        22 | 49 => [176, 120, 86],      // thin wood, pier
        29 => [230, 214, 150],          // beach edge
        41 | 45 => [118, 122, 132],     // rocks
        42 => [128, 133, 147],          // tall rocks
        43 => [70, 210, 220],           // tide pool
        44 => [96, 160, 84],            // grass/beach edge
        46 => [96, 186, 214],           // sea edge
        47 => [150, 140, 70],           // sandstone
        51 => [52, 152, 52],
        70 => [109, 113, 124],
        149 => [179, 207, 252], // ice
        150 => [61, 119, 212],
        _ => return None,
    })
}

/// Color of a pixel (x, y in 0..16) of a pre-defined acre, or `None` if transparent (terrain-driven).
pub fn acre_pixel(acre: u16, x: usize, y: usize) -> Option<Rgb> {
    if acre == 0 || acre > ACRE_TEMPLATE_MAX {
        return None;
    }
    let ofs = acre as usize * 4096 + 4 * (y * 64 + x);
    let c = *OUTSIDE.get(ofs)?;
    // collision type 7 ("null") marks a customizable pixel drawn from the terrain layer
    if c == 7 {
        return None;
    }
    Some(collision_color(c).unwrap_or([255, 0, 255]))
}

/// Inputs needed to draw the terrain.
pub struct TerrainView<'a> {
    pub tiles: &'a [TerrainTile],
    /// Acre template ids, 9×8 (row major), including the sea border.
    pub acres: &'a [u16],
}

impl TerrainView<'_> {
    fn tile(&self, x: usize, y: usize) -> &TerrainTile {
        &self.tiles[x * TILES_H + y]
    }

    /// Acre template id for a terrain-relative coordinate (may be negative = border).
    pub fn acre_at(&self, rel_x: i64, rel_y: i64) -> u16 {
        let ax = (16 + rel_x).div_euclid(16);
        let ay = (16 + rel_y).div_euclid(16);
        if !(0..9).contains(&ax) || !(0..8).contains(&ay) {
            return 0;
        }
        self.acres.get((ay * 9 + ax) as usize).copied().unwrap_or(0)
    }
}

/// Renders the whole map (9×8 acres incl. the sea border) at `px` pixels per terrain tile.
/// Returns RGBA, `(144*px) × (128*px)`.
pub fn render_terrain(view: &TerrainView, px: usize) -> (usize, usize, Vec<u8>) {
    let px = px.clamp(1, 16);
    let tiles_w = 9 * 16;
    let tiles_h = 8 * 16;
    let (w, h) = (tiles_w * px, tiles_h * px);
    let mut out = vec![0u8; w * h * 4];
    for ty in 0..tiles_h {
        for tx in 0..tiles_w {
            let rel_x = tx as i64 - 16;
            let rel_y = ty as i64 - 16;
            let acre = view.acre_at(rel_x, rel_y);
            let inside = rel_x >= 0
                && rel_y >= 0
                && (rel_x as usize) < TILES_W
                && (rel_y as usize) < TILES_H;
            for sy in 0..px {
                for sx in 0..px {
                    // sample the 16×16 tile pattern
                    let ix = (sx * 16 + 8) / px;
                    let iy = (sy * 16 + 8) / px;
                    let ix = ix.min(15);
                    let iy = iy.min(15);
                    let c = match acre_pixel(acre, (tx % 16).min(15), (ty % 16).min(15)) {
                        Some(c) if !inside || acre != 0 => c,
                        _ if inside => tile_pixel(
                            view.tile(rel_x as usize, rel_y as usize),
                            ix as i32,
                            iy as i32,
                        ),
                        Some(c) => c,
                        None => [86, 176, 210],
                    };
                    let o = ((ty * px + sy) * w + tx * px + sx) * 4;
                    out[o] = c[0];
                    out[o + 1] = c[1];
                    out[o + 2] = c[2];
                    out[o + 3] = 255;
                }
            }
        }
    }
    (w, h, out)
}

/// Encodes RGBA pixels as PNG.
pub fn encode_png(w: usize, h: usize, rgba: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut buf, w as u32, h as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(png::Compression::Fast);
        let mut wr = enc.write_header().expect("png header");
        wr.write_image_data(rgba).expect("png data");
    }
    buf
}

/// Decodes a PNG (any color type) into 8-bit RGBA.
pub fn decode_png(bytes: &[u8]) -> Option<(usize, usize, Vec<u8>)> {
    let mut dec = png::Decoder::new(bytes);
    dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = dec.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let (w, h) = (info.width as usize, info.height as usize);
    let src = &buf[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgba => src.to_vec(),
        png::ColorType::Rgb => src
            .chunks_exact(3)
            .flat_map(|c| [c[0], c[1], c[2], 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => src
            .chunks_exact(2)
            .flat_map(|c| [c[0], c[0], c[0], c[1]])
            .collect(),
        png::ColorType::Grayscale => src.iter().flat_map(|c| [*c, *c, *c, 255]).collect(),
        png::ColorType::Indexed => return None,
    };
    Some((w, h, rgba))
}

// --- item colors ------------------------------------------------------------------

const KIND_PALETTE: [u32; 174] = [
    0xD9D9D9, 0xCCD9E8, 0x7F7F7F, 0xD5D5D5, 0xF7F7F7, 0xCFCFCF, 0xB4B4B4, 0xF1F1F1, 0xFFFFFF,
    0x7F7F7F, 0x7F7F7F, 0xB6B6B6, 0x7FBBEB, 0xFFFFFF, 0x7FB2E5, 0xF9FBFD, 0xDFE6ED, 0x7F7F7F,
    0xFFFFF0, 0x7F7F7F, 0xF7F7F7, 0x7F7F7F, 0xE3E3E3, 0xFFFFFF, 0xB1B1B1, 0x7F7F7F, 0xFFFFFF,
    0xF7FBFF, 0xFCF5EB, 0x7FFFFF, 0xBFFFE9, 0xF7FFFF, 0xFAFAED, 0xFFF1E1, 0x7F7F7F, 0xFFF5E6,
    0x7F7FFF, 0xC495F0, 0xD29494, 0xEEDBC3, 0xAFCECF, 0xBFFF7F, 0xE8B48E, 0xFFBFA7, 0xB1CAF6,
    0xFFFBED, 0xED899D, 0x7FFFFF, 0x7F7FC5, 0x7FC5C5, 0xDBC285, 0xD4D4D4, 0x7FB17F, 0xDEDBB5,
    0xC57FC5, 0xAAB597, 0xFFC57F, 0xCC98E5, 0xC57F7F, 0xF4CABC, 0xC7DDC5, 0xA39EC5, 0x97A7A7,
    0x7FE6E8, 0xC97FE9, 0xFF89C9, 0x7FDFFF, 0xB4B4B4, 0x8EC7FF, 0xD89090, 0xFFFCF7, 0x90C590,
    0xFF7FFF, 0xEDEDED, 0xFBFBFF, 0xFFEB7F, 0xECD28F, 0xBFBFBF, 0x7FBF7F, 0xD6FF97, 0xF7FFF7,
    0xFFB4D9, 0xE6ADAD, 0xA57FC0, 0xFFFFF7, 0xF7F2C5, 0xF2F2FC, 0xFFF7FA, 0xBDFD7F, 0xFFFCE6,
    0xD6EBF2, 0xF7BFBF, 0xEFFFFF, 0xFCFCE8, 0xE9E9E9, 0xC7F6C7, 0xFFDAE0, 0xFFCFBC, 0x8FD8D4,
    0xC3E6FC, 0xBBC3CC, 0xD7E1EE, 0xFFFFEF, 0x7FFF7F, 0x98E698, 0xFCF7F2, 0xFF7FFF, 0xBF7F7F,
    0xB2E6D4, 0x7F7FE6, 0xDCAAE9, 0xC9B7ED, 0x9DD9B8, 0xBDB3F6, 0x7FFCCC, 0xA3E8E5, 0xE38AC2,
    0x8C8CB7, 0xFAFFFC, 0xFFF1F0, 0xFFF1DA, 0xFFEED6, 0x7F7FBF, 0xFEFAF2, 0xBFBF7F, 0xB5C691,
    0xFFD27F, 0xFFA27F, 0xECB7EA, 0xF6F3D4, 0xCBFDCB, 0xD7F6F6, 0xEDB7C9, 0xFFF7EA, 0xFFECDC,
    0xE6C29F, 0xFFDFE5, 0xEECFEE, 0xD7EFF2, 0xBF7FBF, 0xFF7F7F, 0xDDC7C7, 0xA0B4F0, 0xC5A289,
    0xFCBFB8, 0xF9D1AF, 0x96C5AB, 0xFFFAF6, 0xCFA896, 0xDFDFDF, 0xC3E6F5, 0xB4ACE6, 0xB7BFC7,
    0xFFFCFC, 0x7FFFBF, 0xA2C0D9, 0xE8D9C5, 0x7FBFBF, 0xEBDFEB, 0xFFB1A3, 0x9FEFE7, 0xF6C0F6,
    0xFAEED9, 0xFFFFFF, 0xFAFAFA, 0xFFFF7F, 0xCCE698, 0xF7F7F7, 0xFFFFFF, 0xCFCFCF, 0xDCE8F4,
    0xEBF1F8, 0xF7F7F7, 0x99CCFF,
];
const KIND_PALETTE_LEN: usize = 174;

fn blend(a: u32, b: u32, amt: f32) -> u32 {
    let ch = |c: u32, s: u32| ((c >> s) & 0xFF) as f32;
    let mix = |s: u32| ((ch(a, s) * amt + ch(b, s) * (1.0 - amt)) as u32) << s;
    mix(16) | mix(8) | mix(0)
}

/// Color for an `ItemKind` ordinal (matches the original editor's palette).
pub fn kind_color(kind: u8) -> u32 {
    if kind == KIND_UNKNOWN {
        return 0x32CD32;
    }
    let i = kind as usize;
    if i < KIND_PALETTE_LEN {
        return KIND_PALETTE[i];
    }
    blend(0xFF0000, KIND_PALETTE[i % KIND_PALETTE_LEN], 0.2)
}

fn field_item_color(id: u16) -> u32 {
    let t = tables();
    let Some(def) = t.field_items.get(&id) else {
        return 0x006400;
    };
    let fk = |n: &str| enum_value("FieldItemKind", n).unwrap_or(-1) as u8;
    let k = def.kind;
    let in_range = |a: &str, b: &str| (fk(a)..=fk(b)).contains(&k);
    if in_range("PltTreeBamboo", "PltTreePalm") {
        if (0xEC9C..=0xECA0).contains(&id) {
            return 0xFFD700;
        }
        return match id {
            0xEA61 => 0xFF0000,
            0xEA62 => 0xFFA500,
            0xEAC8 => 0x00FF00,
            0xEAC9 => 0x8B0000,
            0xEACA => 0xFFDAB9,
            0xEA69 | 0xEAB6 | 0xEAB7 | 0xEAB8 => 0x8B4513,
            0xEA77 | 0xEAC0 | 0xEAC1 | 0xEAC2 => 0xFAFAD2,
            0xEA76 | 0xEAC4 | 0xEAC5 | 0xEAC6 | 0xEAC7 => 0x3CB371,
            _ => 0xF4A460,
        };
    }
    if in_range("PltFlwAnemone", "PltFlwYuri") {
        return 0xFF69B4;
    }
    if in_range("PltWeedAut0", "PltWeedWin1") {
        return 0x556B2F;
    }
    if in_range("FenceBamboo", "FenceWoodWhite") {
        return 0xF08080;
    }
    if k == fk("UnitIconHole") {
        return 0x000000;
    }
    if in_range("PltBushAzalea", "PltBushOsmanthus") {
        return 0x90EE90;
    }
    if in_range("StoneA", "StoneE") {
        return 0xD3D3D3;
    }
    0x006400
}

/// Color of an item id on the map (0xRRGGBB). `None` for empty.
pub fn item_color(id: u16) -> Option<u32> {
    if id == 0xFFFE {
        return None;
    }
    if id >= 60_000 {
        return Some(field_item_color(id));
    }
    Some(kind_color(item_kind(id)))
}

/// Color table for all regular item ids plus field items, for the UI.
pub fn item_color_table() -> (Vec<u32>, Vec<(u16, u32)>) {
    let regular = (0..item_count())
        .map(|i| kind_color(item_kind(i as u16)))
        .collect();
    let field = tables()
        .field_items_sorted
        .iter()
        .map(|f| (f.id, field_item_color(f.id)))
        .collect();
    (regular, field)
}
