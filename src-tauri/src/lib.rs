//! NHSE desktop application (Tauri).
mod backup;
mod cmd;
mod error;
mod settings;
mod sprites;
mod state;

use nhse_core::map::render::encode_png;
use nhse_core::save::island::DesignKind;
use nhse_core::structs::design;
use settings::Paths;
use state::AppState;
use tauri::http::{Response, StatusCode};
use tauri::Manager;

fn png(bytes: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/png")
        .header("Cache-Control", "max-age=31536000, immutable")
        .header("Access-Control-Allow-Origin", "*")
        .body(bytes)
        .unwrap()
}

fn not_found() -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Access-Control-Allow-Origin", "*")
        .body(Vec::new())
        .unwrap()
}

/// `nhse://localhost/<kind>/<arg>?…` image endpoint used by `<img>` tags in the UI.
fn serve(app: &tauri::AppHandle, uri: &tauri::http::Uri) -> Response<Vec<u8>> {
    let state = app.state::<AppState>();
    let path = uri.path().trim_start_matches('/');
    let query: std::collections::HashMap<String, String> = uri
        .query()
        .unwrap_or("")
        .split('&')
        .filter_map(|kv| kv.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let mut parts = path.split('/');
    match (parts.next(), parts.next(), parts.next()) {
        (Some("item"), Some(id), _) => {
            let Ok(id) = id.trim_end_matches(".png").parse::<u16>() else {
                return not_found();
            };
            let variant = query.get("v").and_then(|v| v.parse().ok()).unwrap_or(0);
            png(sprites::item_png(&state.paths.images(), id, variant))
        }
        (Some("icon"), Some(id), _) => {
            let Ok(id) = id.trim_end_matches(".png").parse::<u16>() else {
                return not_found();
            };
            png(sprites::menu_icon_png(id))
        }
        (Some("villager"), Some(name), _) => {
            match sprites::villager_png(name.trim_end_matches(".png")) {
                Some(b) => png(b),
                None => sprites::embedded("Misc/leaf.png")
                    .map(png)
                    .unwrap_or_else(not_found),
            }
        }
        (Some("design"), Some(kind), Some(index)) => {
            let kind = match kind {
                "normal" => DesignKind::Normal,
                "pro" => DesignKind::Pro,
                "flag" => DesignKind::Flag,
                "tailor" => DesignKind::Tailor,
                _ => return not_found(),
            };
            let Ok(index) = index.trim_end_matches(".png").parse::<usize>() else {
                return not_found();
            };
            let sheet: Option<usize> = query.get("sheet").and_then(|v| v.parse().ok());
            let Ok(guard) = state.session.lock() else {
                return not_found();
            };
            let Some(sess) = guard.as_ref() else {
                return not_found();
            };
            let Ok(d) = sess.save.design_raw(kind, index) else {
                return not_found();
            };
            let pro = matches!(kind, DesignKind::Pro | DesignKind::Tailor);
            if !pro || sheet.is_some() {
                return png(encode_png(
                    32,
                    32,
                    &design::render_rgba(d, sheet.unwrap_or(0)),
                ));
            }
            // PRO overview: 2×2 sheets (front, back, right, left)
            let mut out = vec![0u8; 64 * 64 * 4];
            for s in 0..4 {
                let rgba = design::render_rgba(d, s);
                let (ox, oy) = ((s % 2) * 32, (s / 2) * 32);
                for y in 0..32 {
                    let di = ((oy + y) * 64 + ox) * 4;
                    out[di..di + 128].copy_from_slice(&rgba[y * 128..y * 128 + 128]);
                }
            }
            png(encode_png(64, 64, &out))
        }
        (Some("handwriting"), Some(slot), _) => {
            let Ok(slot) = slot.trim_end_matches(".png").parse::<usize>() else {
                return not_found();
            };
            let Ok(guard) = state.session.lock() else {
                return not_found();
            };
            let Some(sess) = guard.as_ref() else {
                return not_found();
            };
            let rgba = sess.save.bulletin_handwriting(slot);
            png(encode_png(
                nhse_core::structs::bbs::HANDWRITING_W,
                nhse_core::structs::bbs::HANDWRITING_H,
                &rgba,
            ))
        }
        _ => not_found(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .register_uri_scheme_protocol("nhse", |ctx, request| {
            serve(ctx.app_handle(), request.uri())
        })
        .setup(|app| {
            let paths = Paths {
                config: app.path().app_config_dir()?,
                data: app.path().app_data_dir()?,
            };
            let _ = std::fs::create_dir_all(&paths.config);
            let _ = std::fs::create_dir_all(&paths.data);
            app.manage(AppState::new(paths));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd::session::app_info,
            cmd::session::set_settings,
            cmd::session::open_save,
            cmd::session::save_summary,
            cmd::session::close_save,
            cmd::session::save_changes,
            cmd::session::revert_save,
            cmd::session::verify_hashes,
            cmd::session::dump_decrypted,
            cmd::session::load_decrypted,
            cmd::session::list_backups,
            cmd::session::delete_backup,
            cmd::session::create_backup,
            cmd::session::download_images,
            cmd::session::remove_images,
            cmd::data::game_data,
            cmd::data::item_info,
            cmd::data::item_names,
            cmd::data::parse_items,
            cmd::data::items_to_cheat,
            cmd::data::item_set,
            cmd::player::player_profile,
            cmd::player::set_player_profile,
            cmd::player::player_photo,
            cmd::player::export_player_photo,
            cmd::player::player_items,
            cmd::player::set_player_items,
            cmd::player::player_recipes,
            cmd::player::set_player_recipes,
            cmd::player::player_reactions,
            cmd::player::set_player_reactions,
            cmd::player::player_achievements,
            cmd::player::set_player_achievements,
            cmd::player::complete_achievements,
            cmd::player::player_flags,
            cmd::player::set_player_flags,
            cmd::player::player_received,
            cmd::player::set_player_received,
            cmd::player::player_mail,
            cmd::player::set_player_mail,
            cmd::player::clear_player_mail,
            cmd::player::export_player_mail,
            cmd::player::import_player_mail,
            cmd::player::replace_in_mail,
            cmd::player::export_items_file,
            cmd::player::import_items_file,
            cmd::island::island_settings,
            cmd::island::set_island_settings,
            cmd::island::villagers,
            cmd::island::set_villager,
            cmd::island::villager_templates,
            cmd::island::replace_villager,
            cmd::island::export_villager,
            cmd::island::import_villager,
            cmd::island::max_friendship_all,
            cmd::island::villager_houses,
            cmd::island::set_villager_house,
            cmd::island::export_villager_house,
            cmd::island::import_villager_house,
            cmd::island::player_houses,
            cmd::island::set_player_house,
            cmd::island::player_room,
            cmd::island::set_player_room,
            cmd::island::export_player_house,
            cmd::island::import_player_house,
            cmd::island::export_player_room,
            cmd::island::import_player_room,
            cmd::island::designs,
            cmd::island::design_pixels,
            cmd::island::set_design_pixels,
            cmd::island::design_png,
            cmd::island::import_design_png,
            cmd::island::export_design,
            cmd::island::import_design,
            cmd::island::set_design_author_all,
            cmd::island::recycle_bin,
            cmd::island::set_recycle_bin,
            cmd::island::buildings,
            cmd::island::set_buildings,
            cmd::island::land_flags,
            cmd::island::set_land_flags,
            cmd::island::turnips,
            cmd::island::set_turnips,
            cmd::island::museum,
            cmd::island::set_museum,
            cmd::island::museum_donate_all,
            cmd::island::museum_sort,
            cmd::island::visitors,
            cmd::island::set_visitors,
            cmd::island::bulletin,
            cmd::island::set_bulletin,
            cmd::island::export_bulletin_post,
            cmd::island::import_bulletin_post,
            cmd::island::export_bulletin_text,
            cmd::island::today,
            cmd::map::field_layer,
            cmd::map::set_field_layer,
            cmd::map::terrain,
            cmd::map::set_terrain,
            cmd::map::terrain_png,
            cmd::map::map_meta,
            cmd::map::set_acres,
            cmd::map::set_plaza,
            cmd::map::set_design_tiles,
            cmd::map::field_clear,
            cmd::map::field_water,
            cmd::map::field_spawn,
            cmd::map::field_replace,
            cmd::map::field_clean_extensions,
            cmd::map::field_unsupported,
            cmd::map::map_export,
            cmd::map::map_import,
        ])
        .run(tauri::generate_context!())
        .expect("error while running NHSE");
}
