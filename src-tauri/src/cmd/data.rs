//! Static game data for the UI (names, kinds, enums, …).
use crate::error::CmdResult;
use crate::state::AppState;
use nhse_core::data::items::{item_count, item_kind, item_size_type, kinds, max_stack, size_dimensions};
use nhse_core::data::tables::{enum_name, tables, FlagDef, NO_COLOR};
use nhse_core::data::{strings, GameStrings};
use nhse_core::map::render::item_color_table;
use serde::Serialize;
use std::collections::BTreeMap;
use tauri::State;

#[derive(Serialize)]
pub struct FieldItemInfo {
    pub id: u16,
    pub name: String,
    pub kind: u8,
    pub kind_name: String,
    pub held: u16,
    pub color: u32,
}

#[derive(Serialize)]
pub struct VillagerInfo {
    pub internal: String,
    pub name: String,
    pub species: u8,
    pub variant: u8,
    pub phrase: String,
    pub has_template: bool,
}

#[derive(Serialize)]
pub struct FlagInfo {
    pub index: u16,
    pub name: String,
    pub label: String,
    pub default: i32,
    pub max: i32,
    pub desc: String,
}

#[derive(Serialize)]
pub struct AchievementInfo {
    pub index: u16,
    pub name: String,
    pub label: String,
    pub tiers: u8,
    pub thresholds: [u32; 5],
}

#[derive(Serialize)]
pub struct GameData {
    pub lang: String,
    /// Display names by item id (unique, never empty).
    pub items: Vec<String>,
    /// Whether the item has a real (non-placeholder) name.
    pub named: Vec<bool>,
    pub kinds: Vec<u8>,
    pub kind_names: Vec<String>,
    pub colors: Vec<u32>,
    /// Footprint (w, h) in field tiles per item id, flattened `[w0, h0, w1, h1, …]`.
    pub sizes: Vec<u8>,
    pub field_items: Vec<FieldItemInfo>,
    pub villagers: Vec<VillagerInfo>,
    pub enums: BTreeMap<String, Vec<(String, i64)>>,
    pub recipes: Vec<(u16, u16)>,
    pub achievements: Vec<AchievementInfo>,
    pub flags_player: Vec<FlagInfo>,
    pub flags_land: Vec<FlagInfo>,
    pub flags_house: Vec<FlagInfo>,
    pub flags_villager: Vec<FlagInfo>,
    pub flags_memory: Vec<FlagInfo>,
    pub lists: BTreeMap<String, Vec<u16>>,
    pub mail: Vec<(String, String)>,
    pub stationery: Vec<u16>,
    pub npc_senders: Vec<(u8, String, String)>,
    pub box_skins: Vec<(u8, String)>,
    pub mail_languages: Vec<String>,
    /// Ordinal values of item kinds the UI needs for special editors.
    pub special_kinds: BTreeMap<String, u8>,
}

fn flags(defs: &[FlagDef], s: &GameStrings) -> Vec<FlagInfo> {
    defs.iter()
        .map(|f| FlagInfo {
            index: f.index,
            name: f.name.clone(),
            label: s.translate_internal(&f.name).to_string(),
            default: f.default,
            max: f.max,
            desc: f.desc.clone(),
        })
        .collect()
}

