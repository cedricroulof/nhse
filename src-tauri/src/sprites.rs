//! Embedded sprite images (menu icons, villager portraits) and the downloadable item image pack.
use nhse_core::data::items::{item_count, item_menu_icon};
use nhse_core::data::tables::{enum_name, tables};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

static SPRITES: &[u8] = include_bytes!("../resources/sprites.zip");

fn cache() -> &'static Mutex<HashMap<String, Option<Vec<u8>>>> {
    static C: OnceLock<Mutex<HashMap<String, Option<Vec<u8>>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Reads an embedded sprite (e.g. `MenuIcon/Fish.png`).
pub fn embedded(name: &str) -> Option<Vec<u8>> {
    if let Some(v) = cache().lock().ok()?.get(name) {
        return v.clone();
    }
    let data = (|| {
        let mut z = zip::ZipArchive::new(std::io::Cursor::new(SPRITES)).ok()?;
        let mut f = z.by_name(name).ok()?;
        let mut b = Vec::new();
        f.read_to_end(&mut b).ok()?;
        Some(b)
    })();
    cache().lock().ok()?.insert(name.to_string(), data.clone());
    data
}

/// Field items (60000+) borrow the image of what they turn into when picked up.
fn sprite_id(id: u16) -> u16 {
    if (id as usize) < item_count() {
        return id;
    }
    match tables().field_items.get(&id) {
        Some(f) if (f.held_item_id() as usize) < item_count() => f.held_item_id(),
        _ => id,
    }
}

/// PNG for an item: the downloaded image pack if present, else the menu icon, else a leaf.
pub fn item_png(img_dir: &Path, id: u16, variant: u16) -> Vec<u8> {
    let sid = sprite_id(id);
    for name in [format!("{sid:05}_{variant}.png"), format!("{sid:05}_0.png")] {
        if let Ok(b) = std::fs::read(img_dir.join(&name)) {
            return b;
        }
    }
    menu_icon_png(sid)
}

pub fn menu_icon_png(id: u16) -> Vec<u8> {
    let icon = item_menu_icon(id);
    let name = enum_name("ItemMenuIconType", icon as i64).unwrap_or("Leaf");
    let file = if name == "Leaf" { "Leaf1".to_string() } else { name.to_string() };
    embedded(&format!("MenuIcon/{file}.png")).or_else(|| embedded("Misc/leaf.png")).unwrap_or_default()
}

pub fn villager_png(internal: &str) -> Option<Vec<u8>> {
    embedded(&format!("Villagers/{internal}.png"))
}

pub fn has_image_pack(img_dir: &Path) -> bool {
    std::fs::read_dir(img_dir).map(|mut d| d.next().is_some()).unwrap_or(false)
}

pub const IMAGE_PACK_URL: &str = "https://github.com/berichan/NHSE-USB/releases/download/0.01/raw.zip";

/// Downloads and extracts the community item image pack into `img_dir`.
pub fn download_image_pack(img_dir: &Path, progress: impl Fn(u64, Option<u64>)) -> Result<usize, String> {
    let resp = ureq::get(IMAGE_PACK_URL).call().map_err(|e| format!("download failed: {e}"))?;
    let total = resp.header("Content-Length").and_then(|v| v.parse::<u64>().ok());
    let mut reader = resp.into_reader();
    let mut buf = Vec::with_capacity(total.unwrap_or(0) as usize);
    let mut chunk = [0u8; 1 << 16];
    loop {
        let n = reader.read(&mut chunk).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        progress(buf.len() as u64, total);
    }
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(buf)).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(img_dir).map_err(|e| e.to_string())?;
    let mut count = 0;
    for i in 0..z.len() {
        let mut f = z.by_index(i).map_err(|e| e.to_string())?;
        if f.is_dir() {
            continue;
        }
        let Some(name) = Path::new(f.name()).file_name().map(|n| n.to_owned()) else { continue };
        if !name.to_string_lossy().ends_with(".png") {
            continue;
        }
        let mut out = Vec::new();
        f.read_to_end(&mut out).map_err(|e| e.to_string())?;
        std::fs::write(img_dir.join(name), out).map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}
