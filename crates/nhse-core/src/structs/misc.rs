//! Small fixed-layout structures (dates, turnip prices, museum, visitors, …).
use crate::structs::Item;
use crate::util::bits::{get_flag, set_flag};
use crate::util::bytes::*;
use crate::util::strings::{get_string, set_string_if_changed};
use serde::{Deserialize, Serialize};

/// `GSaveDate`: year u16, month u8, day u8.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl GDate {
    pub const SIZE: usize = 4;
    pub fn read(d: &[u8], o: usize) -> Self {
        Self {
            year: u16le(d, o),
            month: d[o + 2],
            day: d[o + 3],
        }
    }
    pub fn write(&self, d: &mut [u8], o: usize) {
        put_u16(d, o, self.year);
        d[o + 2] = self.month;
        d[o + 3] = self.day;
    }
    pub fn is_empty(&self) -> bool {
        self.year == 0 && self.month == 0 && self.day == 0
    }
    /// Days since 0001-01-01 (proleptic Gregorian), for date arithmetic.
    pub fn to_days(&self) -> i64 {
        let (y, m, d) = (self.year as i64, self.month as i64, self.day as i64);
        let a = (14 - m) / 12;
        let y2 = y + 4800 - a;
        let m2 = m + 12 * a - 3;
        d + (153 * m2 + 2) / 5 + 365 * y2 + y2 / 4 - y2 / 100 + y2 / 400 - 32045
    }
    pub fn from_days(jdn: i64) -> Self {
        let a = jdn + 32044;
        let b = (4 * a + 3) / 146097;
        let c = a - 146097 * b / 4;
        let d = (4 * c + 3) / 1461;
        let e = c - 1461 * d / 4;
        let m = (5 * e + 2) / 153;
        let day = e - (153 * m + 2) / 5 + 1;
        let month = m + 3 - 12 * (m / 10);
        let year = 100 * b + d - 4800 + m / 10;
        Self {
            year: year as u16,
            month: month as u8,
            day: day as u8,
        }
    }
    pub fn add_days(&self, n: i64) -> Self {
        Self::from_days(self.to_days() + n)
    }
    pub fn today() -> Self {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        // 1970-01-01 is JDN 2440588
        Self::from_days(2_440_588 + (secs / 86_400) as i64)
    }
}

impl std::fmt::Display for GDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// `GSaveDateMD`: month + day.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GDateMD {
    pub month: u8,
    pub day: u8,
}

impl GDateMD {
    pub fn read(d: &[u8], o: usize) -> Self {
        Self {
            month: d[o],
            day: d[o + 1],
        }
    }
    pub fn write(&self, d: &mut [u8], o: usize) {
        d[o] = self.month;
        d[o + 1] = self.day;
    }
}

/// `GSaveTime`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl GTime {
    pub fn read(d: &[u8], o: usize) -> Self {
        Self {
            year: u16le(d, o),
            month: d[o + 2],
            day: d[o + 3],
            hour: d[o + 4],
            minute: d[o + 5],
            second: d[o + 6],
        }
    }
    pub fn timestamp(&self) -> String {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

/// `GSaveShopKabu` — the turnip market for the current week.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Turnips {
    pub buy_price: u32,
    /// Sunday AM, Sunday PM, Monday AM, … Saturday PM (14 entries).
    pub sell: [u32; 14],
    pub pattern: u32,
    pub fever_start: u32,
}

impl Turnips {
    pub const SIZE: usize = 0x44;
    pub fn read(d: &[u8], o: usize) -> Self {
        let mut sell = [0u32; 14];
        for (i, s) in sell.iter_mut().enumerate() {
            *s = u32le(d, o + 4 + i * 4);
        }
        Self {
            buy_price: u32le(d, o),
            sell,
            pattern: u32le(d, o + 0x3C),
            fever_start: u32le(d, o + 0x40),
        }
    }
    pub fn write(&self, d: &mut [u8], o: usize) {
        put_u32(d, o, self.buy_price);
        for (i, s) in self.sell.iter().enumerate() {
            put_u32(d, o + 4 + i * 4, *s);
        }
        put_u32(d, o + 0x3C, self.pattern);
        put_u32(d, o + 0x40, self.fever_start);
    }
}

