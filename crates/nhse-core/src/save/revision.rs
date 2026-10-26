//! Save revision detection.
use super::hash::{details_for, HashRev};
use crate::util::bytes::{u16le, u32le};
use serde::Serialize;

/// Metadata stored in the first 0x40 bytes of each `*Header.dat` file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FileHeaderInfo {
    pub major: u32,
    pub minor: u32,
    pub unk1: u16,
    pub header_revision: u16,
    pub unk2: u16,
    pub save_revision: u16,
}

impl FileHeaderInfo {
    pub const SIZE: usize = 0x40;

    pub fn read(header: &[u8]) -> Self {
        Self {
            major: u32le(header, 0x00),
            minor: u32le(header, 0x04),
            unk1: u16le(header, 0x08),
            header_revision: u16le(header, 0x0A),
            unk2: u16le(header, 0x0C),
            save_revision: u16le(header, 0x0E),
        }
    }
}

/// A known game patch.
#[derive(Debug, Clone, Copy)]
pub struct Revision {
    pub version: &'static str,
    pub major: u32,
    pub minor: u32,
    pub save_revision: u16,
    pub hash: HashRev,
}

impl Revision {
    pub fn matches(&self, info: &FileHeaderInfo) -> bool {
        info.major == self.major
            && info.minor == self.minor
            && info.save_revision == self.save_revision
            && info.header_revision == 0
            && info.unk1 == 2
            && info.unk2 == 2
    }
}

macro_rules! rev {
    ($v:expr, $maj:expr, $min:expr, $sr:expr, $h:ident) => {
        Revision { version: $v, major: $maj, minor: $min, save_revision: $sr, hash: HashRev::$h }
    };
}

/// Every supported game patch, indexed by "revision index".
pub static REVISIONS: &[Revision] = &[
    rev!("1.0.0", 0x00067, 0x0006F, 0, R100),
    rev!("1.1.0", 0x0006D, 0x00078, 1, R110),
    rev!("1.1.1", 0x0006D, 0x00078, 2, R110),
    rev!("1.1.2", 0x0006D, 0x00078, 3, R110),
    rev!("1.1.3", 0x0006D, 0x00078, 4, R110),
    rev!("1.1.4", 0x0006D, 0x00078, 5, R110),
    rev!("1.2.0", 0x20006, 0x20008, 6, R120),
    rev!("1.2.1", 0x20006, 0x20008, 7, R120),
    rev!("1.3.0", 0x40002, 0x40008, 8, R130),
    rev!("1.3.1", 0x40002, 0x40008, 9, R130),
    rev!("1.4.0", 0x50001, 0x5000B, 10, R140),
    rev!("1.4.1", 0x50001, 0x5000B, 11, R140),
    rev!("1.4.2", 0x50001, 0x5000B, 12, R140),
    rev!("1.5.0", 0x60001, 0x6000C, 13, R150),
    rev!("1.5.1", 0x60001, 0x6000C, 14, R150),
    rev!("1.6.0", 0x70001, 0x70006, 15, R160),
    rev!("1.7.0", 0x74001, 0x74005, 16, R170),
    rev!("1.8.0", 0x78001, 0x78001, 17, R180),
    rev!("1.9.0", 0x7C001, 0x7C006, 18, R190),
    rev!("1.10.0", 0x7D001, 0x7D004, 19, R1100),
    rev!("1.11.0", 0x7E001, 0x7E001, 20, R1110),
    rev!("1.11.1", 0x7E001, 0x7E001, 21, R1110),
    rev!("2.0.0", 0x80009, 0x80085, 22, R200),
    rev!("2.0.1", 0x80009, 0x80085, 23, R200),
    rev!("2.0.2", 0x80009, 0x80085, 24, R200),
    rev!("2.0.3", 0x80009, 0x80085, 25, R200),
    rev!("2.0.4", 0x80009, 0x80085, 26, R200),
    rev!("2.0.5", 0x80009, 0x80085, 27, R200),
    rev!("2.0.6", 0x80009, 0x80085, 28, R200),
    rev!("2.0.7", 0x80009, 0x80085, 29, R200),
    rev!("2.0.8", 0x80009, 0x80085, 30, R200),
    rev!("3.0.0", 0xA0002, 0xA0028, 31, R300),
    rev!("3.0.1", 0xA0002, 0xA0028, 32, R300),
    rev!("3.0.2", 0xA0002, 0xA0028, 33, R300),
    rev!("3.0.3", 0xA0002, 0xA0028, 34, R300),
];

/// Result of revision detection.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct DetectedRevision {
    /// Index into [`REVISIONS`].
    pub index: usize,
    /// True when the header did not exactly match a known patch, but the save layout
    /// (major/minor version and file sizes) is identical to a known one.
    pub compatibility_mode: bool,
}

/// Finds the exact revision index for a header, if known.
pub fn exact_revision(info: &FileHeaderInfo) -> Option<usize> {
    REVISIONS.iter().position(|r| r.matches(info))
}

/// Detects the revision of `main.dat`.
///
/// Newer game patches frequently bump only the `save_revision` counter while keeping the exact
/// same layout. Rather than refusing to open such saves (a recurring upstream problem), we fall
/// back to the newest known revision with the same major/minor version and identical file size.
pub fn detect(info: &FileHeaderInfo, main_size: usize) -> Option<DetectedRevision> {
    if let Some(index) = exact_revision(info) {
        return Some(DetectedRevision { index, compatibility_mode: false });
    }
    REVISIONS
        .iter()
        .enumerate()
        .rev()
        .find(|(_, r)| {
            r.major == info.major
                && r.minor == info.minor
                && info.save_revision > r.save_revision
                && details_for(r.hash, "main.dat").map(|d| d.file_size as usize) == Some(main_size)
        })
        .map(|(index, _)| DetectedRevision { index, compatibility_mode: true })
}

/// Expected size of a file for a revision.
pub fn expected_size(index: usize, file_name: &str) -> Option<usize> {
    let rev = REVISIONS.get(index)?;
    details_for(rev.hash, file_name).map(|d| d.file_size as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_ordered() {
        for (i, r) in REVISIONS.iter().enumerate() {
            assert_eq!(r.save_revision as usize, i);
        }
    }

    #[test]
    fn compat_mode() {
        let latest = REVISIONS.last().unwrap();
        let info = FileHeaderInfo { major: latest.major, minor: latest.minor, unk1: 2, header_revision: 0, unk2: 2, save_revision: 35 };
        let size = expected_size(REVISIONS.len() - 1, "main.dat").unwrap();
        let d = detect(&info, size).unwrap();
        assert!(d.compatibility_mode);
        assert_eq!(d.index, REVISIONS.len() - 1);
        assert!(detect(&info, size + 4).is_none());
    }
}
