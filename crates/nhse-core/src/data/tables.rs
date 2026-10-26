//! Lookup tables extracted from the original project (achievements, flags, recipes, …).
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementDef {
    pub index: u16,
    pub count: u8,
    pub t1: u32,
    pub t2: u32,
    pub t3: u32,
    pub t4: u32,
    pub t5: u32,
    #[serde(rename = "flagLand")]
    pub flag_land: i16,
    #[serde(rename = "flagPlayer")]
    pub flag_player: i16,
    pub name: String,
}

impl AchievementDef {
    pub fn max_threshold(&self) -> u32 {
        self.t1.max(self.t2).max(self.t3).max(self.t4).max(self.t5)
    }
    pub fn thresholds(&self) -> [u32; 5] {
        [self.t1, self.t2, self.t3, self.t4, self.t5]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagDef {
    #[serde(rename = "def")]
    pub default: i32,
    pub max: i32,
    pub index: u16,
    pub name: String,
    #[serde(default)]
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldItemDef {
    pub id: u16,
    pub dig: u16,
    pub pick: u16,
    pub name: String,
    pub kind: u8,
}

impl FieldItemDef {
    /// Item obtained when picking up / digging up this field item (falls back to itself).
    pub fn held_item_id(&self) -> u16 {
        if self.pick != 0xFFFE {
            self.pick
        } else if self.dig != 0xFFFE {
            self.dig
        } else {
            self.id
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemakeInfo {
    pub index: i16,
    pub item: u16,
    #[serde(rename = "bodyNum")]
    pub body_num: i8,
    pub body0: Vec<u8>,
    pub body1: Vec<u8>,
    pub fabric0: Vec<u8>,
    pub fabric1: Vec<u8>,
    #[serde(rename = "fabricVisibleOff")]
    pub fabric_visible_off: bool,
}

pub const NO_COLOR: u8 = 14;

impl RemakeInfo {
    pub fn has_body_color(&self, v: usize) -> bool {
        self.body0[v] != NO_COLOR || self.body1[v] != NO_COLOR
    }
    pub fn has_fabric_color(&self, v: usize) -> bool {
        self.fabric0[v] != NO_COLOR || self.fabric1[v] != NO_COLOR
    }
}

#[derive(Debug, Deserialize)]
struct RawTables {
    achievements: Vec<AchievementDef>,
    flags_player: Vec<FlagDef>,
    flags_land: Vec<FlagDef>,
    flags_house: Vec<FlagDef>,
    flags_villager: Vec<FlagDef>,
    flags_memory: Vec<FlagDef>,
    field_items: Vec<FieldItemDef>,
    remake_info: Vec<RemakeInfo>,
    remake_map: Vec<(u16, i16)>,
    recipes: Vec<(u16, u16)>,
    game_lists: HashMap<String, Vec<u16>>,
}

/// All static tables, indexed for fast lookup.
#[derive(Debug)]
pub struct Tables {
    pub achievements: Vec<AchievementDef>,
    pub flags_player: Vec<FlagDef>,
    pub flags_land: Vec<FlagDef>,
    pub flags_house: Vec<FlagDef>,
    pub flags_villager: Vec<FlagDef>,
    pub flags_memory: Vec<FlagDef>,
    pub field_items: HashMap<u16, FieldItemDef>,
    pub field_items_sorted: Vec<FieldItemDef>,
    pub remake_info: HashMap<i16, RemakeInfo>,
    pub remake_map: HashMap<u16, i16>,
    /// Recipe index → crafted item id (ordered by recipe index).
    pub recipes: Vec<(u16, u16)>,
    pub recipe_map: HashMap<u16, u16>,
    pub game_lists: HashMap<String, Vec<u16>>,
    pub enums: HashMap<String, Vec<(String, i64)>>,
}

impl Tables {
    pub fn achievement(&self, index: usize) -> Option<&AchievementDef> {
        self.achievements.iter().find(|a| a.index as usize == index)
    }
    pub fn list(&self, name: &str) -> &[u16] {
        self.game_lists
            .get(name)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
    pub fn remake_for_item(&self, item: u16) -> Option<&RemakeInfo> {
        let idx = self.remake_map.get(&item)?;
        self.remake_info.get(idx)
    }
}

static TABLES: OnceLock<Tables> = OnceLock::new();

pub fn tables() -> &'static Tables {
    TABLES.get_or_init(|| {
        let raw: RawTables = serde_json::from_str(include_str!("../../data/tables.json"))
            .expect("embedded tables.json");
        let enums: HashMap<String, Vec<(String, i64)>> =
            serde_json::from_str(include_str!("../../data/enums.json"))
                .expect("embedded enums.json");
        let mut field_items_sorted = raw.field_items.clone();
        field_items_sorted.sort_by_key(|f| f.id);
        Tables {
            achievements: raw.achievements,
            flags_player: raw.flags_player,
            flags_land: raw.flags_land,
            flags_house: raw.flags_house,
            flags_villager: raw.flags_villager,
            flags_memory: raw.flags_memory,
            field_items: raw.field_items.into_iter().map(|f| (f.id, f)).collect(),
            field_items_sorted,
            remake_info: raw.remake_info.into_iter().map(|r| (r.index, r)).collect(),
            remake_map: raw.remake_map.into_iter().collect(),
            recipe_map: raw.recipes.iter().copied().collect(),
            recipes: raw.recipes,
            game_lists: raw.game_lists,
            enums,
        }
    })
}

/// Name of an enum value (e.g. `enum_name("Reaction", 3) == "Joy"`).
pub fn enum_name(enum_name: &str, value: i64) -> Option<&'static str> {
    tables()
        .enums
        .get(enum_name)?
        .iter()
        .find(|(_, v)| *v == value)
        .map(|(n, _)| n.as_str())
}

/// All `(name, value)` pairs of an enum.
pub fn enum_values(enum_name: &str) -> &'static [(String, i64)] {
    tables()
        .enums
        .get(enum_name)
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}

/// Value of an enum member by name.
pub fn enum_value(enum_name: &str, member: &str) -> Option<i64> {
    tables()
        .enums
        .get(enum_name)?
        .iter()
        .find(|(n, _)| n == member)
        .map(|(_, v)| *v)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn loads() {
        let t = tables();
        assert!(t.achievements.len() > 90);
        assert!(t.recipes.len() > 900);
        assert_eq!(enum_name("Reaction", 3), Some("Joy"));
        assert_eq!(enum_value("ItemKind", "Unknown"), Some(255));
        assert!(t.field_items.contains_key(&60000));
    }
}
