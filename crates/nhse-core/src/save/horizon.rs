//! The complete save: `main.dat` plus every player's files.
use super::file::SaveFile;
use super::hash::HashRegion;
use super::offsets_gen::{self as ofs, MainOffsets, PersonalOffsets, PostBoxOffsets};
use super::provider::SaveSource;
use super::revision::{self, DetectedRevision, REVISIONS};
use crate::error::{Error, Result};
use crate::util::bytes::replace_occurrences;
use serde::Serialize;
use std::path::Path;

/// Files belonging to one player (a `VillagerN` folder).
#[derive(Debug, Clone)]
pub struct PlayerFiles {
    pub dir: String,
    pub personal: SaveFile,
    pub photo: SaveFile,
    pub postbox: SaveFile,
    pub profile: SaveFile,
    pub wherearen: Option<SaveFile>,
}

impl PlayerFiles {
    fn load(src: &SaveSource, dir: &str) -> Result<Self> {
        Ok(Self {
            dir: dir.to_string(),
            personal: SaveFile::load(src, dir, "personal")?,
            photo: SaveFile::load(src, dir, "photo_studio_island")?,
            postbox: SaveFile::load(src, dir, "postbox")?,
            profile: SaveFile::load(src, dir, "profile")?,
            wherearen: if SaveFile::exists(src, dir, "wherearen") {
                Some(SaveFile::load(src, dir, "wherearen")?)
            } else {
                None
            },
        })
    }

    pub fn files(&self) -> impl Iterator<Item = &SaveFile> {
        [&self.personal, &self.photo, &self.postbox, &self.profile]
            .into_iter()
            .chain(self.wherearen.iter())
    }

    pub fn files_mut(&mut self) -> impl Iterator<Item = &mut SaveFile> {
        [
            &mut self.personal,
            &mut self.photo,
            &mut self.postbox,
            &mut self.profile,
        ]
        .into_iter()
        .chain(self.wherearen.iter_mut())
    }
}

/// A file whose hashes do not validate.
#[derive(Debug, Clone, Serialize)]
pub struct InvalidHash {
    pub file: String,
    pub region: String,
}

#[derive(Debug)]
pub struct HorizonSave {
    pub source: SaveSource,
    pub revision: DetectedRevision,
    pub main: SaveFile,
    pub players: Vec<PlayerFiles>,
    pub main_ofs: &'static MainOffsets,
    pub personal_ofs: &'static PersonalOffsets,
    pub postbox_ofs: &'static PostBoxOffsets,
    /// Non fatal problems detected while loading.
    pub warnings: Vec<String>,
}

impl HorizonSave {
    /// Opens a save from a folder, a `main.dat` path, or a `.zip`.
    pub fn open(path: &Path) -> Result<Self> {
        let src = if path.is_dir() {
            SaveSource::folder(path)?
        } else if path
            .extension()
            .map(|e| e.eq_ignore_ascii_case("zip"))
            .unwrap_or(false)
        {
            SaveSource::zip(path)?
        } else if path
            .file_name()
            .map(|n| n.eq_ignore_ascii_case("main.dat") || n.eq_ignore_ascii_case("mainHeader.dat"))
            .unwrap_or(false)
        {
            SaveSource::folder(
                path.parent()
                    .ok_or_else(|| Error::Invalid("invalid path".into()))?,
            )?
        } else {
            return Err(Error::Invalid(format!(
                "Unsupported selection: {}. Choose the save folder (containing main.dat), main.dat itself, or a .zip of the save.",
                path.display()
            )));
        };
        Self::from_source(src)
    }

    pub fn from_source(src: SaveSource) -> Result<Self> {
        let main = SaveFile::load(&src, "", "main")?;
        let detected = revision::detect(&main.info, main.data.len()).ok_or_else(|| {
            Error::UnknownRevision(format!(
                "This save comes from a game version NHSE does not know yet (header {:X}/{:X}, save revision {}, main.dat size 0x{:X}).",
                main.info.major,
                main.info.minor,
                main.info.save_revision,
                main.data.len()
            ))
        })?;
        let idx = detected.index;
        let main_ofs = ofs::main_offsets(idx).expect("offset table covers every revision");
        let personal_ofs = ofs::personal_offsets(idx).expect("offset table covers every revision");
        let postbox_ofs = ofs::postbox_offsets(idx).expect("offset table covers every revision");

        let mut warnings = Vec::new();
        if detected.compatibility_mode {
            warnings.push(format!(
                "Save revision {} is newer than the latest known ({}), but its layout is identical to game version {}. Editing should be safe; keep a backup.",
                main.info.save_revision,
                REVISIONS[idx].save_revision,
                REVISIONS[idx].version
            ));
        }

        let mut players = Vec::new();
        for dir in src.player_dirs()? {
            players.push(PlayerFiles::load(&src, &dir)?);
        }
        if players.is_empty() {
            return Err(Error::Invalid(
                "No players (Villager0 … Villager7 folders) were found next to main.dat.".into(),
            ));
        }

        let mut save = Self {
            source: src,
            revision: detected,
            main,
            players,
            main_ofs,
            personal_ofs,
            postbox_ofs,
            warnings,
        };
        save.validate_sizes()?;
        Ok(save)
    }