#[tauri::command]
pub fn game_data(state: State<AppState>, lang: Option<String>) -> CmdResult<GameData> {
    let lang = lang.unwrap_or_else(|| state.lang());
    let s = strings(&lang);
    let t = tables();
    let n = item_count();
    let (colors, field_colors) = item_color_table();
    let fc: std::collections::HashMap<u16, u32> = field_colors.into_iter().collect();
    let mut sizes = Vec::with_capacity(n * 2);
    for i in 0..n {
        let (w, h) = size_dimensions(item_size_type(i as u16));
        sizes.push(w);
        sizes.push(h);
    }
    let templates: std::collections::HashSet<&String> = nhse_core::editing::villagers::available_templates().iter().collect();
    let villagers = s
        .villagers
        .iter()
        .filter_map(|(internal, name)| {
            let (species, variant) = nhse_core::structs::villager::parse_internal_name(internal)?;
            Some(VillagerInfo {
                internal: internal.clone(),
                name: name.clone(),
                species,
                variant,
                phrase: s.villager_phrase(internal),
                has_template: templates.contains(internal),
            })
        })
        .collect();
    let kd = kinds();
    let special_kinds: BTreeMap<String, u8> = [
        ("flower", kd.flower),
        ("diy_recipe", kd.diy_recipe),
        ("message_bottle", kd.message_bottle),
        ("fossil_unknown", kd.fossil_unknown),
        ("tree", kd.tree),
        ("insect", kd.insect),
        ("fish", kd.fish),
        ("dive_fish", kd.dive_fish),
        ("fossil", kd.fossil),
        ("music", kd.music),
        ("picture", kd.picture),
        ("sculpture", kd.sculpture),
        ("rug", kd.rug),
        ("room_wall", kd.room_wall),
        ("room_floor", kd.room_floor),
        ("ftr_wall", kd.ftr_wall),
    ]
    .into_iter()
    .map(|(a, b)| (a.to_string(), b))
    .collect();
    let enum_list = [
        "ItemKind", "Reaction", "VillagerPersonality", "VillagerSpecies", "Hemisphere", "AirportColor", "IslandFlowers",
        "TurnipPattern", "VisitorNPC", "ItemWrappingPaper", "ItemRemakeCommonPattern", "OutsideAcre", "BuildingType",
        "TerrainUnitModel", "SoundAmbientKind", "WallType", "RoofType", "DoorKind", "BridgeType", "BridgeMaterial",
        "SlopeType", "FieldItemKind", "ItemCustomColor", "ItemMenuIconType",
    ];
    Ok(GameData {
        lang: s.lang.to_string(),
        items: s.items_display.clone(),
        named: s.items.iter().enumerate().map(|(i, x)| i == 0 || !x.trim().is_empty()).collect(),
        kinds: (0..n).map(|i| item_kind(i as u16)).collect(),
        kind_names: (0..=255).map(|k| enum_name("ItemKind", k).unwrap_or("").to_string()).collect(),
        colors,
        sizes,
        field_items: t
            .field_items_sorted
            .iter()
            .map(|f| FieldItemInfo {
                id: f.id,
                name: f.name.clone(),
                kind: f.kind,
                kind_name: enum_name("FieldItemKind", f.kind as i64).unwrap_or("").to_string(),
                held: f.held_item_id(),
                color: fc.get(&f.id).copied().unwrap_or(0x006400),
            })
            .collect(),
        villagers,
        enums: enum_list.iter().map(|e| (e.to_string(), nhse_core::data::enum_values(e).to_vec())).collect(),
        recipes: t.recipes.clone(),
        achievements: t
            .achievements
            .iter()
            .map(|a| AchievementInfo {
                index: a.index,
                name: a.name.clone(),
                label: s.translate_internal(&a.name).to_string(),
                tiers: a.count,
                thresholds: a.thresholds(),
            })
            .collect(),
        flags_player: flags(&t.flags_player, s),
        flags_land: flags(&t.flags_land, s),
        flags_house: flags(&t.flags_house, s),
        flags_villager: flags(&t.flags_villager, s),
        flags_memory: flags(&t.flags_memory, s),
        lists: t.game_lists.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        mail: s.mail.clone(),
        stationery: nhse_core::structs::mail::STATIONERY.to_vec(),
        npc_senders: nhse_core::structs::mail::NPC_SENDERS.iter().map(|(a, b, c)| (*a, b.to_string(), c.to_string())).collect(),
        box_skins: nhse_core::structs::mail::BOX_SKINS.iter().map(|(a, b)| (*a, b.to_string())).collect(),
        mail_languages: nhse_core::structs::mail::LANGUAGES.iter().map(|x| x.to_string()).collect(),
        special_kinds,
    })
}

