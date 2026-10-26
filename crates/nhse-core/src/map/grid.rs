//! A column-major grid of items (field item layers and room layers).
use crate::data::items::{item_size_type, size_dimensions};
use crate::structs::Item;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct ItemGrid {
    pub width: usize,
    pub height: usize,
    /// `tiles[x * height + y]`
    pub tiles: Vec<Item>,
}

/// Result of a placement check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Placement {
    Free,
    Collision,
    OutOfBounds,
}

impl ItemGrid {
    pub fn new(width: usize, height: usize, tiles: Vec<Item>) -> Self {
        assert_eq!(tiles.len(), width * height);
        Self {
            width,
            height,
            tiles,
        }
    }

    #[inline]
    pub fn index(&self, x: usize, y: usize) -> usize {
        x * self.height + y
    }

    pub fn contains(&self, x: i64, y: i64) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    pub fn get(&self, x: usize, y: usize) -> Item {
        self.tiles[self.index(x, y)]
    }

    pub fn get_safe(&self, x: i64, y: i64) -> Item {
        if self.contains(x, y) {
            self.get(x as usize, y as usize)
        } else {
            Item::NONE
        }
    }

    pub fn set(&mut self, x: usize, y: usize, it: Item) {
        let i = self.index(x, y);
        self.tiles[i] = it;
    }

    /// Footprint of an item in grid units (rotation applied, buried/dropped items are 1×1 tile).
    pub fn footprint(it: &Item) -> (usize, usize) {
        if it.is_buried() || it.is_dropped() {
            return (2, 2);
        }
        let (mut w, mut h) = size_dimensions(item_size_type(it.display_id()));
        if it.rotation() & 1 == 1 {
            std::mem::swap(&mut w, &mut h);
        }
        (w as usize, h as usize)
    }

    fn clamped_footprint(&self, it: &Item, x: usize, y: usize) -> (usize, usize) {
        let (w, h) = Self::footprint(it);
        (
            w.min(self.width.saturating_sub(x)),
            h.min(self.height.saturating_sub(y)),
        )
    }

    pub fn check_placement(
        &self,
        it: &Item,
        x: usize,
        y: usize,
        ignore: Option<(usize, usize)>,
    ) -> Placement {
        let (w, h) = Self::footprint(it);
        if x + w > self.width || y + h > self.height {
            return Placement::OutOfBounds;
        }
        for ix in 0..w {
            for iy in 0..h {
                let t = self.get(x + ix, y + iy);
                if t.is_none() {
                    continue;
                }
                // the tile belongs to the item being replaced
                if let Some((rx, ry)) = ignore {
                    if (x + ix, y + iy) == (rx, ry) {
                        continue;
                    }
                    if t.is_extension()
                        && (x + ix) as i64 - t.extension_x() as i64 == rx as i64
                        && (y + iy) as i64 - t.extension_y() as i64 == ry as i64
                    {
                        continue;
                    }
                }
                return Placement::Collision;
            }
        }
        Placement::Free
    }

    /// Finds the root tile of the item covering (x, y), following extension tiles.
    pub fn root_of(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        let t = self.get(x, y);
        if t.is_none() {
            return None;
        }
        if !t.is_extension() {
            return Some((x, y));
        }
        let rx = x as i64 - t.extension_x() as i64;
        let ry = y as i64 - t.extension_y() as i64;
        if self.contains(rx, ry) {
            let r = self.get(rx as usize, ry as usize);
            if r.id == t.extension_item_id() {
                return Some((rx as usize, ry as usize));
            }
        }
        None
    }

    pub fn set_extensions(&mut self, root: &Item, x: usize, y: usize) {
        let (w, h) = self.clamped_footprint(root, x, y);
        for ix in 0..w {
            for iy in 0..h {
                if ix == 0 && iy == 0 {
                    continue;
                }
                self.set(x + ix, y + iy, Item::extension(root.id, ix as u8, iy as u8));
            }
        }
    }

    pub fn delete_extensions(&mut self, root: &Item, x: usize, y: usize) {
        let (w, h) = self.clamped_footprint(root, x, y);
        for ix in 0..w {
            for iy in 0..h {
                if ix == 0 && iy == 0 {
                    continue;
                }
                let t = self.get(x + ix, y + iy);
                if t.is_extension() {
                    self.set(x + ix, y + iy, Item::NONE);
                }
            }
        }
    }

    /// Places an item with its extension tiles. `force` skips the collision check.
    pub fn place(
        &mut self,
        it: Item,
        x: usize,
        y: usize,
        auto_extension: bool,
        force: bool,
    ) -> Placement {
        if !force {
            match self.check_placement(&it, x, y, None) {
                Placement::Free => {}
                p => return p,
            }
        }
        if auto_extension {
            self.set_extensions(&it, x, y);
        }
        self.set(x, y, it);
        Placement::Free
    }

    /// Removes the item covering (x, y) including its extension tiles.
    pub fn remove_at(&mut self, x: usize, y: usize) -> Option<Item> {
        let (rx, ry) = self.root_of(x, y).or_else(|| {
            // dangling extension: just clear the tile
            if !self.get(x, y).is_none() {
                Some((x, y))
            } else {
                None
            }
        })?;
        let root = self.get(rx, ry);
        if root.is_root() {
            self.delete_extensions(&root, rx, ry);
        }
        self.set(rx, ry, Item::NONE);
        Some(root)
    }