    pub fn version(&self) -> &'static str {
        REVISIONS[self.revision.index].version
    }

    fn hash_rev(&self) -> super::hash::HashRev {
        REVISIONS[self.revision.index].hash
    }

    /// Every player must have been migrated to the same revision as `main.dat`.
    fn validate_sizes(&mut self) -> Result<()> {
        let idx = self.revision.index;
        let check = |f: &SaveFile| -> Option<String> {
            let expect = revision::expected_size(idx, &f.file_name())?;
            (f.data.len() != expect).then(|| {
                format!(
                    "{}: size 0x{:X} does not match expected 0x{:X} for {}",
                    f.data_path(),
                    f.data.len(),
                    expect,
                    REVISIONS[idx].version
                )
            })
        };
        if let Some(e) = check(&self.main) {
            return Err(Error::Invalid(e));
        }
        let mut errors = Vec::new();
        for p in &self.players {
            for f in p.files() {
                if let Some(e) = check(f) {
                    errors.push(e);
                }
            }
        }
        if !errors.is_empty() {
            return Err(Error::Invalid(format!(
                "Some player files were not updated to the same game version as main.dat. Launch the game and let every resident save once, then dump again.\n{}",
                errors.join("\n")
            )));
        }
        Ok(())
    }

    /// Hashes, encrypts and writes every file back to where it was loaded from.
    pub fn save(&mut self, seed: u32) -> Result<()> {
        let rev = self.hash_rev();
        self.main.update_hashes(rev);
        self.main.write_encrypted(&mut self.source, seed)?;
        for p in &mut self.players {
            for f in p.files_mut() {
                f.update_hashes(rev);
                f.write_encrypted(&mut self.source, seed)?;
            }
        }
        self.source.flush()
    }

    /// Lists hash regions that fail validation (before saving).
    pub fn invalid_hashes(&self) -> Vec<InvalidHash> {
        let rev = self.hash_rev();
        let mut out = Vec::new();
        let mut push = |f: &SaveFile, r: Vec<HashRegion>| {
            for x in r {
                out.push(InvalidHash {
                    file: f.data_path(),
                    region: x.to_string(),
                });
            }
        };
        push(&self.main, self.main.invalid_hashes(rev));
        for p in &self.players {
            for f in p.files() {
                push(f, f.invalid_hashes(rev));
            }
        }
        out
    }

    /// Replaces an identity byte sequence (town or player id+name) across all files.
    pub fn change_identity(&mut self, original: &[u8], updated: &[u8]) -> usize {
        if original == updated {
            return 0;
        }
        let mut n = replace_occurrences(&mut self.main.data, original, updated);
        for p in &mut self.players {
            for f in p.files_mut() {
                n += replace_occurrences(&mut f.data, original, updated);
            }
        }
        n
    }

    /// Writes every decrypted file into `dir` (same folder structure as the save).
    pub fn dump_decrypted(&self, dir: &Path) -> Result<usize> {
        let mut n = 0;
        let mut write = |f: &SaveFile| -> Result<()> {
            let p = dir.join(f.data_path());
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(p, &f.data)?;
            n += 1;
            Ok(())
        };
        write(&self.main)?;
        for p in &self.players {
            for f in p.files() {
                write(f)?;
            }
        }
        Ok(n)
    }

    /// Loads decrypted files previously produced by [`Self::dump_decrypted`].
    pub fn load_decrypted(&mut self, dir: &Path) -> Result<usize> {
        let mut n = 0;
        let mut load = |f: &mut SaveFile| -> Result<()> {
            let p = dir.join(f.data_path());
            if p.is_file() {
                f.replace_decrypted(std::fs::read(&p)?)?;
                n += 1;
            }
            Ok(())
        };
        load(&mut self.main)?;
        for p in &mut self.players {
            for f in p.files_mut() {
                load(f)?;
            }
        }
        if n == 0 {
            return Err(Error::MissingFile(format!(
                "no decrypted files (main.dat, Villager0/personal.dat, …) found in {}",
                dir.display()
            )));
        }
        Ok(n)
    }
}
