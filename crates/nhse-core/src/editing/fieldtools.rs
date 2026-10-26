//! Bulk operations on field item layers.
use crate::data::items::{item_size_type, size_dimensions};
use crate::data::tables::{enum_value, tables};
use crate::map::grid::{ItemGrid, Placement};
use crate::structs::Item;
use serde::{Deserialize, Serialize};

/// Categories of field items that can be cleared in bulk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClearKind {
    Weeds,
    Trees,
    Flowers,
    Bushes,
    Plants,
    Fences,
    Stones,
    Holes,
    FieldObjects,
    PlacedItems,
    Shells,
    Branches,
    Everything,
}

fn fk_range(a: &str, b: &str) -> std::ops::RangeInclusive<u8> {
    (enum_value("FieldItemKind", a).unwrap_or(0) as u8)..=(enum_value("FieldItemKind", b).unwrap_or(0) as u8)
}

/// `FieldItemKind` of a field item id, if it is one.
pub fn field_kind(id: u16) -> Option<u8> {
    tables().field_items.get(&id).map(|f| f.kind)
}

pub fn matches(kind: ClearKind, it: &Item) -> bool {
    if it.is_none() {
        return false;
    }
    let id = it.display_id();
    let fk = field_kind(id);
    let in_kind = |a: &str, b: &str| fk.is_some_and(|k| fk_range(a, b).contains(&k));
    match kind {
        ClearKind::Weeds => in_kind("PltWeedAut0", "PltWeedWin1"),
        ClearKind::Trees => in_kind("PltTreeBamboo", "PltTreePalm"),
        ClearKind::Flowers => in_kind("PltFlwAnemone", "PltFlwYuri"),
        ClearKind::Bushes => in_kind("PltBushAzalea", "PltBushPlumeria"),
        ClearKind::Plants => in_kind("PltFlwAnemone", "PltWeedWin1"),
        ClearKind::Fences => in_kind("FenceBamboo", "FenceWoodWhite"),
        ClearKind::Stones => in_kind("StoneA", "StoneE"),
        ClearKind::Holes => fk == enum_value("FieldItemKind", "UnitIconHole").map(|v| v as u8),
        ClearKind::FieldObjects => fk.is_some(),
        ClearKind::PlacedItems => fk.is_none(),
        ClearKind::Shells => tables().list("Shells").contains(&id),
        ClearKind::Branches => id == 2500,
        ClearKind::Everything => true,
    }
}

/// Clears matching items (and their extension tiles) in a rectangle. Returns the count of items.
pub fn clear(g: &mut ItemGrid, kind: ClearKind, x0: usize, y0: usize, w: usize, h: usize) -> usize {
    let mut n = 0;
    for x in x0..(x0 + w).min(g.width) {
        for y in y0..(y0 + h).min(g.height) {
            let t = g.get(x, y);
            if !t.is_root() || !matches(kind, &t) {
                continue;
            }
            g.delete_extensions(&t, x, y);
            g.set(x, y, Item::NONE);
            n += 1;
        }
    }
    // dangling extensions of cleared kinds (e.g. an extension inside the rect whose root was outside)
    if kind == ClearKind::Everything {
        n += g.remove_where(x0, y0, w, h, |t| t.is_extension());
    }
    n
}

/// Waters every flower in the rectangle (`all` = gold watering by all visitors).
pub fn water_flowers(g: &mut ItemGrid, all: bool, x0: usize, y0: usize, w: usize, h: usize) -> usize {
    let flowers = fk_range("PltFlwAnemone", "PltFlwYuri");
    g.modify_where(
        x0,
        y0,
        w,
        h,
        |t| t.is_root() && field_kind(t.id).is_some_and(|k| flowers.contains(&k)),
        |t| t.water(all),
    )
}

/// How spawned items are laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arrangement {
    Square,
    Horizontal,
    Vertical,
}

/// Places `items` starting at (x, y) in a grid. Occupied/out-of-bounds cells are skipped.
/// Returns the number of items placed.
pub fn spawn(g: &mut ItemGrid, items: &[Item], x: usize, y: usize, arrange: Arrangement, overwrite: bool) -> usize {
    if items.is_empty() {
        return 0;
    }
    let (mut sx, mut sy) = ItemGrid::footprint(&items[0]);
    sx += sx % 2;
    sy += sy % 2;
    let per_row = match arrange {
        Arrangement::Square => ((items.len() as f64).sqrt().ceil() as usize).max(1),
        Arrangement::Horizontal => usize::MAX,
        Arrangement::Vertical => 1,
    };
    let (mut cx, mut cy) = (x, y);
    let mut col = 0;
    let mut placed = 0;
    let mut i = 0;
    let mut guard = 0;
    while i < items.len() && guard < 200_000 {
        guard += 1;
        if cy + sy > g.height {
            break;
        }
        let it = items[i];
        let ok = if cx + sx > g.width {
            false
        } else {
            match g.check_placement(&it, cx, cy, None) {
                Placement::Free => true,
                Placement::Collision => overwrite,
                Placement::OutOfBounds => false,
            }
        };
        if ok {
            if overwrite {
                let (w, h) = ItemGrid::footprint(&it);
                for ix in 0..w {
                    for iy in 0..h {
                        if cx + ix < g.width && cy + iy < g.height {
                            g.remove_at(cx + ix, cy + iy);
                        }
                    }
                }
            }
            g.place(it, cx, cy, true, true);
            placed += 1;
            i += 1;
        }
        col += 1;
        cx += sx;
        if col >= per_row || cx + sx > g.width {
            col = 0;
            cx = x;
            cy += sy;
        }
    }
    placed
}

/// DIY recipe cards for a recipe index range, optionally sorted by crafted item name.
pub fn diy_cards(min: u16, max: u16, alphabetical: bool, names: Option<&crate::data::GameStrings>) -> Vec<Item> {
    let mut v: Vec<(u16, u16)> = tables()
        .recipes
        .iter()
        .copied()
        .filter(|(r, _)| *r >= min && *r <= max && *r != 0x1CA && *r != 0x1CD)
        .collect();
    if alphabetical {
        if let Some(s) = names {
            v.sort_by_key(|(_, item)| s.item_name(*item).to_lowercase());
        }
    }
    v.into_iter()
        .map(|(r, _)| Item { id: Item::DIY_RECIPE, system: 0, additional: 0, free: r as u32 })
        .collect()
}

/// Footprint in tiles for UI previews.
pub fn footprint_of(id: u16) -> (u8, u8) {
    size_dimensions(item_size_type(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spawn_square() {
        let mut g = ItemGrid::new(64, 64, vec![Item::NONE; 64 * 64]);
        let items = vec![Item::new(0x0A54); 9];
        let n = spawn(&mut g, &items, 0, 0, Arrangement::Square, false);
        assert_eq!(n, 9);
        let roots = g.tiles.iter().filter(|t| t.is_root()).count();
        assert_eq!(roots, 9);
    }
}
