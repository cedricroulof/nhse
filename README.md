NHSE
=====
![License](https://img.shields.io/badge/License-GPLv3-blue.svg)

Save editor for **Animal Crossing: New Horizons**, rewritten in Rust with a native
[Tauri](https://tauri.app) desktop app. It runs natively on Apple Silicon Macs.

It edits save data dumped from the Nintendo Switch. Bring your own save: the app does not dump it from your console.

## Install (macOS, Apple Silicon)

1. Download the latest `.dmg` from the
   [**NHSE (Rust/Tauri)** pre-release](https://github.com/cedricroulof/nhse/releases/tag/nightly-rust)
   (or from the *Build macOS app* workflow artifacts).
2. Open it and drag **NHSE** to **Applications**.
3. The app is ad-hoc signed, not notarized, so macOS blocks the first launch. Run this once in Terminal:
   ```sh
   xattr -dr com.apple.quarantine /Applications/NHSE.app
   ```
4. Open NHSE and choose your save folder (the one containing `main.dat`) or a `.zip` of it.

Optional: **Settings → Item images** downloads a community pack (~90 MB) with a picture for every item.

## Features

- **Island:** villagers (replace with any villager, including their house and belongings; memories; greetings; clothes and furniture), map editor, player and villager houses with a room furniture editor, museum, custom designs (pixel editor, PNG import/export), turnips, visitors, bulletin board, recycle bin and island flags.
- **Map editor:** pan and zoom with trackpad gestures, place/erase/select tools with automatic multi-tile handling, undo/redo, copy and paste of areas, bulk tools (remove weeds and other categories, water flowers, spawn items or DIY cards, replace), buildings, plaza, acres, and `.nhl`/`.nht`/`.nhb`/`.nha`/`.nhmd` import/export.
- **Residents:** profile, pockets, storage, DIY recipes, catalog, reactions, Nook Miles achievements, mailbox (letters, find and replace) and player flags.
- **Safety:** automatic backup of the save files when a save is opened (Backups page), hash verification, and revert to the last saved state. Files are only written when you press **Save** (⌘S).
- **Compatibility:** game versions 1.0.0 to 3.0.3. Saves from newer patches with an unchanged layout open in compatibility mode. Exchange files (`.nhi`, `.nhv2`, `.nhvh2`, `.nhph2`, `.nhpr2`, `.nhd`, `.nhpd`, …) work with the original NHSE.

Save reading and writing was cross-checked byte for byte against the original C# NHSE across game versions.

### Improvements

- Compact backups that store only the save files, plus a page to manage them.
- Item search with wildcards (`*`, `?`), regular expressions (`/…/`) and hex ids.
- Villager greetings use the game's full 47-character storage, with a counter that shows how much fits in a speech bubble.
- Find and replace across a player's mailbox.
- Room furniture can overhang the room edge, like in the game (Settings).
- "Give all items" skips items that crash the game.
- Fill storage with every wearable, variants included.
- Export bulletin board posts as files or text.
- Clearer map colours for shores, cliffs, waterfalls and elevation.
- Flower gene and watering editor.
- ZIP saves where the files sit inside a sub-folder now open.

## Building from source

Requirements: Rust (stable), Node.js 22, and the [Tauri prerequisites](https://tauri.app/start/prerequisites/).

```sh
npm ci
npx tauri dev                       # run in development
npx tauri build --bundles app,dmg   # release build
cargo test -p nhse-core             # core library tests
```

Layout:

- `crates/nhse-core`: save format library (encryption, hashes, revisions, data structures, game data, map and editing tools).
- `src-tauri`: desktop backend (commands, backups, image protocol).
- `src`: React + TypeScript user interface.

## Other

This project is a rewrite of NHSE by Kaphotics and contributors, and like the original it is licensed under the GNU GPL v3 (see `LICENSE`).

**We do not support or condone cheating at the expense of others.** Please be responsible!
