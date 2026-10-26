//! Map commands. Large layers travel as raw bytes (`tauri::ipc::Response` / raw request bodies).
use crate::error::{err, CmdError, CmdResult};
use crate::state::AppState;
use crate::{with_save, with_save_mut};
use nhse_core::editing::fieldtools::{self, Arrangement, ClearKind};
use nhse_core::editing::formats;
use nhse_core::map::field::Plaza;
use nhse_core::map::render::{encode_png, render_terrain, TerrainView};
use nhse_core::map::terrain::{TerrainTile, TILES_H, TILES_W};
use nhse_core::map::ItemGrid;
use nhse_core::Item;
use serde::{Deserialize, Serialize};
use tauri::ipc::{InvokeBody, Request, Response};
use tauri::State;

fn header_usize(req: &Request, name: &str) -> CmdResult<usize> {
    req.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| CmdError(format!("missing header {name}")))
}

fn raw_body<'a>(req: &'a Request) -> CmdResult<&'a [u8]> {
    match req.body() {
        InvokeBody::Raw(b) => Ok(b),
        _ => err("expected a binary request body"),
    }
}

/// Field item layer as raw bytes (column-major, 8 bytes per tile).
#[tauri::command]
pub fn field_layer(state: State<AppState>, layer: usize) -> CmdResult<Response> {
    with_save!(state, |s| Ok(Response::new(
        s.field_layer_bytes(layer).to_vec()
    )))
}

#[tauri::command]
pub fn set_field_layer(state: State<AppState>, request: Request) -> CmdResult<()> {
    let layer = header_usize(&request, "x-layer")?;
    let body = raw_body(&request)?;
    with_save_mut!(state, |s| Ok(s.set_field_layer_bytes(layer, body)?))
}

/// Terrain tiles as raw bytes (0xE per tile, column-major 112×96).
#[tauri::command]
pub fn terrain(state: State<AppState>) -> CmdResult<Response> {
    with_save!(state, |s| Ok(Response::new(s.terrain_bytes().to_vec())))
}

#[tauri::command]
pub fn set_terrain(state: State<AppState>, request: Request) -> CmdResult<()> {
    let body = raw_body(&request)?;
    with_save_mut!(state, |s| Ok(s.set_terrain_bytes(body)?))
}

/// Rendered terrain + acre templates (PNG), `px` pixels per terrain tile.
#[tauri::command]
pub fn terrain_png(state: State<AppState>, px: Option<usize>) -> CmdResult<Response> {
    with_save!(state, |s| {
        let tiles = s.terrain();
        let acres = s.acres();
        let (w, h, rgba) = render_terrain(
            &TerrainView {
                tiles: &tiles,
                acres: &acres,
            },
            px.unwrap_or(8),
        );
        Ok(Response::new(encode_png(w, h, &rgba)))
    })
}

#[derive(Serialize, Deserialize)]
pub struct MapMeta {
    pub acres: Vec<u16>,
    pub plaza: Plaza,
    pub design_tiles: Vec<u16>,
    pub design_dims: (usize, usize),
    pub terrain_dims: (usize, usize),
}

#[tauri::command]
pub fn map_meta(state: State<AppState>) -> CmdResult<MapMeta> {
    with_save!(state, |s| Ok(MapMeta {
        acres: s.acres(),
        plaza: s.plaza(),
        design_tiles: s.map_design_tiles(),
        design_dims: s.map_design_dims(),
        terrain_dims: (TILES_W, TILES_H),
    }))
}

#[tauri::command]
pub fn set_acres(state: State<AppState>, acres: Vec<u16>) -> CmdResult<()> {
    with_save_mut!(state, |s| {
        s.set_acres(&acres);
        Ok(())
    })
}

#[tauri::command]
pub fn set_plaza(state: State<AppState>, plaza: Plaza) -> CmdResult<()> {
    with_save_mut!(state, |s| {
        s.set_plaza(plaza);
        Ok(())
    })
}

#[tauri::command]
pub fn set_design_tiles(state: State<AppState>, tiles: Vec<u16>) -> CmdResult<()> {
    with_save_mut!(state, |s| {
        s.set_map_design_tiles(&tiles);
        Ok(())
    })
}

#[derive(Deserialize, Clone, Copy)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

fn with_layer<T>(
    s: &mut nhse_core::HorizonSave,
    layer: usize,
    f: impl FnOnce(&mut ItemGrid) -> T,
) -> T {
    let mut g = s.field_layer(layer);
    let r = f(&mut g);
    s.set_field_layer(layer, &g);
    r
}

#[tauri::command]
pub fn field_clear(
    state: State<AppState>,
    layer: usize,
    kind: ClearKind,
    rect: Option<Rect>,
) -> CmdResult<usize> {
    with_save_mut!(state, |s| {
        let l = s.field_layout();
        let r = rect.unwrap_or(Rect {
            x: 0,
            y: 0,
            w: l.width,
            h: l.height,
        });
        Ok(with_layer(s, layer, |g| {
            fieldtools::clear(g, kind, r.x, r.y, r.w, r.h)
        }))
    })
}

