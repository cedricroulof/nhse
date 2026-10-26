//! An encrypted `<name>.dat` + `<name>Header.dat` pair, held decrypted in memory.
use super::crypto;
use super::hash::{self, HashRegion, HashRev};
use super::provider::SaveSource;
use super::revision::FileHeaderInfo;
use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct SaveFile {
    /// Base name without extension, e.g. `personal`.
    pub name: String,
    /// Folder relative to the save root (`""` or `Villager0`).
    pub dir: String,
    /// Decrypted file contents.
    pub data: Vec<u8>,
    /// Header (key material + version info).
    pub header: Vec<u8>,
    pub info: FileHeaderInfo,
}

impl SaveFile {
    fn rel(dir: &str, file: &str) -> String {
        if dir.is_empty() {
            file.to_string()
        } else {
            format!("{dir}/{file}")
        }
    }

    pub fn data_path(&self) -> String {
        Self::rel(&self.dir, &format!("{}.dat", self.name))
    }

    pub fn header_path(&self) -> String {
        Self::rel(&self.dir, &format!("{}Header.dat", self.name))
    }

    pub fn file_name(&self) -> String {
        format!("{}.dat", self.name)
    }

    pub fn exists(src: &SaveSource, dir: &str, name: &str) -> bool {
        src.exists(&Self::rel(dir, &format!("{name}.dat"))) && src.exists(&Self::rel(dir, &format!("{name}Header.dat")))
    }

    /// Reads and decrypts a file pair.
    pub fn load(src: &SaveSource, dir: &str, name: &str) -> Result<Self> {
        let header = src.read(&Self::rel(dir, &format!("{name}Header.dat")))?;
        let mut data = src.read(&Self::rel(dir, &format!("{name}.dat")))?;
        if header.len() < 0x300 {
            return Err(Error::Invalid(format!(
                "{} is too small ({} bytes) to be a valid header",
                Self::rel(dir, &format!("{name}Header.dat")),
                header.len()
            )));
        }
        crypto::decrypt(&header, &mut data);
        let info = FileHeaderInfo::read(&header);
        Ok(Self { name: name.to_string(), dir: dir.to_string(), data, header, info })
    }

    /// Creates a file pair from already decrypted data (used by "load decrypted").
    pub fn replace_decrypted(&mut self, data: Vec<u8>) -> Result<()> {
        if data.len() != self.data.len() {
            return Err(Error::Invalid(format!(
                "{}: expected {} bytes, got {}",
                self.file_name(),
                self.data.len(),
                data.len()
            )));
        }
        self.data = data;
        Ok(())
    }

    pub fn update_hashes(&mut self, rev: HashRev) {
        if let Some(d) = hash::details_for(rev, &self.file_name()) {
            hash::update_hashes(&mut self.data, d);
        }
    }

    pub fn invalid_hashes(&self, rev: HashRev) -> Vec<HashRegion> {
        hash::details_for(rev, &self.file_name())
            .map(|d| hash::invalid_hashes(&self.data, d))
            .unwrap_or_default()
    }

    /// Encrypts the file and writes both parts to `src`.
    pub fn write_encrypted(&self, src: &mut SaveSource, seed: u32) -> Result<()> {
        let (enc, hdr) = crypto::encrypt(&self.data, seed, &self.header);
        src.write(&self.data_path(), &enc)?;
        src.write(&self.header_path(), &hdr)?;
        Ok(())
    }
}
