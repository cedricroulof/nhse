//! Opening, saving, backups and application settings.
use crate::backup::{self, BackupInfo};
use crate::error::{err, CmdResult};
use crate::settings::Settings;
use crate::state::{AppState, Session};
use crate::{sprites, with_save, with_save_mut};
use nhse_core::map::field::FieldLayout;
use nhse_core::save::horizon::InvalidHash;
use nhse_core::HorizonSave;
use serde::Serialize;
use std::path::PathBuf;
use tauri::{Emitter, State};

#[derive(Serialize)]
pub struct AppInfo {
    pub version: String,
    pub languages: Vec<(String, String)>,
    pub settings: Settings,
    pub has_image_pack: bool,
    pub backups_dir: String,
    pub platform: String,
    pub supported_versions: Vec<String>,
}

#[tauri::command]
pub fn app_info(state: State<AppState>) -> CmdResult<AppInfo> {
    let settings = state.settings.lock()?.clone();
    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION").into(),
        languages: nhse_core::data::LANGUAGES.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect(),
        settings,
        has_image_pack: sprites::has_image_pack(&state.paths.images()),
        backups_dir: state.paths.backups().to_string_lossy().into(),
        platform: std::env::consts::OS.into(),
        supported_versions: nhse_core::save::REVISIONS.iter().map(|r| r.version.to_string()).collect(),
    })
}

#[tauri::command]
pub fn set_settings(state: State<AppState>, settings: Settings) -> CmdResult<Settings> {
    let mut s = state.settings.lock()?;
    *s = settings;
    s.save(&state.paths.settings_file());
    Ok(s.clone())
}

