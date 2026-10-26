// Typed wrappers around the Rust commands. Field names mirror the Rust structs (snake_case).
import { invoke } from "@tauri-apps/api/core";

export interface Item { id: number; system: number; additional: number; free: number; }
export interface GDate { year: number; month: number; day: number; }
export interface GDateMD { month: number; day: number; }

export interface Settings {
  language: string; theme: "system" | "light" | "dark"; backup_on_open: boolean; backup_keep: number;
  recent: string[]; map_auto_extensions: boolean; map_prevent_overlap: boolean; room_allow_overhang: boolean; advanced: boolean;
}
export interface AppInfo {
  version: string; languages: [string, string][]; settings: Settings; has_image_pack: boolean;
  backups_dir: string; platform: string; supported_versions: string[];
}
export interface FieldLayout { acres_w: number; acres_h: number; width: number; height: number; offset_x: number; offset_y: number; }
export interface PlayerRef { index: number; dir: string; name: string; }
export interface SaveSummary {
  path: string; is_zip: boolean; town_name: string; version: string; revision_index: number; compatibility_mode: boolean;
  warnings: string[]; players: PlayerRef[]; dirty: boolean; revision: number; last_saved: string; field: FieldLayout;
  pattern_count: number; villager_format: number; house_format: number; backup: string | null;
}
export interface InvalidHash { file: string; region: string; }
export interface BackupInfo { name: string; path: string; size: number; modified: string; }

export interface FieldItemInfo { id: number; name: string; kind: number; kind_name: string; held: number; color: number; }
export interface VillagerInfo { internal: string; name: string; species: number; variant: number; phrase: string; has_template: boolean; }
export interface FlagInfo { index: number; name: string; label: string; default: number; max: number; desc: string; }
export interface AchievementInfo { index: number; name: string; label: string; tiers: number; thresholds: number[]; }
export interface GameData {
  lang: string; items: string[]; named: boolean[]; kinds: number[]; kind_names: string[]; colors: number[]; sizes: number[];
  field_items: FieldItemInfo[]; villagers: VillagerInfo[]; enums: Record<string, [string, number][]>; recipes: [number, number][];
  achievements: AchievementInfo[]; flags_player: FlagInfo[]; flags_land: FlagInfo[]; flags_house: FlagInfo[];
  flags_villager: FlagInfo[]; flags_memory: FlagInfo[]; lists: Record<string, number[]>; mail: [string, string][];
  stationery: number[]; npc_senders: [number, string, string][]; box_skins: [number, string][]; mail_languages: string[];
  special_kinds: Record<string, number>;
}
export interface Variant { index: number; name: string; colors: string; }
export interface ItemInfo {
  id: number; name: string; kind: number; kind_name: string; max_stack: number | null; width: number; height: number;
  is_flower_gene: boolean; is_recipe: boolean; is_fossil_unknown: boolean; is_tree: boolean; is_field_item: boolean;
  body_label: string | null; body: Variant[]; fabric_label: string | null; fabric: Variant[]; fabric_customizable: boolean;
}

export interface PlayerProfile {
  index: number; dir: string; name: string; town_name: string; player_id: number; town_id: number;
  wallet: number; bank: number; nook_miles: number; total_nook_miles: number;
  pocket_count: number; bag_count: number; storage_count: number; storage_capacity: number;
  hotel_tickets: number | null; poki: number | null; birthday: GDateMD; profile_birthday: GDateMD; profile_fruit: number;
  profile_timestamp: GDate; profile_made_village: boolean; has_photo: boolean; invalid_values: string[];
}
export type PlayerList = "pocket" | "storage";
export interface Reactions { learned: number[]; wheel: number[]; new_flags: boolean[]; }
export interface Achievements { counts: number[]; read: boolean[][]; dates: GDate[][]; flags: number[]; }
export interface Received { items: number[]; remake: number[]; remake_items: number[]; }

