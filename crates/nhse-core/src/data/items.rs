//! Per-item metadata (kind, size, menu icon, …) derived from the game's item table.
use crate::data::tables::{enum_value, tables};
use std::sync::OnceLock;

static KIND: &[u8] = include_bytes!("../../resources/byte/item_kind.bin");
static SIZE: &[u8] = include_bytes!("../../resources/byte/item_size.bin");
static SORT: &[u8] = include_bytes!("../../resources/byte/item_sort_order.bin");
static MENU_ICON: &[u8] = include_bytes!("../../resources/byte/item_menuicon.bin");
static CLOTH_GROUP: &[u8] = include_bytes!("../../resources/byte/item_cloth_group.bin");

pub const KIND_UNKNOWN: u8 = 255;

/// Cached ordinal values of the `ItemKind` members the editor logic needs.
pub struct Kinds {
    pub flower: u8,
    pub flower_bud: u8,
    pub flower_seed: u8,
    pub unit_icon_flw_anemone: u8,
    pub unit_icon_flw_tulip: u8,
    pub diy_recipe: u8,
    pub message_bottle: u8,
    pub fossil_unknown: u8,
    pub tree: u8,
    pub weed: u8,
    pub insect: u8,
    pub fish: u8,
    pub dive_fish: u8,
    pub bush: u8,
    pub dummy_presentbox: u8,
    pub ore: u8,
    pub craft_material: u8,
    pub craft_phone_case: u8,
    pub craft_remake: u8,
    pub fence: u8,
    pub music: u8,
    pub ftr: u8,
    pub rug: u8,
    pub room_wall: u8,
    pub room_floor: u8,
    pub ftr_wall: u8,
    pub counter: u8,
    pub pillar: u8,
    pub fossil: u8,
    pub poster: u8,
    pub picture: u8,
    pub sculpture: u8,
    pub picture_fake: u8,
    pub sculpture_fake: u8,
    clothing: Vec<u8>,
    furniture_like: Vec<u8>,
}

fn k(name: &str) -> u8 {
    enum_value("ItemKind", name).unwrap_or(255) as u8
}

pub fn kinds() -> &'static Kinds {
    static K: OnceLock<Kinds> = OnceLock::new();
    K.get_or_init(|| Kinds {
        flower: k("Kind_Flower"),
        flower_bud: k("Kind_FlowerBud"),
        flower_seed: k("Kind_FlowerSeed"),
        unit_icon_flw_anemone: k("UnitIcon_FlwAnemone"),
        unit_icon_flw_tulip: k("UnitIcon_FlwTulip"),
        diy_recipe: k("Kind_DIYRecipe"),
        message_bottle: k("Kind_MessageBottle"),
        fossil_unknown: k("Kind_FossilUnknown"),
        tree: k("Kind_Tree"),
        weed: k("Kind_Weed"),
        insect: k("Kind_Insect"),
        fish: k("Kind_Fish"),
        dive_fish: k("Kind_DiveFish"),
        bush: k("Kind_Bush"),
        dummy_presentbox: k("Kind_DummyPresentbox"),
        ore: k("Kind_Ore"),
        craft_material: k("Kind_CraftMaterial"),
        craft_phone_case: k("Kind_CraftPhoneCase"),
        craft_remake: k("Kind_CraftRemake"),
        fence: k("Kind_Fence"),
        music: k("Kind_Music"),
        ftr: k("Kind_Ftr"),
        rug: k("Kind_Rug"),
        room_wall: k("Kind_RoomWall"),
        room_floor: k("Kind_RoomFloor"),
        ftr_wall: k("Kind_FtrWall"),
        counter: k("Kind_Counter"),
        pillar: k("Kind_Pillar"),
        fossil: k("Kind_Fossil"),
        poster: k("Kind_Poster"),
        picture: k("Kind_Picture"),
        sculpture: k("Kind_Sculpture"),
        picture_fake: k("Kind_PictureFake"),
        sculpture_fake: k("Kind_SculptureFake"),
        clothing: [
            "Bottoms_Long", "Bottoms_Middle", "Bottoms_Short", "Onepiece_Dress", "Onepiece_Long", "Onepiece_Middle",
            "Onepiece_Short", "Shoes_Boots", "Shoes_Pumps", "Top_Long", "Top_Middle", "Top_Short",
        ]
        .iter()
        .map(|n| k(n))
        .collect(),
        furniture_like: ["Ftr_1x1_Chair", "Ftr_1x1_Floor", "Ftr_2x1_Bed", "Ftr_2x1_Floor", "Ftr_2x2_Floor"].iter().map(|n| k(n)).collect(),
    })
}