/// `GSaveRoomFloorWall`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomFloorWall {
    pub accent_wall: Item,
    pub wall: Item,
    pub floor: Item,
    pub accent_wall_design: u16,
    pub wall_design: u16,
    pub floor_design: u16,
    pub accent_wall_direction: u8,
    pub info_bit: u8,
    pub floor_direction: u8,
}

impl RoomFloorWall {
    pub const SIZE: usize = 0x24;
    pub fn read(d: &[u8], o: usize) -> Self {
        Self {
            accent_wall: Item::read(d, o),
            wall: Item::read(d, o + 8),
            floor: Item::read(d, o + 0x10),
            accent_wall_design: u16le(d, o + 0x18),
            wall_design: u16le(d, o + 0x1A),
            floor_design: u16le(d, o + 0x1C),
            accent_wall_direction: d[o + 0x1E],
            info_bit: d[o + 0x1F],
            floor_direction: d[o + 0x20],
        }
    }
    pub fn write(&self, d: &mut [u8], o: usize) {
        self.accent_wall.write(d, o);
        self.wall.write(d, o + 8);
        self.floor.write(d, o + 0x10);
        put_u16(d, o + 0x18, self.accent_wall_design);
        put_u16(d, o + 0x1A, self.wall_design);
        put_u16(d, o + 0x1C, self.floor_design);
        d[o + 0x1E] = self.accent_wall_direction;
        d[o + 0x1F] = self.info_bit;
        d[o + 0x20] = self.floor_direction;
    }
}

/// `GSavePlayerId`: land id + player base id (0x38 bytes).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerId {
    pub town_id: u32,
    pub town_name: String,
    pub island_ruby_type: u8,
    pub player_id: u32,
    pub player_name: String,
    pub gender: u32,
}

impl PlayerId {
    pub const SIZE: usize = 0x38;
    pub fn read(d: &[u8], o: usize) -> Self {
        Self {
            town_id: u32le(d, o),
            town_name: get_string(d, o + 4, 10),
            island_ruby_type: d[o + 0x18],
            player_id: u32le(d, o + 0x1C),
            player_name: get_string(d, o + 0x20, 10),
            gender: u32le(d, o + 0x34),
        }
    }
    pub fn write(&self, d: &mut [u8], o: usize) {
        put_u32(d, o, self.town_id);
        set_string_if_changed(d, o + 4, 10, &self.town_name);
        d[o + 0x18] = self.island_ruby_type;
        put_u32(d, o + 0x1C, self.player_id);
        set_string_if_changed(d, o + 0x20, 10, &self.player_name);
        put_u32(d, o + 0x34, self.gender);
    }
}

/// Museum donations (`GSaveMuseum`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Museum {
    pub level: i32,
    pub dates: Vec<GDate>,
    pub items: Vec<Item>,
    pub players: Vec<u8>,
}

impl Museum {
    pub const SIZE: usize = 0x3404;
    pub const ENTRIES: usize = 1024;
    pub fn read(d: &[u8], o: usize) -> Self {
        Self {
            level: i32le(d, o),
            dates: (0..Self::ENTRIES)
                .map(|i| GDate::read(d, o + 4 + i * 4))
                .collect(),
            items: Item::read_array(d, o + 0x1004, Self::ENTRIES),
            players: d[o + 0x3004..o + 0x3404].to_vec(),
        }
    }
    pub fn write(&self, d: &mut [u8], o: usize) {
        put_i32(d, o, self.level);
        for (i, dt) in self.dates.iter().enumerate().take(Self::ENTRIES) {
            dt.write(d, o + 4 + i * 4);
        }
        Item::write_array(
            d,
            o + 0x1004,
            &self.items[..self.items.len().min(Self::ENTRIES)],
        );
        let n = self.players.len().min(0x400);
        d[o + 0x3004..o + 0x3004 + n].copy_from_slice(&self.players[..n]);
    }

    /// Sorts donations chronologically, empty slots last.
    pub fn sort(&mut self) {
        let mut rows: Vec<(GDate, Item, u8)> = (0..self.items.len())
            .map(|i| (self.dates[i], self.items[i], self.players[i]))
            .collect();
        rows.sort_by(|a, b| {
            a.1.is_none()
                .cmp(&b.1.is_none())
                .then(a.0.is_empty().cmp(&b.0.is_empty()))
                .then(a.0.cmp(&b.0))
        });
        for (i, (dt, it, p)) in rows.into_iter().enumerate() {
            self.dates[i] = dt;
            self.items[i] = it;
            self.players[i] = p;
        }
    }