export interface PlayerId { town_id: number; town_name: string; island_ruby_type: number; player_id: number; player_name: string; gender: number; }
export interface Mail {
  slot: number; status: number; favorite: boolean; sender_type: number; sender_island_id: number; sender_island_name: string;
  sender_species: number; sender_variant: number; sender_id: number; sender_name: string; receiver_island_id: number;
  receiver_island_name: string; receiver_player_id: number; receiver_player_name: string; date: GDate; item: Item;
  wrapping: number; is_letter: boolean; to_line: string; message: string; from_line: string; stationery: number;
  sort_index: number; language: string;
}
export interface MailBox { mails: Mail[]; next_id: number; }

export interface IslandSettings {
  hemisphere: number; tour_hemisphere: number; airport_color: number; weather_seed: number; tour_weather_seed: number;
  specialty_fruit: number; sister_fruit: number; specialty_flower: number; sister_flower: number; fruit_flags: number[];
  last_saved: string; camp_status: number; camp_species: number; camp_variant: number; camp_last_visit: GDate;
}
export interface RoomFloorWall {
  accent_wall: Item; wall: Item; floor: Item; accent_wall_design: number; wall_design: number; floor_design: number;
  accent_wall_direction: number; info_bit: number; floor_direction: number;
}
export interface Memory { player: PlayerId; friendship: number; nickname: string; greetings: string[]; greeting_set_date: GDate; flags: number[]; }
export interface Villager {
  index: number; format: number; species: number; variant: number; internal_name: string; personality: number; catchphrase: string;
  birth_type: number; inducement_type: number; move_type: number; moving_out: boolean; diy_end: number[]; diy_recipe: number;
  room: RoomFloorWall; wear: Item[]; furniture: Item[]; memories: Memory[]; event_flags: number[];
}
export interface HouseExterior { level: number; wall: number; roof: number; door: number; order_wall: number; order_roof: number; order_door: number; door_deco: Item; }
export interface VillagerHouse { index: number; format: number; exterior: HouseExterior; status: number; npc1: number; npc2: number; build_player: number; room: RoomFloorWall | null; }
export interface PlayerHouse { index: number; format: number; exterior: HouseExterior; loan: number; npc1: number; npc2: number; player_house_flag: boolean; mailbox: Item; order_mailbox: Item; event_flags: number[]; }
export interface PlayerRoom { layers: Item[][]; surfaces: RoomFloorWall; music: number; music_shuffle: boolean; ambient: number | null; }
export type DesignKind = "normal" | "pro" | "flag" | "tailor";
export interface DesignInfo { index: number; name: string; town_name: string; player_name: string; town_id: number; player_id: number; usage: number; pro: boolean; empty: boolean; }
export interface DesignPixels { info: DesignInfo; palette: [number, number, number][]; pixels: number[]; }
export interface Turnips { buy_price: number; sell: number[]; pattern: number; fever_start: number; }
export interface Museum { level: number; dates: GDate[]; items: Item[]; players: number[]; }
export interface Visitors {
  schedule: number[]; create_pos: number[]; pos2: number[]; is_normal_live: boolean[]; is_birthday_live: boolean[];
  init_schedule: number[]; flags_14: boolean[]; day_wisp: number; day_celeste: number;
}
export interface Post { slot: number; date: GDate; body: string; footer: string; author: PlayerId; design_id: number; unique_id: number; has_handwriting: boolean; index_table: number; }
export interface Board { built: GDate; latest_unique_id: number; posts: Post[]; }
export interface Building { kind: number; x: number; y: number; angle: number; bit: number; type_: number; type_arg: number; unique_id: number; unused: number; }
export interface Plaza { x: number; z: number; }
export interface MapMeta { acres: number[]; plaza: Plaza; design_tiles: number[]; design_dims: [number, number]; terrain_dims: [number, number]; }
export interface Rect { x: number; y: number; w: number; h: number; }
export type ClearKind = "weeds" | "trees" | "flowers" | "bushes" | "plants" | "fences" | "stones" | "holes" | "field_objects" | "placed_items" | "shells" | "branches" | "everything";
export type Arrangement = "square" | "horizontal" | "vertical";
export type SpawnSource =
  | { type: "item"; item: Item; count: number }
  | { type: "diy"; min: number; max: number; alphabetical: boolean; count: number }
  | { type: "list"; items: Item[]; dropped: boolean };