impl Kinds {
    pub fn is_flower(&self, kind: u8) -> bool {
        (self.flower..=self.flower_bud).contains(&kind) || (self.unit_icon_flw_anemone..=self.unit_icon_flw_tulip).contains(&kind)
    }
    pub fn is_flower_picked(&self, kind: u8) -> bool {
        kind == self.flower || (self.unit_icon_flw_anemone..=self.unit_icon_flw_tulip).contains(&kind)
    }
    /// True if this item stores flower genes.
    pub fn is_flower_gene(&self, kind: u8, id: u16) -> bool {
        self.is_flower(kind) && (id >= 60_000 || !self.is_flower_picked(kind))
    }
    pub fn is_clothing(&self, kind: u8) -> bool {
        self.clothing.contains(&kind)
    }
    pub fn is_crafting(&self, kind: u8) -> bool {
        kind == self.ore || kind == self.craft_material || kind == self.craft_phone_case || kind == self.craft_remake
    }
    pub fn is_furniture(&self, kind: u8) -> bool {
        self.furniture_like.contains(&kind) || kind == self.ftr
    }
}

fn u16_at(d: &[u8], id: u16) -> Option<u16> {
    let o = id as usize * 2;
    (o + 1 < d.len()).then(|| u16::from_le_bytes([d[o], d[o + 1]]))
}

/// Number of item ids described by the item table.
pub fn item_count() -> usize {
    KIND.len()
}

/// `ItemKind` ordinal of an item id (field items resolve through what they drop).
pub fn item_kind(id: u16) -> u8 {
    match KIND.get(id as usize) {
        Some(k) => *k,
        None => field_item_kind(id),
    }
}

fn field_item_kind(id: u16) -> u8 {
    let t = tables();
    let Some(def) = t.field_items.get(&id) else { return KIND_UNKNOWN };
    let held = def.held_item_id();
    if held < 60_000 {
        let k = KIND.get(held as usize).copied().unwrap_or(KIND_UNKNOWN);
        if k != KIND_UNKNOWN {
            return k;
        }
    }
    field_kind_to_item_kind(def.kind)
}

/// Maps a `FieldItemKind` ordinal to an `ItemKind` ordinal (trees, flowers, weeds).
pub fn field_kind_to_item_kind(fk: u8) -> u8 {
    let fv = |n: &str| enum_value("FieldItemKind", n).unwrap_or(-1) as i64;
    let v = fk as i64;
    let kd = kinds();
    if (fv("PltTreeBamboo")..=fv("PltTreePalm")).contains(&v) {
        kd.tree
    } else if (fv("PltFlwAnemone")..=fv("PltFlwYuri")).contains(&v) {
        kd.flower
    } else if (fv("PltWeedAut0")..=fv("PltWeedWin1")).contains(&v) {
        kd.weed
    } else {
        KIND_UNKNOWN
    }
}

/// `ItemSizeType` ordinal (255 = unknown).
pub fn item_size_type(id: u16) -> u8 {
    SIZE.get(id as usize).copied().unwrap_or(255)
}

pub fn item_sort_order(id: u16) -> u8 {
    SORT.get(id as usize).copied().unwrap_or(255)
}

pub fn item_menu_icon(id: u16) -> u16 {
    u16_at(MENU_ICON, id).unwrap_or(u16::MAX)
}

pub fn item_cloth_group(id: u16) -> u16 {
    u16_at(CLOTH_GROUP, id).unwrap_or(u16::MAX)
}

