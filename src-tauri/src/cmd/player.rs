//! Player (`personal.dat`, `postbox.dat`) commands.
use crate::error::{err, CmdResult};
use crate::state::AppState;
use crate::{with_save, with_save_mut};
use base64::Engine;
use nhse_core::save::player::{PlayerList, PlayerProfile};
use nhse_core::structs::mail::Mail;
use nhse_core::structs::misc::{Achievements, GDate, Reactions};
use nhse_core::Item;
use serde::{Deserialize, Serialize};
use tauri::State;

#[tauri::command]
pub fn player_profile(state: State<AppState>, player: usize) -> CmdResult<PlayerProfile> {
    with_save!(state, |s| Ok(s.player_profile(player)?))
}

#[tauri::command]
pub fn set_player_profile(
    state: State<AppState>,
    player: usize,
    profile: PlayerProfile,
) -> CmdResult<PlayerProfile> {
    with_save_mut!(state, |s| {
        s.set_player_profile(player, &profile)?;
        Ok(s.player_profile(player)?)
    })
}

/// Profile photo as a `data:` URL (empty if none).
#[tauri::command]
pub fn player_photo(state: State<AppState>, player: usize) -> CmdResult<String> {
    with_save!(state, |s| {
        let jpg = s.player_photo(player);
        if jpg.is_empty() {
            return Ok(String::new());
        }
        Ok(format!(
            "data:image/jpeg;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(jpg)
        ))
    })
}

#[tauri::command]
pub fn export_player_photo(state: State<AppState>, player: usize, path: String) -> CmdResult<()> {
    with_save!(state, |s| {
        let jpg = s.player_photo(player);
        if jpg.is_empty() {
            return err("This player has no profile photo.");
        }
        std::fs::write(path, jpg)?;
        Ok(())
    })
}

#[tauri::command]
pub fn player_items(
    state: State<AppState>,
    player: usize,
    list: PlayerList,
) -> CmdResult<Vec<Item>> {
    with_save!(state, |s| Ok(s.player_items(player, list)?))
}

#[tauri::command]
pub fn set_player_items(
    state: State<AppState>,
    player: usize,
    list: PlayerList,
    items: Vec<Item>,
) -> CmdResult<()> {
    with_save_mut!(state, |s| Ok(s.set_player_items(player, list, &items)?))
}

/// Recipe flags as a byte per recipe: bit0 known, bit1 made, bit2 new, bit3 favorite.
#[tauri::command]
pub fn player_recipes(state: State<AppState>, player: usize) -> CmdResult<Vec<u8>> {
    with_save!(state, |s| Ok(s
        .recipes(player)?
        .iter()
        .map(|f| f
            .iter()
            .enumerate()
            .fold(0u8, |a, (i, b)| a | ((*b as u8) << i)))
        .collect()))
}

#[tauri::command]
pub fn set_player_recipes(state: State<AppState>, player: usize, flags: Vec<u8>) -> CmdResult<()> {
    let f: Vec<[bool; 4]> = flags
        .iter()
        .map(|b| std::array::from_fn(|i| b & (1 << i) != 0))
        .collect();
    with_save_mut!(state, |s| Ok(s.set_recipes(player, &f)?))
}

#[tauri::command]
pub fn player_reactions(state: State<AppState>, player: usize) -> CmdResult<Reactions> {
    with_save!(state, |s| Ok(s.reactions(player)?))
}

#[tauri::command]
pub fn set_player_reactions(
    state: State<AppState>,
    player: usize,
    reactions: Reactions,
) -> CmdResult<()> {
    with_save_mut!(state, |s| Ok(s.set_reactions(player, &reactions)?))
}

#[tauri::command]
pub fn player_achievements(state: State<AppState>, player: usize) -> CmdResult<Achievements> {
    with_save!(state, |s| Ok(s.achievements(player)?))
}

#[tauri::command]
pub fn set_player_achievements(
    state: State<AppState>,
    player: usize,
    achievements: Achievements,
) -> CmdResult<()> {
    with_save_mut!(state, |s| Ok(s.set_achievements(player, &achievements)?))
}

/// Completes every achievement tier (dates staggered from today).
#[tauri::command]
pub fn complete_achievements(
    state: State<AppState>,
    player: usize,
    indices: Option<Vec<usize>>,
) -> CmdResult<Achievements> {
    with_save_mut!(state, |s| {
        let mut a = s.achievements(player)?;
        let list: Vec<usize> = indices.unwrap_or_else(|| (0..Achievements::MAX).collect());
        let today = GDate::today();
        for i in list {
            if i < Achievements::MAX {
                a.complete(i, today);
            }
        }
        s.set_achievements(player, &a)?;
        Ok(a)
    })
}

#[tauri::command]
pub fn player_flags(state: State<AppState>, player: usize) -> CmdResult<Vec<i16>> {
    with_save!(state, |s| Ok(s.player_flags(player)?))
}

#[tauri::command]
pub fn set_player_flags(state: State<AppState>, player: usize, flags: Vec<i16>) -> CmdResult<()> {
    with_save_mut!(state, |s| Ok(s.set_player_flags(player, &flags)?))
}

#[derive(Serialize, Deserialize)]
pub struct Received {
    /// One byte (0/1) per item id.
    pub items: Vec<u8>,
    /// One byte (0/1) per remake bit (remake index × 8 + variant).
    pub remake: Vec<u8>,
    /// For each remake index: the item id it belongs to (0 when unknown).
    pub remake_items: Vec<u16>,
}

