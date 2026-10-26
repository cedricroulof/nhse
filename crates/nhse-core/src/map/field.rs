//! Map accessors on the save (field item layers, flags, terrain, acres, plaza).
use super::grid::ItemGrid;
use super::terrain::{TerrainTile, TILES_H, TILES_W, TILE_SIZE};
use super::ITEM_TILES_PER_ACRE;
use crate::save::HorizonSave;
use crate::structs::Item;
use crate::util::bits::{get_flag, set_flag};
use crate::util::bytes::*;
use serde::{Deserialize, Serialize};

pub const ACRES_W: usize = 9;
pub const ACRES_H: usize = 8;
const ACRE_BYTES: usize = ACRES_W * ACRES_H * 2;
pub const MAP_DESIGN_NONE: u16 = 0xF800;

/// Geometry of the field item layers for this save.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FieldLayout {
    /// Field item layer size in acres (7×6 before 3.0, 9×6 afterwards).
    pub acres_w: usize,
    pub acres_h: usize,
    /// Size in tiles.
    pub width: usize,
    pub height: usize,
    /// Offset of the layer inside the whole 9×8-acre map, in item tiles.
    pub offset_x: usize,
    pub offset_y: usize,
}

/// Plaza position (top-left), in item tile units of the whole map.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Plaza {
    pub x: u32,
    pub z: u32,
}

impl HorizonSave {
    pub fn field_layout(&self) -> FieldLayout {
        let aw = self.main_ofs.field_item_acre_width;
        let ah = 6;
        FieldLayout {
            acres_w: aw,
            acres_h: ah,
            width: aw * ITEM_TILES_PER_ACRE,
            height: ah * ITEM_TILES_PER_ACRE,
            offset_x: ((9 - aw) / 2) * ITEM_TILES_PER_ACRE,
            offset_y: ITEM_TILES_PER_ACRE,
        }
    }

    fn field_tile_count(&self) -> usize {
        let l = self.field_layout();
        l.width * l.height
    }

    fn field_layer_offset(&self, layer: usize) -> usize {
        self.main_ofs.field_item + layer * self.field_tile_count() * Item::SIZE
    }

    fn field_flag_offset(&self, layer: usize) -> usize {
        let n = self.field_tile_count();
        self.main_ofs.field_item + 2 * n * Item::SIZE + layer * (n / 8)
    }

    pub fn field_layer(&self, layer: usize) -> ItemGrid {
        let l = self.field_layout();
        let tiles = Item::read_array(&self.main.data, self.field_layer_offset(layer.min(1)), l.width * l.height);
        ItemGrid::new(l.width, l.height, tiles)
    }

    pub fn set_field_layer(&mut self, layer: usize, g: &ItemGrid) {
        let o = self.field_layer_offset(layer.min(1));
        Item::write_array(&mut self.main.data, o, &g.tiles);
    }

    /// Raw layer bytes (for fast transfer to the UI).
    pub fn field_layer_bytes(&self, layer: usize) -> &[u8] {
        let o = self.field_layer_offset(layer.min(1));
        &self.main.data[o..o + self.field_tile_count() * Item::SIZE]
    }

    pub fn set_field_layer_bytes(&mut self, layer: usize, bytes: &[u8]) -> crate::Result<()> {
        let n = self.field_tile_count() * Item::SIZE;
        if bytes.len() != n {
            return Err(crate::Error::Invalid(format!("field layer must be {n} bytes, got {}", bytes.len())));
        }
        let o = self.field_layer_offset(layer.min(1));
        self.main.data[o..o + n].copy_from_slice(bytes);
        Ok(())
    }

    /// "Active" (animated) flag of a field item tile. Flags are row-major over the tile grid.
    pub fn field_flag(&self, layer: usize, x: usize, y: usize) -> bool {
        let l = self.field_layout();
        get_flag(&self.main.data, self.field_flag_offset(layer), y * l.width + x)
    }

    pub fn set_field_flag(&mut self, layer: usize, x: usize, y: usize, v: bool) {
        let l = self.field_layout();
        let o = self.field_flag_offset(layer);
        set_flag(&mut self.main.data, o, y * l.width + x, v);
    }

    pub fn clear_field_flags(&mut self, layer: usize) {
        let o = self.field_flag_offset(layer);
        let n = self.field_tile_count() / 8;
        self.main.data[o..o + n].fill(0);
    }

    // --- terrain ------------------------------------------------------------------

    pub fn terrain(&self) -> Vec<TerrainTile> {
        let o = self.main_ofs.land_making_map;
        (0..TILES_W * TILES_H).map(|i| TerrainTile::read(&self.main.data, o + i * TILE_SIZE)).collect()
    }

    pub fn set_terrain(&mut self, tiles: &[TerrainTile]) {
        let o = self.main_ofs.land_making_map;
        for (i, t) in tiles.iter().enumerate().take(TILES_W * TILES_H) {
            t.write(&mut self.main.data, o + i * TILE_SIZE);
        }
    }

    pub fn terrain_bytes(&self) -> &[u8] {
        let o = self.main_ofs.land_making_map;
        &self.main.data[o..o + TILES_W * TILES_H * TILE_SIZE]
    }

    pub fn set_terrain_bytes(&mut self, bytes: &[u8]) -> crate::Result<()> {
        let n = TILES_W * TILES_H * TILE_SIZE;
        if bytes.len() != n {
            return Err(crate::Error::Invalid(format!("terrain must be {n} bytes, got {}", bytes.len())));
        }
        let o = self.main_ofs.land_making_map;
        self.main.data[o..o + n].copy_from_slice(bytes);
        Ok(())
    }

    // --- acres ---------------------------------------------------------------------

    pub fn acres(&self) -> Vec<u16> {
        let o = self.main_ofs.outside_field;
        read_u16_array(&self.main.data[o..o + ACRE_BYTES])
    }

    pub fn set_acres(&mut self, acres: &[u16]) {
        let o = self.main_ofs.outside_field;
        let n = acres.len().min(ACRES_W * ACRES_H);
        write_u16_array(&mut self.main.data[o..o + n * 2], &acres[..n]);
    }

    pub fn plaza(&self) -> Plaza {
        let o = self.main_ofs.outside_field + ACRE_BYTES;
        Plaza { x: u32le(&self.main.data, o + 4), z: u32le(&self.main.data, o + 8) }
    }

    pub fn set_plaza(&mut self, p: Plaza) {
        let o = self.main_ofs.outside_field + ACRE_BYTES;
        put_u32(&mut self.main.data, o + 4, p.x);
        put_u32(&mut self.main.data, o + 8, p.z);
    }

    pub fn outside_field_template(&self) -> (u16, u16) {
        let o = self.main_ofs.outside_field + ACRE_BYTES;
        (u16le(&self.main.data, o), u16le(&self.main.data, o + 2))
    }

    // --- custom design paths ("my design" tiles on the ground) ------------------------

    /// One u16 per terrain tile (row-major 112×96): design index, or 0xF800 for none.
    pub fn map_design_tiles(&self) -> Vec<u16> {
        let o = self.main_ofs.my_design_map;
        read_u16_array(&self.main.data[o..o + TILES_W * TILES_H * 2])
    }

    pub fn set_map_design_tiles(&mut self, v: &[u16]) {
        let o = self.main_ofs.my_design_map;
        let n = v.len().min(TILES_W * TILES_H);
        write_u16_array(&mut self.main.data[o..o + n * 2], &v[..n]);
    }
}