/// Footprint (in half-tiles, i.e. field item grid units) for an `ItemSizeType` ordinal.
pub fn size_dimensions(size_type: u8) -> (u8, u8) {
    let name = crate::data::tables::enum_name("ItemSizeType", size_type as i64).unwrap_or("");
    match name {
        "S_1_0x1_0" => (2, 2),
        "S_2_0x1_0" => (4, 2),
        "S_2_0x2_0" => (4, 4),
        "S_3_0x1_0" => (6, 2),
        "S_3_0x2_0" => (6, 4),
        "S_3_0x3_0" => (6, 6),
        "S_1_0x0_5" => (2, 1),
        "S_2_0x0_5" => (4, 1),
        "S_1_5x1_5" => (3, 3),
        "S_1_0x0_5_Wall" => (2, 1),
        "S_0_5x1_0_Wall" => (1, 2),
        "S_1_0x1_0_Wall" => (2, 2),
        "S_1_0x1_5_Wall" => (2, 3),
        "S_1_0x2_0_Wall" => (2, 4),
        "S_2_0x1_0_Wall" => (4, 2),
        "S_2_0x1_5_Wall" => (4, 3),
        "S_2_0x2_0_Wall" => (4, 4),
        "S_1_0x1_0_Rug" => (2, 2),
        "S_2_0x1_0_Rug" => (4, 2),
        "S_2_0x2_0_Rug" => (4, 4),
        "S_3_0x2_0_Rug" => (6, 4),
        "S_3_0x3_0_Rug" => (6, 6),
        "S_4_0x3_0_Rug" => (8, 6),
        "S_4_0x4_0_Rug" => (8, 8),
        "S_5_0x4_0_Rug" => (10, 8),
        "S_5_0x5_0_Rug" => (10, 10),
        _ => (2, 2),
    }
}

/// Maximum stack size for an item, if it is stackable/known.
pub fn max_stack(id: u16) -> Option<u16> {
    let kd = kinds();
    let mut kind = item_kind(id);
    if kd.is_flower_picked(kind) {
        kind = kd.flower;
    }
    let name = crate::data::tables::enum_name("ItemKind", kind as i64)?;
    Some(match name {
        "Kind_CookingMaterial" | "Kind_CraftRemake" | "Kind_Fence" | "Kind_WeedLight" => 50,
        "Kind_PartyPopper" | "Kind_BlowBubble" | "Kind_FierworkHand" | "Kind_TreeSeedling" | "Kind_BushSeedling"
        | "Kind_VegeSeedling" | "Kind_Vegetable" | "Kind_FlowerSeed" | "Kind_Flower" | "Kind_Fruit" | "Kind_Mushroom"
        | "Kind_Turnip" | "Kind_FishBait" | "Kind_PitFallSeed" | "Kind_Medicine" | "Kind_Honeycomb" | "Kind_SnowCrystal"
        | "Kind_AutumnLeaf" | "Kind_Sakurapetal" | "Kind_XmasDeco" | "Kind_StarPiece" | "Kind_ShellDrift"
        | "Kind_WrappingPaper" | "Kind_Otoshidama" | "Kind_Money" | "Kind_BdayCupcake" | "Kind_JohnnyQuest"
        | "Kind_JohnnyQuestDust" | "Kind_PirateQuest" | "Kind_TailorTicket" | "Kind_MilePlaneTicket"
        | "Kind_SouvenirChocolate" => 10,
        "Kind_Weed" => 99,
        "Kind_CraftMaterial" | "Kind_Ore" | "Kind_EasterEgg" | "Kind_LoveCrystal" | "Kind_Candy" | "Kind_Vine" => 30,
        "Kind_YutaroWisp" | "Kind_RollanTicket" => 5,
        "Kind_Feather" => 3,
        "Unknown" => return None,
        n if KNOWN_SINGLE.contains(&n) => 1,
        _ => return None,
    })
}