    /// Donates every item from `ids` not yet donated. Returns how many were added.
    pub fn donate_all(&mut self, ids: &[u16], player: u8, per_day: usize) -> usize {
        let have: std::collections::HashSet<u16> = self
            .items
            .iter()
            .filter(|i| !i.is_none())
            .map(|i| i.id)
            .collect();
        let mut latest = self
            .dates
            .iter()
            .filter(|d| !d.is_empty())
            .max()
            .copied()
            .unwrap_or_else(GDate::today);
        let mut added = 0;
        for &id in ids {
            if have.contains(&id) {
                continue;
            }
            let Some(slot) = self.items.iter().position(|i| i.is_none()) else {
                break;
            };
            if added % per_day.max(1) == 0 {
                latest = latest.add_days(1);
            }
            self.items[slot] = Item::new(id);
            self.dates[slot] = latest;
            self.players[slot] = player;
            added += 1;
        }
        added
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.items.len() {
            self.items[index] = Item::NONE;
            self.dates[index] = GDate::default();
            self.players[index] = 0;
        }
    }
}

/// `GSaveVisitorNpc` — the weekly special visitor schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visitors {
    pub schedule: [i32; 7],
    pub create_pos: [f32; 3],
    pub pos2: [f32; 3],
    pub is_normal_live: [bool; 7],
    pub is_birthday_live: [bool; 7],
    pub init_schedule: [i32; 7],
    pub flags_14: [bool; 14],
    pub day_wisp: i32,
    pub day_celeste: i32,
}

impl Visitors {
    pub const SIZE: usize = 0x78;
    pub fn read(d: &[u8], o: usize) -> Self {
        let f = |x: usize| f32::from_bits(u32le(d, o + x));
        Self {
            schedule: std::array::from_fn(|i| i32le(d, o + i * 4)),
            create_pos: [f(0x1C), f(0x20), f(0x24)],
            pos2: [f(0x28), f(0x2C), f(0x30)],
            is_normal_live: std::array::from_fn(|i| d[o + 0x34 + i] != 0),
            is_birthday_live: std::array::from_fn(|i| d[o + 0x3B + i] != 0),
            init_schedule: std::array::from_fn(|i| i32le(d, o + 0x44 + i * 4)),
            flags_14: std::array::from_fn(|i| d[o + 0x60 + i] != 0),
            day_wisp: i32le(d, o + 0x70),
            day_celeste: i32le(d, o + 0x74),
        }
    }
    pub fn write(&self, d: &mut [u8], o: usize) {
        for i in 0..7 {
            put_i32(d, o + i * 4, self.schedule[i]);
            put_bool(d, o + 0x34 + i, self.is_normal_live[i]);
            put_bool(d, o + 0x3B + i, self.is_birthday_live[i]);
            put_i32(d, o + 0x44 + i * 4, self.init_schedule[i]);
        }
        for i in 0..3 {
            put_u32(d, o + 0x1C + i * 4, self.create_pos[i].to_bits());
            put_u32(d, o + 0x28 + i * 4, self.pos2[i].to_bits());
        }
        for i in 0..14 {
            put_bool(d, o + 0x60 + i, self.flags_14[i]);
        }
        put_i32(d, o + 0x70, self.day_wisp);
        put_i32(d, o + 0x74, self.day_celeste);
    }
}

/// Player reactions (`GSavePlayerManpu` / `GSavePlayerManpu15`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reactions {
    /// Learned reactions (64 entries before 1.5.0, 256 afterwards). `0` = empty slot.
    pub learned: Vec<u8>,
    /// Reaction wheel (8 slots).
    pub wheel: [u8; 8],
    pub new_flags: Vec<bool>,
}

