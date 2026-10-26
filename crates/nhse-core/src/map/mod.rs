//! Island map: field item layers, terrain, acres, buildings and rendering.
pub mod grid;
pub mod terrain;
pub mod render;
pub mod field;

pub use grid::ItemGrid;
pub use terrain::TerrainTile;

/// Field item tiles per acre side.
pub const ITEM_TILES_PER_ACRE: usize = 32;
/// Terrain tiles per acre side.
pub const TERRAIN_TILES_PER_ACRE: usize = 16;
/// Whole map (including the sea border) in acres.
pub const MAP_ACRES_W: usize = 9;
pub const MAP_ACRES_H: usize = 8;
/// Terrain layer size in acres (land + beach).
pub const TERRAIN_ACRES_W: usize = 7;
pub const TERRAIN_ACRES_H: usize = 6;
/// Whole map in field item units.
pub const MAP_W: usize = MAP_ACRES_W * ITEM_TILES_PER_ACRE; // 288
pub const MAP_H: usize = MAP_ACRES_H * ITEM_TILES_PER_ACRE; // 256