#[tauri::command]
pub fn field_water(
    state: State<AppState>,
    layer: usize,
    all: bool,
    rect: Option<Rect>,
) -> CmdResult<usize> {
    with_save_mut!(state, |s| {
        let l = s.field_layout();
        let r = rect.unwrap_or(Rect {
            x: 0,
            y: 0,
            w: l.width,
            h: l.height,
        });
        Ok(with_layer(s, layer, |g| {
            fieldtools::water_flowers(g, all, r.x, r.y, r.w, r.h)
        }))
    })
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpawnSource {
    Item {
        item: Item,
        count: usize,
    },
    Diy {
        min: u16,
        max: u16,
        alphabetical: bool,
        count: usize,
    },
    List {
        items: Vec<Item>,
        dropped: bool,
    },
}

#[tauri::command]
pub fn field_spawn(
    state: State<AppState>,
    layer: usize,
    x: usize,
    y: usize,
    arrange: Arrangement,
    source: SpawnSource,
    overwrite: bool,
) -> CmdResult<usize> {
    let lang = state.lang();
    with_save_mut!(state, |s| {
        let items: Vec<Item> = match source {
            SpawnSource::Item { item, count } => vec![item; count.min(10_000)],
            SpawnSource::Diy {
                min,
                max,
                alphabetical,
                count,
            } => {
                let cards = fieldtools::diy_cards(
                    min,
                    max,
                    alphabetical,
                    Some(nhse_core::data::strings(&lang)),
                );
                cards
                    .iter()
                    .cycle()
                    .take(cards.len() * count.clamp(1, 20))
                    .copied()
                    .collect()
            }
            SpawnSource::List { items, dropped } => items
                .into_iter()
                .filter(|i| !i.is_none())
                .map(|mut i| {
                    if dropped {
                        i.system = 0x20;
                    }
                    i
                })
                .collect(),
        };
        Ok(with_layer(s, layer, |g| {
            fieldtools::spawn(g, &items, x, y, arrange, overwrite)
        }))
    })
}

#[tauri::command]
pub fn field_replace(
    state: State<AppState>,
    layer: usize,
    old: Item,
    new: Item,
    id_only: bool,
    rect: Option<Rect>,
) -> CmdResult<usize> {
    with_save_mut!(state, |s| {
        let l = s.field_layout();
        let r = rect.unwrap_or(Rect {
            x: 0,
            y: 0,
            w: l.width,
            h: l.height,
        });
        match with_layer(s, layer, |g| g.replace_all(&old, &new, r.x, r.y, r.w, r.h, id_only)) {
            Some(n) => Ok(n),
            None => err("The new item has a different footprint than the old one; use \"match by item only\" or pick a same-sized item."),
        }
    })
}

#[tauri::command]
pub fn field_clean_extensions(state: State<AppState>, layer: usize) -> CmdResult<usize> {
    with_save_mut!(state, |s| Ok(
        with_layer(s, layer, |g| g.clear_dangling_extensions())
    ))
}

/// Tiles on the upper layer (1) with nothing underneath.
#[tauri::command]
pub fn field_unsupported(state: State<AppState>) -> CmdResult<Vec<(usize, usize)>> {
    with_save!(state, |s| Ok(s
        .field_layer(1)
        .unsupported_tiles(&s.field_layer(0))))
}

// --- dumps --------------------------------------------------------------------------------

#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DumpKind {
    /// Whole field item layer (`.nhl`)
    FieldLayer,
    /// One acre (32×32 items, row-major) of a field layer (`.nhl`)
    FieldAcre,
    /// Active flags of a layer (`.nhlf`)
    FieldFlags,
    /// Whole terrain (`.nht`)
    Terrain,
    /// One acre of terrain (16×16, row-major) (`.nht`)
    TerrainAcre,
    /// Buildings (`.nhb`)
    Buildings,
    /// Acre templates (`.nha`)
    Acres,
    /// Ground design tiles (`.nhmd`)
    Designs,
}

fn flags_range(s: &nhse_core::HorizonSave, layer: usize) -> std::ops::Range<usize> {
    let l = s.field_layout();
    let n = l.width * l.height;
    let o = s.main_ofs.field_item + 2 * n * 8 + layer * (n / 8);
    o..o + n / 8
}