#[tauri::command]
pub fn player_received(state: State<AppState>, player: usize) -> CmdResult<Received> {
    with_save!(state, |s| {
        let (items, remake) = s.received_items(player)?;
        let t = nhse_core::data::tables();
        let mut remake_items = vec![0u16; remake.len() / 8 + 1];
        let mut sorted: Vec<(u16, i16)> = t.remake_map.iter().map(|(a, b)| (*a, *b)).collect();
        sorted.sort();
        for (item, idx) in sorted {
            if let Some(slot) = remake_items.get_mut(idx as usize) {
                if *slot == 0 {
                    *slot = item;
                }
            }
        }
        Ok(Received {
            items: items.into_iter().map(|b| b as u8).collect(),
            remake: remake.into_iter().map(|b| b as u8).collect(),
            remake_items,
        })
    })
}

#[tauri::command]
pub fn set_player_received(
    state: State<AppState>,
    player: usize,
    items: Vec<u8>,
    remake: Vec<u8>,
) -> CmdResult<()> {
    let i: Vec<bool> = items.iter().map(|b| *b != 0).collect();
    let r: Vec<bool> = remake.iter().map(|b| *b != 0).collect();
    with_save_mut!(state, |s| Ok(s.set_received_items(player, &i, &r)?))
}

// --- mail -----------------------------------------------------------------------

#[derive(Serialize)]
pub struct MailBox {
    pub mails: Vec<Mail>,
    pub next_id: u32,
}

#[tauri::command]
pub fn player_mail(state: State<AppState>, player: usize) -> CmdResult<MailBox> {
    with_save!(state, |s| Ok(MailBox {
        mails: s.mails(player)?,
        next_id: s.postbox_next_id(player)?
    }))
}

/// Saves a letter. New letters get the next postbox sort index assigned.
#[tauri::command]
pub fn set_player_mail(
    state: State<AppState>,
    player: usize,
    mut mail: Mail,
) -> CmdResult<MailBox> {
    with_save_mut!(state, |s| {
        let existing = &s.mails(player)?[mail.slot];
        if !existing.is_used() || existing.sort_index == 0 {
            let next = s.postbox_next_id(player)?;
            mail.sort_index = next.min(299) as u16;
            s.set_postbox_next_id(player, (next + 1).min(299))?;
        }
        if mail.status != 3 && mail.status != 4 {
            mail.status = 3;
        }
        s.set_mail(player, &mail)?;
        Ok(MailBox {
            mails: s.mails(player)?,
            next_id: s.postbox_next_id(player)?,
        })
    })
}

#[tauri::command]
pub fn clear_player_mail(state: State<AppState>, player: usize, slot: usize) -> CmdResult<()> {
    with_save_mut!(state, |s| Ok(s.clear_mail(player, slot)?))
}

#[tauri::command]
pub fn export_player_mail(
    state: State<AppState>,
    player: usize,
    slot: usize,
    path: String,
) -> CmdResult<()> {
    with_save!(state, |s| {
        std::fs::write(path, s.mail_raw(player, slot)?)?;
        Ok(())
    })
}

#[tauri::command]
pub fn import_player_mail(
    state: State<AppState>,
    player: usize,
    slot: usize,
    path: String,
) -> CmdResult<()> {
    let data = std::fs::read(path)?;
    with_save_mut!(state, |s| {
        let existing = &s.mails(player)?[slot];
        let keep_index = existing.is_used().then_some(existing.sort_index);
        s.set_mail_raw(player, slot, &data)?;
        let mut m = s.mails(player)?[slot].clone();
        match keep_index {
            Some(i) => m.sort_index = i,
            None => {
                let next = s.postbox_next_id(player)?;
                m.sort_index = next.min(299) as u16;
                s.set_postbox_next_id(player, (next + 1).min(299))?;
            }
        }
        let o = s.postbox_ofs.mail_list + slot * nhse_core::structs::mail::SIZE + 0x374;
        nhse_core::util::bytes::put_u16(&mut s.players[player].postbox.data, o, m.sort_index);
        Ok(())
    })
}

/// Find & replace text across every letter of a player's postbox.
#[tauri::command]
pub fn replace_in_mail(
    state: State<AppState>,
    player: usize,
    find: String,
    replace: String,
    names: bool,
    text: bool,
) -> CmdResult<usize> {
    if find.is_empty() {
        return Ok(0);
    }
    with_save_mut!(state, |s| {
        let mut count = 0;
        for mut m in s.mails(player)? {
            if !m.is_used() {
                continue;
            }
            let mut hits = 0;
            let mut fix = |v: &mut String| {
                let n = v.matches(find.as_str()).count();
                if n > 0 {
                    hits += n;
                    *v = v.replace(find.as_str(), &replace);
                }
            };
            if names {
                fix(&mut m.sender_name);
                fix(&mut m.receiver_player_name);
                fix(&mut m.sender_island_name);
                fix(&mut m.receiver_island_name);
            }
            if text {
                fix(&mut m.to_line);
                fix(&mut m.message);
                fix(&mut m.from_line);
            }
            if hits > 0 {
                count += hits;
                s.set_mail(player, &m)?;
            }
        }
        Ok(count)
    })
}

// --- item list files ---------------------------------------------------------------

#[tauri::command]
pub fn export_items_file(items: Vec<Item>, path: String) -> CmdResult<()> {
    let mut d = vec![0u8; items.len() * 8];
    Item::write_array(&mut d, 0, &items);
    std::fs::write(path, d)?;
    Ok(())
}

#[tauri::command]
pub fn import_items_file(path: String) -> CmdResult<Vec<Item>> {
    let d = std::fs::read(path)?;
    if d.len() % 8 != 0 {
        return err(format!(
            "Item list files must be a multiple of 8 bytes (got {}).",
            d.len()
        ));
    }
    Ok(Item::read_array(&d, 0, d.len() / 8))
}