export type DumpKind = "field_layer" | "field_acre" | "field_flags" | "terrain" | "terrain_acre" | "buildings" | "acres" | "designs";

const call = <T,>(cmd: string, args?: Record<string, unknown>) => invoke<T>(cmd, args);

export const api = {
  appInfo: () => call<AppInfo>("app_info"),
  setSettings: (settings: Settings) => call<Settings>("set_settings", { settings }),
  openSave: (path: string) => call<SaveSummary>("open_save", { path }),
  saveSummary: () => call<SaveSummary | null>("save_summary"),
  closeSave: () => call<void>("close_save"),
  saveChanges: () => call<SaveSummary>("save_changes"),
  revertSave: () => call<SaveSummary>("revert_save"),
  verifyHashes: () => call<InvalidHash[]>("verify_hashes"),
  dumpDecrypted: (dir: string) => call<number>("dump_decrypted", { dir }),
  loadDecrypted: (dir: string) => call<number>("load_decrypted", { dir }),
  listBackups: () => call<BackupInfo[]>("list_backups"),
  deleteBackup: (path: string) => call<void>("delete_backup", { path }),
  createBackup: () => call<string | null>("create_backup"),
  downloadImages: () => call<number>("download_images"),
  removeImages: () => call<void>("remove_images"),

  gameData: (lang?: string) => call<GameData>("game_data", { lang }),
  itemInfo: (id: number) => call<ItemInfo>("item_info", { id }),
  itemNames: (items: Item[]) => call<string[]>("item_names", { items }),
  parseItems: (text: string) => call<[Item[], string[]]>("parse_items", { text }),
  itemsToCheat: (items: Item[], offset: number) => call<string>("items_to_cheat", { items, offset }),

  playerProfile: (player: number) => call<PlayerProfile>("player_profile", { player }),
  setPlayerProfile: (player: number, profile: PlayerProfile) => call<PlayerProfile>("set_player_profile", { player, profile }),
  playerPhoto: (player: number) => call<string>("player_photo", { player }),
  exportPlayerPhoto: (player: number, path: string) => call<void>("export_player_photo", { player, path }),
  playerItems: (player: number, list: PlayerList) => call<Item[]>("player_items", { player, list }),
  setPlayerItems: (player: number, list: PlayerList, items: Item[]) => call<void>("set_player_items", { player, list, items }),
  playerRecipes: (player: number) => call<number[]>("player_recipes", { player }),
  setPlayerRecipes: (player: number, flags: number[]) => call<void>("set_player_recipes", { player, flags }),
  playerReactions: (player: number) => call<Reactions>("player_reactions", { player }),
  setPlayerReactions: (player: number, reactions: Reactions) => call<void>("set_player_reactions", { player, reactions }),
  playerAchievements: (player: number) => call<Achievements>("player_achievements", { player }),
  setPlayerAchievements: (player: number, achievements: Achievements) => call<void>("set_player_achievements", { player, achievements }),
  completeAchievements: (player: number, indices?: number[]) => call<Achievements>("complete_achievements", { player, indices }),
  playerFlags: (player: number) => call<number[]>("player_flags", { player }),
  setPlayerFlags: (player: number, flags: number[]) => call<void>("set_player_flags", { player, flags }),
  playerReceived: (player: number) => call<Received>("player_received", { player }),
  setPlayerReceived: (player: number, items: number[], remake: number[]) => call<void>("set_player_received", { player, items, remake }),
  playerMail: (player: number) => call<MailBox>("player_mail", { player }),
  setPlayerMail: (player: number, mail: Mail) => call<MailBox>("set_player_mail", { player, mail }),
  clearPlayerMail: (player: number, slot: number) => call<void>("clear_player_mail", { player, slot }),
  exportPlayerMail: (player: number, slot: number, path: string) => call<void>("export_player_mail", { player, slot, path }),
  importPlayerMail: (player: number, slot: number, path: string) => call<void>("import_player_mail", { player, slot, path }),
  replaceInMail: (player: number, find: string, replace: string, names: boolean, text: boolean) => call<number>("replace_in_mail", { player, find, replace, names, text }),
  exportItemsFile: (items: Item[], path: string) => call<void>("export_items_file", { items, path }),
  importItemsFile: (path: string) => call<Item[]>("import_items_file", { path }),

  islandSettings: () => call<IslandSettings>("island_settings"),
  setIslandSettings: (settings: IslandSettings) => call<void>("set_island_settings", { settings }),
  villagers: () => call<Villager[]>("villagers"),
  setVillager: (villager: Villager) => call<Villager[]>("set_villager", { villager }),
  villagerTemplates: () => call<string[]>("villager_templates"),
  replaceVillager: (index: number, internal: string, moveOut: boolean) => call<Villager[]>("replace_villager", { index, internal, moveOut }),
  exportVillager: (index: number, path: string) => call<void>("export_villager", { index, path }),
  importVillager: (index: number, path: string, adopt: boolean) => call<{ foreign: boolean; owner: string }>("import_villager", { index, path, adopt }),
  maxFriendshipAll: () => call<Villager[]>("max_friendship_all"),
  villagerHouses: () => call<VillagerHouse[]>("villager_houses"),
  setVillagerHouse: (house: VillagerHouse) => call<void>("set_villager_house", { house }),
  exportVillagerHouse: (index: number, path: string) => call<void>("export_villager_house", { index, path }),
  importVillagerHouse: (index: number, path: string) => call<void>("import_villager_house", { index, path }),
  playerHouses: () => call<PlayerHouse[]>("player_houses"),
  setPlayerHouse: (house: PlayerHouse) => call<void>("set_player_house", { house }),
  playerRoom: (house: number, room: number) => call<PlayerRoom>("player_room", { house, room }),
  setPlayerRoom: (house: number, room: number, data: PlayerRoom) => call<void>("set_player_room", { house, room, data }),
  exportPlayerHouse: (index: number, path: string) => call<void>("export_player_house", { index, path }),
  importPlayerHouse: (index: number, path: string) => call<void>("import_player_house", { index, path }),
  exportPlayerRoom: (house: number, room: number, path: string) => call<void>("export_player_room", { house, room, path }),
  importPlayerRoom: (house: number, room: number, path: string) => call<void>("import_player_room", { house, room, path }),
  designs: (kind: DesignKind) => call<DesignInfo[]>("designs", { kind }),
  designPixels: (kind: DesignKind, index: number) => call<DesignPixels>("design_pixels", { kind, index }),
  setDesignPixels: (kind: DesignKind, index: number, name: string, palette: number[][], pixels: number[]) => call<void>("set_design_pixels", { kind, index, name, palette, pixels }),
  designPng: (kind: DesignKind, index: number, path: string, scale?: number) => call<void>("design_png", { kind, index, path, scale }),
  importDesignPng: (kind: DesignKind, index: number, path: string, name?: string) => call<void>("import_design_png", { kind, index, path, name }),
  exportDesign: (kind: DesignKind, index: number, path: string) => call<void>("export_design", { kind, index, path }),
  importDesign: (kind: DesignKind, index: number, path: string, adopt: boolean) => call<void>("import_design", { kind, index, path, adopt }),
  setDesignAuthorAll: (kind: DesignKind, player: number) => call<number>("set_design_author_all", { kind, player }),
  recycleBin: () => call<Item[]>("recycle_bin"),
  setRecycleBin: (items: Item[]) => call<void>("set_recycle_bin", { items }),
  buildings: () => call<Building[]>("buildings"),
  setBuildings: (buildings: Building[]) => call<void>("set_buildings", { buildings }),
  landFlags: () => call<number[]>("land_flags"),
  setLandFlags: (flags: number[]) => call<void>("set_land_flags", { flags }),
  turnips: () => call<Turnips>("turnips"),
  setTurnips: (turnips: Turnips) => call<void>("set_turnips", { turnips }),
  museum: () => call<Museum>("museum"),
  setMuseum: (museum: Museum) => call<void>("set_museum", { museum }),
  museumDonateAll: (categories: string[], player: number, perDay: number) => call<[Museum, number]>("museum_donate_all", { categories, player, perDay }),
  museumSort: () => call<Museum>("museum_sort"),
  visitors: () => call<Visitors>("visitors"),
  setVisitors: (visitors: Visitors) => call<void>("set_visitors", { visitors }),
  bulletin: () => call<Board>("bulletin"),
  setBulletin: (board: Board) => call<void>("set_bulletin", { board }),
  exportBulletinPost: (slot: number, path: string) => call<void>("export_bulletin_post", { slot, path }),
  importBulletinPost: (slot: number, path: string) => call<void>("import_bulletin_post", { slot, path }),
  exportBulletinText: (path: string) => call<number>("export_bulletin_text", { path }),
  today: () => call<GDate>("today"),

  fieldLayer: async (layer: number) => new Uint8Array(await call<ArrayBuffer>("field_layer", { layer })),
  setFieldLayer: (layer: number, bytes: Uint8Array) => invoke<void>("set_field_layer", bytes, { headers: { "x-layer": String(layer) } }),
  terrain: async () => new Uint8Array(await call<ArrayBuffer>("terrain")),
  setTerrain: (bytes: Uint8Array) => invoke<void>("set_terrain", bytes),
  terrainPng: async (px?: number) => new Uint8Array(await call<ArrayBuffer>("terrain_png", { px })),
  mapMeta: () => call<MapMeta>("map_meta"),
  setAcres: (acres: number[]) => call<void>("set_acres", { acres }),
  setPlaza: (plaza: Plaza) => call<void>("set_plaza", { plaza }),
  setDesignTiles: (tiles: number[]) => call<void>("set_design_tiles", { tiles }),
  fieldClear: (layer: number, kind: ClearKind, rect?: Rect) => call<number>("field_clear", { layer, kind, rect }),
  fieldWater: (layer: number, all: boolean, rect?: Rect) => call<number>("field_water", { layer, all, rect }),
  fieldSpawn: (layer: number, x: number, y: number, arrange: Arrangement, source: SpawnSource, overwrite: boolean) => call<number>("field_spawn", { layer, x, y, arrange, source, overwrite }),
  fieldReplace: (layer: number, oldItem: Item, newItem: Item, idOnly: boolean, rect?: Rect) => call<number>("field_replace", { layer, old: oldItem, new: newItem, idOnly, rect }),
  fieldCleanExtensions: (layer: number) => call<number>("field_clean_extensions", { layer }),
  fieldUnsupported: () => call<[number, number][]>("field_unsupported"),
  mapExport: (kind: DumpKind, path: string, layer?: number, acreX?: number, acreY?: number) => call<void>("map_export", { kind, path, layer, acreX, acreY }),
  mapImport: (kind: DumpKind, path: string, layer?: number, acreX?: number, acreY?: number) => call<void>("map_import", { kind, path, layer, acreX, acreY }),
};

export const NONE: Item = { id: 0xfffe, system: 0, additional: 0, free: 0 };
export const isNone = (i: Item) => i.id === 0xfffe;
export const isExtension = (i: Item) => i.id === 0xfffd;

export const itemSet = (category: string, variants: boolean) => invoke<Item[]>("item_set", { category, variants });
