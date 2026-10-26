//! Persistent user preferences (stored as JSON in the app config directory).
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Language of game data (item / villager names).
    pub language: String,
    /// "system" | "light" | "dark"
    pub theme: String,
    /// Back up the save files (only the save files) when a save is opened.
    pub backup_on_open: bool,
    /// Number of automatic backups kept per island (older ones are pruned).
    pub backup_keep: usize,
    /// Recently opened saves (most recent first).
    pub recent: Vec<String>,
    /// Map editor: automatically create/delete extension tiles for multi-tile items.
    pub map_auto_extensions: bool,
    /// Map editor: refuse to overwrite occupied tiles.
    pub map_prevent_overlap: bool,
    /// Allow placing room items partially outside the room grid.
    pub room_allow_overhang: bool,
    /// Show hex values and other advanced controls.
    pub advanced: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "en".into(),
            theme: "system".into(),
            backup_on_open: true,
            backup_keep: 25,
            recent: Vec::new(),
            map_auto_extensions: true,
            map_prevent_overlap: true,
            room_allow_overhang: false,
            advanced: false,
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) {
        if let Some(p) = path.parent() {
            let _ = std::fs::create_dir_all(p);
        }
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, s);
        }
    }

    pub fn push_recent(&mut self, p: &str) {
        self.recent.retain(|x| x != p);
        self.recent.insert(0, p.to_string());
        self.recent.truncate(10);
    }
}

/// Well-known locations used by the app.
#[derive(Debug, Clone)]
pub struct Paths {
    pub config: PathBuf,
    pub data: PathBuf,
}

impl Paths {
    pub fn settings_file(&self) -> PathBuf {
        self.config.join("settings.json")
    }
    pub fn backups(&self) -> PathBuf {
        self.data.join("backups")
    }
    pub fn images(&self) -> PathBuf {
        self.data.join("img")
    }
}