impl Reactions {
    pub fn capacity(format: u8) -> usize {
        if format >= 15 {
            256
        } else {
            64
        }
    }
    pub fn read(d: &[u8], o: usize, format: u8) -> Self {
        let n = Self::capacity(format);
        Self {
            learned: d[o..o + n].to_vec(),
            wheel: std::array::from_fn(|i| d[o + n + i]),
            new_flags: (0..n).map(|i| d[o + n + 8 + i] != 0).collect(),
        }
    }
    pub fn write(&self, d: &mut [u8], o: usize, format: u8) {
        let n = Self::capacity(format);
        for i in 0..n {
            d[o + i] = self.learned.get(i).copied().unwrap_or(0);
            put_bool(
                d,
                o + n + 8 + i,
                self.new_flags.get(i).copied().unwrap_or(false),
            );
        }
        d[o + n..o + n + 8].copy_from_slice(&self.wheel);
    }
    /// Learns every valid reaction not yet known. Returns how many were added.
    pub fn learn_all(&mut self) -> usize {
        let mut added = 0;
        for (name, v) in crate::data::tables::enum_values("Reaction") {
            let v = *v as u8;
            if v == 0 || name.starts_with("UNUSED") || self.learned.contains(&v) {
                continue;
            }
            if let Some(slot) = self.learned.iter().position(|x| *x == 0) {
                self.learned[slot] = v;
                added += 1;
            }
        }
        added
    }
}

/// The DIY recipe book: four 2048-bit bitfields (known, made, new, favorite).
pub struct RecipeBook;

impl RecipeBook {
    pub const SIZE: usize = 0x400;
    pub const COUNT: usize = 0x100 * 8;
    pub const BRIDGE_KIT: u16 = 0x1CA;
    pub const CAMPSITE_KIT: u16 = 0x1CD;
    pub fn get(d: &[u8], o: usize, field: usize, recipe: usize) -> bool {
        get_flag(d, o + field * 0x100, recipe)
    }
    pub fn set(d: &mut [u8], o: usize, field: usize, recipe: usize, v: bool) {
        set_flag(d, o + field * 0x100, recipe, v)
    }
}

/// Achievement / Nook Miles+ data (`CountAchievement` …).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievements {
    pub counts: Vec<u32>,
    pub read: Vec<[bool; 6]>,
    pub dates: Vec<[GDate; 6]>,
    pub flags: Vec<u8>,
}

impl Achievements {
    pub const MAX: usize = 512;
    pub const SIZE: usize = Self::MAX * 35;
    pub fn read(d: &[u8], o: usize) -> Self {
        let n = Self::MAX;
        Self {
            counts: (0..n).map(|i| u32le(d, o + i * 4)).collect(),
            read: (0..n)
                .map(|i| std::array::from_fn(|j| d[o + 0x800 + i * 6 + j] != 0))
                .collect(),
            dates: (0..n)
                .map(|i| std::array::from_fn(|j| GDate::read(d, o + 0x1400 + (i * 6 + j) * 4)))
                .collect(),
            flags: d[o + 0x4400..o + 0x4600].to_vec(),
        }
    }
    pub fn write(&self, d: &mut [u8], o: usize) {
        for i in 0..Self::MAX {
            put_u32(d, o + i * 4, self.counts[i]);
            for j in 0..6 {
                put_bool(d, o + 0x800 + i * 6 + j, self.read[i][j]);
                self.dates[i][j].write(d, o + 0x1400 + (i * 6 + j) * 4);
            }
            d[o + 0x4400 + i] = self.flags[i];
        }
    }
    pub fn clear(&mut self, i: usize) {
        self.counts[i] = 0;
        self.read[i] = [false; 6];
        self.dates[i] = [GDate::default(); 6];
        self.flags[i] = 0;
    }
    /// Completes an achievement (all tiers, staggered dates starting at `start`).
    pub fn complete(&mut self, i: usize, start: GDate) {
        let Some(def) = crate::data::tables::tables().achievement(i) else {
            self.clear(i);
            return;
        };
        let max = def.max_threshold();
        if max != 0 {
            self.counts[i] = self.counts[i].max(max);
        }
        self.flags[i] = 1;
        let mut t = start;
        for j in 0..(def.count as usize).min(6) {
            if self.read[i][j] || !self.dates[i][j].is_empty() {
                continue;
            }
            self.dates[i][j] = t;
            self.read[i][j] = true;
            t = t.add_days(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dates() {
        let d = GDate {
            year: 2020,
            month: 3,
            day: 20,
        };
        assert_eq!(GDate::from_days(d.to_days()), d);
        assert_eq!(
            d.add_days(12),
            GDate {
                year: 2020,
                month: 4,
                day: 1
            }
        );
        assert_eq!(
            GDate {
                year: 2020,
                month: 12,
                day: 31
            }
            .add_days(1),
            GDate {
                year: 2021,
                month: 1,
                day: 1
            }
        );
    }
}