    /// Clears every tile inside the rectangle matching `pred`. Returns how many were removed.
    pub fn remove_where(
        &mut self,
        x0: usize,
        y0: usize,
        w: usize,
        h: usize,
        pred: impl Fn(&Item) -> bool,
    ) -> usize {
        let mut n = 0;
        for x in x0..(x0 + w).min(self.width) {
            for y in y0..(y0 + h).min(self.height) {
                let t = self.get(x, y);
                if !t.is_none() && pred(&t) {
                    self.set(x, y, Item::NONE);
                    n += 1;
                }
            }
        }
        n
    }

    /// Applies `f` to every root tile inside the rectangle matching `pred`.
    pub fn modify_where(
        &mut self,
        x0: usize,
        y0: usize,
        w: usize,
        h: usize,
        pred: impl Fn(&Item) -> bool,
        f: impl Fn(&mut Item),
    ) -> usize {
        let mut n = 0;
        for x in x0..(x0 + w).min(self.width) {
            for y in y0..(y0 + h).min(self.height) {
                let i = self.index(x, y);
                if pred(&self.tiles[i]) {
                    f(&mut self.tiles[i]);
                    n += 1;
                }
            }
        }
        n
    }

    /// Replaces every occurrence of `old` (exact match) with `new` inside the rectangle.
    /// Returns `None` if the footprints differ (could overlap neighbours).
    #[allow(clippy::too_many_arguments)]
    pub fn replace_all(
        &mut self,
        old: &Item,
        new: &Item,
        x0: usize,
        y0: usize,
        w: usize,
        h: usize,
        match_id_only: bool,
    ) -> Option<usize> {
        if Self::footprint(old) != Self::footprint(new) && !match_id_only {
            return None;
        }
        let mut n = 0;
        for x in x0..(x0 + w).min(self.width) {
            for y in y0..(y0 + h).min(self.height) {
                let t = self.get(x, y);
                if !t.is_root() {
                    continue;
                }
                let hit = if match_id_only {
                    t.id == old.id
                } else {
                    t == *old
                };
                if !hit {
                    continue;
                }
                let mut repl = *new;
                if match_id_only {
                    // keep the existing rotation / buried state
                    repl.system = (repl.system & !3) | (t.system & 3);
                }
                if Self::footprint(&t) != Self::footprint(&repl) {
                    continue;
                }
                self.delete_extensions(&t, x, y);
                self.set(x, y, repl);
                self.set_extensions(&repl, x, y);
                n += 1;
            }
        }
        Some(n)
    }

    /// Removes extension tiles whose root is missing. Returns how many were cleared.
    pub fn clear_dangling_extensions(&mut self) -> usize {
        let mut n = 0;
        for x in 0..self.width {
            for y in 0..self.height {
                let t = self.get(x, y);
                if t.is_extension() && self.root_of(x, y).is_none() {
                    self.set(x, y, Item::NONE);
                    n += 1;
                }
            }
        }
        n
    }

    /// Extracts a rectangle (row-major `w*h` result) for copy/paste and acre dumps.
    pub fn extract(&self, x0: i64, y0: i64, w: usize, h: usize) -> Vec<Item> {
        let mut out = Vec::with_capacity(w * h);
        for y in 0..h as i64 {
            for x in 0..w as i64 {
                out.push(self.get_safe(x0 + x, y0 + y));
            }
        }
        out
    }

    /// Pastes a row-major rectangle; tiles outside the grid are skipped. Returns tiles written.
    pub fn paste(
        &mut self,
        x0: i64,
        y0: i64,
        w: usize,
        h: usize,
        data: &[Item],
        skip_none: bool,
    ) -> usize {
        let mut n = 0;
        for y in 0..h {
            for x in 0..w {
                let it = data[y * w + x];
                if skip_none && it.is_none() {
                    continue;
                }
                let (tx, ty) = (x0 + x as i64, y0 + y as i64);
                if self.contains(tx, ty) {
                    self.set(tx as usize, ty as usize, it);
                    n += 1;
                }
            }
        }
        n
    }

    /// Coordinates of tiles on this layer that have nothing underneath on `base`.
    pub fn unsupported_tiles(&self, base: &ItemGrid) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for x in 0..self.width {
            for y in 0..self.height {
                if !self.get(x, y).is_none() && base.get(x, y).is_none() {
                    out.push((x, y));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn place_and_remove() {
        let mut g = ItemGrid::new(10, 10, vec![Item::NONE; 100]);
        let it = Item::new(100);
        assert_eq!(g.place(it, 2, 2, true, false), Placement::Free);
        let (w, h) = ItemGrid::footprint(&it);
        assert!(g.get(2 + w - 1, 2 + h - 1).is_extension() || (w, h) == (1, 1));
        assert_eq!(g.root_of(2 + w - 1, 2 + h - 1), Some((2, 2)));
        assert_eq!(g.place(it, 2, 2, true, false), Placement::Collision);
        g.remove_at(2 + w - 1, 2 + h - 1);
        assert!(g.tiles.iter().all(|t| t.is_none()));
    }
}