#[derive(Serialize)]
pub struct Variant {
    pub index: u8,
    pub name: String,
    pub colors: String,
}

#[derive(Serialize)]
pub struct ItemInfo {
    pub id: u16,
    pub name: String,
    pub kind: u8,
    pub kind_name: String,
    pub max_stack: Option<u16>,
    pub width: u8,
    pub height: u8,
    pub is_flower_gene: bool,
    pub is_recipe: bool,
    pub is_fossil_unknown: bool,
    pub is_tree: bool,
    pub is_field_item: bool,
    pub body_label: Option<String>,
    pub body: Vec<Variant>,
    pub fabric_label: Option<String>,
    pub fabric: Vec<Variant>,
    pub fabric_customizable: bool,
}

fn color_text(c0: u8, c1: u8) -> String {
    let name = |c: u8| enum_name("ItemCustomColor", c as i64).unwrap_or("?").to_string();
    if c0 == NO_COLOR {
        if c1 == NO_COLOR {
            return String::new();
        }
        return name(c1);
    }
    if c1 == NO_COLOR || c0 == c1 {
        return name(c0);
    }
    format!("{}-{}", name(c0), name(c1))
}

#[tauri::command]
pub fn item_info(state: State<AppState>, id: u16) -> CmdResult<ItemInfo> {
    let s = strings(&state.lang());
    let kd = kinds();
    let kind = item_kind(id);
    let (w, h) = size_dimensions(item_size_type(id));
    let remake = tables().remake_for_item(id);
    let key = format!("{id:05}");
    let mut body = Vec::new();
    let mut fabric = Vec::new();
    if let Some(r) = remake {
        for i in 0..8usize {
            let name = s.body_color.get(&format!("{key}_{i}")).cloned();
            if r.has_body_color(i) || name.is_some() {
                body.push(Variant { index: i as u8, name: name.unwrap_or_default(), colors: color_text(r.body0[i], r.body1[i]) });
            }
            let fname = s.fabric_color.get(&format!("{key}_{i}")).cloned();
            if r.has_fabric_color(i) {
                fabric.push(Variant { index: i as u8, name: fname.unwrap_or_default(), colors: color_text(r.fabric0[i], r.fabric1[i]) });
            }
        }
    }
    Ok(ItemInfo {
        id,
        name: s.item_name(id),
        kind,
        kind_name: enum_name("ItemKind", kind as i64).unwrap_or("Unknown").to_string(),
        max_stack: max_stack(id),
        width: w,
        height: h,
        is_flower_gene: kd.is_flower_gene(kind, id),
        is_recipe: kind == kd.diy_recipe || kind == kd.message_bottle,
        is_fossil_unknown: kind == kd.fossil_unknown,
        is_tree: kind == kd.tree,
        is_field_item: id >= 60_000 && id < 0xFFFD,
        body_label: s.body_parts.get(&key).cloned(),
        body,
        fabric_label: s.fabric_parts.get(&key).cloned(),
        fabric_customizable: remake.is_some_and(|r| r.fabric0.iter().chain(r.fabric1.iter()).any(|c| *c != NO_COLOR) || s.fabric_parts.contains_key(&key)),
        fabric,
    })
}

/// Full descriptive name for a list of items (recipes, genes, fossils, …).
#[tauri::command]
pub fn item_names(state: State<AppState>, items: Vec<nhse_core::Item>) -> CmdResult<Vec<String>> {
    let s = strings(&state.lang());
    Ok(items.iter().map(|i| i.name(s)).collect())
}

/// Parses pasted text (hex codes or names) into items.
#[tauri::command]
pub fn parse_items(state: State<AppState>, text: String) -> CmdResult<(Vec<nhse_core::Item>, Vec<String>)> {
    let s = strings(&state.lang());
    Ok(nhse_core::editing::itemparse::parse_items(&text, s))
}

#[tauri::command]
pub fn items_to_cheat(items: Vec<nhse_core::Item>, offset: u32) -> CmdResult<String> {
    Ok(nhse_core::editing::formats::items_to_cheat(&items, offset))
}