#[derive(Serialize)]
pub struct PlayerRef {
    pub index: usize,
    pub dir: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct SaveSummary {
    pub path: String,
    pub is_zip: bool,
    pub town_name: String,
    pub version: String,
    pub revision_index: usize,
    pub compatibility_mode: bool,
    pub warnings: Vec<String>,
    pub players: Vec<PlayerRef>,
    pub dirty: bool,
    pub revision: u64,
    pub last_saved: String,
    pub field: FieldLayout,
    pub pattern_count: usize,
    pub villager_format: u8,
    pub house_format: u8,
    pub backup: Option<String>,
}

fn summary(sess: &Session, backup: Option<String>) -> SaveSummary {
    let s = &sess.save;
    SaveSummary {
        path: s.source.location().to_string_lossy().into(),
        is_zip: s.source.is_zip(),
        town_name: s.town_name(),
        version: s.version().into(),
        revision_index: s.revision.index,
        compatibility_mode: s.revision.compatibility_mode,
        warnings: s.warnings.clone(),
        players: (0..s.players.len()).map(|i| PlayerRef { index: i, dir: s.players[i].dir.clone(), name: s.player_name(i) }).collect(),
        dirty: sess.dirty,
        revision: sess.revision,
        last_saved: s.island_settings().last_saved,
        field: s.field_layout(),
        pattern_count: s.main_ofs.pattern_count,
        villager_format: s.main_ofs.villager_format,
        house_format: s.main_ofs.player_house_format,
        backup,
    }
}

#[tauri::command]
pub fn open_save(state: State<AppState>, path: String) -> CmdResult<SaveSummary> {
    let p = PathBuf::from(&path);
    let save = HorizonSave::open(&p)?;
    let (do_backup, keep) = {
        let st = state.settings.lock()?;
        (st.backup_on_open, st.backup_keep)
    };
    let mut warnings = Vec::new();
    let backup = if do_backup {
        match backup::create(&state.paths.backups(), &save.source, &save.town_name(), keep) {
            Ok(b) => b.map(|x| x.to_string_lossy().to_string()),
            Err(e) => {
                warnings.push(format!("Automatic backup failed: {e}"));
                None
            }
        }
    } else {
        None
    };
    {
        let mut st = state.settings.lock()?;
        st.push_recent(&save.source.location().to_string_lossy());
        st.save(&state.paths.settings_file());
    }
    let mut sess = Session { save, revision: 1, dirty: false };
    sess.save.warnings.extend(warnings);
    let sum = summary(&sess, backup);
    *state.session.lock()? = Some(sess);
    Ok(sum)
}

#[tauri::command]
pub fn save_summary(state: State<AppState>) -> CmdResult<Option<SaveSummary>> {
    let g = state.session.lock()?;
    Ok(g.as_ref().map(|s| summary(s, None)))
}

#[tauri::command]
pub fn close_save(state: State<AppState>) -> CmdResult<()> {
    *state.session.lock()? = None;
    Ok(())
}

#[tauri::command]
pub fn save_changes(state: State<AppState>) -> CmdResult<SaveSummary> {
    let mut g = state.session.lock()?;
    let sess = g.as_mut().ok_or_else(|| crate::error::CmdError("No save is open.".into()))?;
    let seed = (chrono::Local::now().timestamp_nanos_opt().unwrap_or(0) as u64 ^ 0x5DEE_CE66) as u32;
    sess.save.save(seed)?;
    sess.dirty = false;
    Ok(summary(sess, None))
}

/// Reloads the save from disk, discarding unsaved edits.
#[tauri::command]
pub fn revert_save(state: State<AppState>) -> CmdResult<SaveSummary> {
    let path = with_save!(state, |s| s.source.location().to_path_buf());
    let save = HorizonSave::open(&path)?;
    let sess = Session { save, revision: 1, dirty: false };
    let sum = summary(&sess, None);
    *state.session.lock()? = Some(sess);
    Ok(sum)
}

#[tauri::command]
pub fn verify_hashes(state: State<AppState>) -> CmdResult<Vec<InvalidHash>> {
    with_save!(state, |s| Ok(s.invalid_hashes()))
}

#[tauri::command]
pub fn dump_decrypted(state: State<AppState>, dir: String) -> CmdResult<usize> {
    with_save!(state, |s| Ok(s.dump_decrypted(&PathBuf::from(dir))?))
}

#[tauri::command]
pub fn load_decrypted(state: State<AppState>, dir: String) -> CmdResult<usize> {
    with_save_mut!(state, |s| Ok(s.load_decrypted(&PathBuf::from(dir))?))
}

#[tauri::command]
pub fn list_backups(state: State<AppState>) -> CmdResult<Vec<BackupInfo>> {
    Ok(backup::list(&state.paths.backups()))
}

#[tauri::command]
pub fn delete_backup(state: State<AppState>, path: String) -> CmdResult<()> {
    Ok(backup::delete(&state.paths.backups(), &path)?)
}

#[tauri::command]
pub fn create_backup(state: State<AppState>) -> CmdResult<Option<String>> {
    let keep = state.settings.lock()?.backup_keep;
    with_save!(state, |s| Ok(backup::create(&state.paths.backups(), &s.source, &s.town_name(), keep)?
        .map(|p| p.to_string_lossy().to_string())))
}

#[derive(Clone, Serialize)]
struct Progress {
    done: u64,
    total: Option<u64>,
}

#[tauri::command]
pub async fn download_images(app: tauri::AppHandle, state: State<'_, AppState>) -> CmdResult<usize> {
    let dir = state.paths.images();
    let handle = app.clone();
    let res = tauri::async_runtime::spawn_blocking(move || {
        sprites::download_image_pack(&dir, |done, total| {
            let _ = handle.emit("image-progress", Progress { done, total });
        })
    })
    .await?;
    match res {
        Ok(n) => Ok(n),
        Err(e) => err(e),
    }
}

#[tauri::command]
pub fn remove_images(state: State<AppState>) -> CmdResult<()> {
    let d = state.paths.images();
    if d.exists() {
        std::fs::remove_dir_all(&d)?;
    }
    Ok(())
}