#[tauri::command]
pub fn map_export(
    state: State<AppState>,
    kind: DumpKind,
    path: String,
    layer: Option<usize>,
    acre_x: Option<usize>,
    acre_y: Option<usize>,
) -> CmdResult<()> {
    with_save!(state, |s| {
        let layer = layer.unwrap_or(0);
        let data: Vec<u8> = match kind {
            DumpKind::FieldLayer => s.field_layer_bytes(layer).to_vec(),
            DumpKind::FieldAcre => {
                let g = s.field_layer(layer);
                let items = g.extract(
                    (acre_x.unwrap_or(0) * 32) as i64,
                    (acre_y.unwrap_or(0) * 32) as i64,
                    32,
                    32,
                );
                let mut d = vec![0u8; items.len() * 8];
                Item::write_array(&mut d, 0, &items);
                d
            }
            DumpKind::FieldFlags => s.main.data[flags_range(s, layer)].to_vec(),
            DumpKind::Terrain => s.terrain_bytes().to_vec(),
            DumpKind::TerrainAcre => {
                let t = s.terrain();
                let (ax, ay) = (acre_x.unwrap_or(0) * 16, acre_y.unwrap_or(0) * 16);
                let mut d = vec![0u8; 16 * 16 * 14];
                for y in 0..16 {
                    for x in 0..16 {
                        let tile = if ax + x < TILES_W && ay + y < TILES_H {
                            t[(ax + x) * TILES_H + ay + y]
                        } else {
                            TerrainTile::default()
                        };
                        tile.write(&mut d, (y * 16 + x) * 14);
                    }
                }
                d
            }
            DumpKind::Buildings => {
                let b = s.buildings();
                let mut d = vec![0u8; b.len() * 0x14];
                for (i, x) in b.iter().enumerate() {
                    x.write(&mut d, i * 0x14);
                }
                d
            }
            DumpKind::Acres => s.acres().iter().flat_map(|a| a.to_le_bytes()).collect(),
            DumpKind::Designs => s.map_design_bytes(),
        };
        std::fs::write(path, data)?;
        Ok(())
    })
}

#[tauri::command]
pub fn map_import(
    state: State<AppState>,
    kind: DumpKind,
    path: String,
    layer: Option<usize>,
    acre_x: Option<usize>,
    acre_y: Option<usize>,
) -> CmdResult<()> {
    let data = std::fs::read(&path)?;
    with_save_mut!(state, |s| {
        let layer = layer.unwrap_or(0);
        let size_err = |expect: usize| {
            err(format!(
                "This file is {} bytes; expected {} bytes.",
                data.len(),
                expect
            ))
        };
        match kind {
            DumpKind::FieldLayer => {
                let expect = s.field_layer_bytes(layer).len();
                match formats::field_layer_compatible(&data, expect) {
                    Some(d) => s.set_field_layer_bytes(layer, &d)?,
                    None => return size_err(expect),
                }
            }
            DumpKind::FieldAcre => {
                if data.len() != 32 * 32 * 8 {
                    return size_err(32 * 32 * 8);
                }
                let items = Item::read_array(&data, 0, 32 * 32);
                with_layer(s, layer, |g| {
                    g.paste(
                        (acre_x.unwrap_or(0) * 32) as i64,
                        (acre_y.unwrap_or(0) * 32) as i64,
                        32,
                        32,
                        &items,
                        false,
                    )
                });
            }
            DumpKind::FieldFlags => {
                let r = flags_range(s, layer);
                match formats::field_flags_compatible(&data, r.len()) {
                    Some(d) => s.main.data[r].copy_from_slice(&d),
                    None => return size_err(r.len()),
                }
            }
            DumpKind::Terrain => s.set_terrain_bytes(&data)?,
            DumpKind::TerrainAcre => {
                if data.len() != 16 * 16 * 14 {
                    return size_err(16 * 16 * 14);
                }
                let mut t = s.terrain();
                let (ax, ay) = (acre_x.unwrap_or(0) * 16, acre_y.unwrap_or(0) * 16);
                for y in 0..16 {
                    for x in 0..16 {
                        if ax + x < TILES_W && ay + y < TILES_H {
                            t[(ax + x) * TILES_H + ay + y] =
                                TerrainTile::read(&data, (y * 16 + x) * 14);
                        }
                    }
                }
                s.set_terrain(&t);
            }
            DumpKind::Buildings => {
                if data.len() != 46 * 0x14 && data.len() != 40 * 0x14 {
                    return size_err(46 * 0x14);
                }
                let mut b = s.buildings();
                for (i, x) in b.iter_mut().enumerate().take(data.len() / 0x14) {
                    *x = nhse_core::structs::building::Building::read(&data, i * 0x14);
                }
                s.set_buildings(&b);
            }
            DumpKind::Acres => {
                let cur = s.acres();
                if data.len() != cur.len() * 2 {
                    return size_err(cur.len() * 2);
                }
                s.set_acres(&nhse_core::util::bytes::read_u16_array(&data));
            }
            DumpKind::Designs => s.import_map_design_bytes(&data)?,
        }
        Ok(())
    })
}