const KNOWN_SINGLE: &[&str] = &[
    "Kind_Ftr", "Kind_Dishes", "Kind_Drink", "Kind_RoomWall", "Kind_RoomFloor", "Kind_Rug", "Kind_RugMyDesign", "Kind_Socks",
    "Kind_Cap", "Kind_Helmet", "Kind_Accessory", "Kind_Bag", "Kind_Umbrella", "Kind_FtrWall", "Kind_Counter", "Kind_Pillar",
    "Kind_FishingRod", "Kind_Net", "Kind_Shovel", "Kind_Axe", "Kind_Watering", "Kind_Slingshot", "Kind_ChangeStick",
    "Kind_WoodenStickTool", "Kind_Ladder", "Kind_GroundMaker", "Kind_RiverMaker", "Kind_CliffMaker", "Kind_HandBag",
    "Kind_Ocarina", "Kind_Panflute", "Kind_Tambourine", "Kind_MaracasCarnival", "Kind_StickLight", "Kind_StickLightColorful",
    "Kind_Uchiwa", "Kind_SubToolSensu", "Kind_Windmill", "Kind_Partyhorn", "Kind_Balloon", "Kind_HandheldPennant",
    "Kind_BigbagPresent", "Kind_JuiceFuzzyapple", "Kind_Megaphone", "Kind_SoySet", "Kind_FlowerShower", "Kind_Candyfloss",
    "Kind_SubToolDonut", "Kind_SubToolEat", "Kind_SubToolEatRemakeable", "Kind_Tapioca", "Kind_SubToolCan", "Kind_Icecandy",
    "Kind_SubToolIcecream", "Kind_SubToolIcesoft", "Kind_SubToolEatDrop", "Kind_SubToolGeneric", "Kind_Basket", "Kind_Lantern",
    "Kind_SubToolRemakeable", "Kind_Timer", "Kind_Gyroid", "Kind_GyroidScrap", "Kind_Tree", "Kind_Bush", "Kind_VegeTree",
    "Kind_FlowerBud", "Kind_TurnipExpired", "Kind_CraftPhoneCase", "Kind_Trash", "Kind_Insect", "Kind_Fish", "Kind_DiveFish",
    "Kind_ShellFish", "Kind_FishToy", "Kind_InsectToy", "Kind_Fossil", "Kind_FossilUnknown", "Kind_Music", "Kind_MusicMiss",
    "Kind_Bromide", "Kind_Poster", "Kind_HousePost", "Kind_DoorDeco", "Kind_DummyRecipe", "Kind_DummyDIYRecipe",
    "Kind_DummyHowtoBook", "Kind_LicenseItem", "Kind_BridgeItem", "Kind_SlopeItem", "Kind_DIYRecipe", "Kind_MessageBottle",
    "Kind_HousingKit", "Kind_HousingKitRcoQuest", "Kind_HousingKitBirdge", "Kind_FireworkM", "Kind_QuestWrapping",
    "Kind_QuestChristmasPresentbox", "Kind_LostQuest", "Kind_LostQuestDust", "Kind_TreasureQuest", "Kind_TreasureQuestDust",
    "Kind_HarvestDish", "Kind_RainbowFeather", "Kind_SettingLadder", "Kind_SincerityTowel", "Kind_Giftbox", "Kind_PinataStick",
    "Kind_NpcOutfit", "Kind_PlayerDemoOutfit", "Kind_Picture", "Kind_Sculpture", "Kind_PictureFake", "Kind_SculptureFake",
    "Kind_SmartPhone", "Kind_DummyFtr", "Kind_SequenceOnly", "Kind_MyDesignObject", "Kind_MyDesignTexture",
    "Kind_CommonFabricRug", "Kind_CommonFabricObject", "Kind_CommonFabricTexture", "Kind_OneRoomBox", "Kind_DummyWrapping",
    "Kind_DummyPresentbox", "Kind_DummyCardboard", "Kind_EventObjFtr", "Kind_NnpcRoomMarker", "Kind_PhotoStudioList",
    "Kind_ShopTorso", "Kind_DummyWrappingOtoshidama", "Kind_GardenEditList",
];
