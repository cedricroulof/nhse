//! Murmur3 (x86_32) checksums used to validate save data regions.
use super::hashes_gen as gen;

#[inline]
fn scramble(v: u32) -> u32 {
    let v = v.wrapping_mul(0x16A8_8000) | (v.wrapping_mul(0xCC9E_2D51) >> 17);
    v.wrapping_mul(0x1B87_3593)
}

#[inline]
fn advance(mut h: u32, v: u32) -> u32 {
    h ^= scramble(v);
    h = h.rotate_left(13);
    h.wrapping_mul(5).wrapping_add(0xE654_6B64)
}

#[inline]
fn finalize(mut h: u32, len: u32) -> u32 {
    h ^= len;
    h ^= h >> 16;
    h = h.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    h = h.wrapping_mul(0xC2B2_AE35);
    h ^= h >> 16;
    h
}

/// Computes the game's Murmur3 hash of `data` (length must be a multiple of 4).
pub fn murmur3(data: &[u8], seed: u32) -> u32 {
    let mut h = seed;
    for c in data.chunks_exact(4) {
        h = advance(h, u32::from_le_bytes([c[0], c[1], c[2], c[3]]));
    }
    finalize(h, data.len() as u32)
}

/// A hashed region: the u32 checksum lives at `offset`, covering the `length` bytes after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct HashRegion {
    pub offset: usize,
    pub length: usize,
}

impl HashRegion {
    pub const fn new(offset: usize, length: usize) -> Self {
        Self { offset, length }
    }
    pub fn begin(&self) -> usize {
        self.offset + 4
    }
    pub fn end(&self) -> usize {
        self.begin() + self.length
    }
}

impl std::fmt::Display for HashRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "0x{:X}: (0x{:X}-0x{:X})",
            self.offset,
            self.begin(),
            self.end()
        )
    }
}

/// Hash regions for one file of a given revision.
#[derive(Debug)]
pub struct FileHashDetails {
    pub file_name: &'static str,
    pub file_size: u32,
    pub regions: &'static [HashRegion],
}

/// Hash table set for a hash revision index (see [`super::revision::REVISIONS`]).
pub fn hash_set(hash_rev: HashRev) -> &'static [FileHashDetails] {
    match hash_rev {
        HashRev::R100 => gen::REV_100,
        HashRev::R110 => gen::REV_110,
        HashRev::R120 => gen::REV_120,
        HashRev::R130 => gen::REV_130,
        HashRev::R140 => gen::REV_140,
        HashRev::R150 => gen::REV_150,
        HashRev::R160 => gen::REV_160,
        HashRev::R170 => gen::REV_170,
        HashRev::R180 => gen::REV_180,
        HashRev::R190 => gen::REV_190,
        HashRev::R1100 => gen::REV_1100,
        HashRev::R1110 => gen::REV_1110,
        HashRev::R200 => gen::REV_200,
        HashRev::R300 => gen::REV_300,
    }
}

/// Distinct hash/size layouts across game patches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashRev {
    R100,
    R110,
    R120,
    R130,
    R140,
    R150,
    R160,
    R170,
    R180,
    R190,
    R1100,
    R1110,
    R200,
    R300,
}

pub fn details_for(hash_rev: HashRev, file_name: &str) -> Option<&'static FileHashDetails> {
    hash_set(hash_rev).iter().find(|d| d.file_name == file_name)
}

/// Recomputes and stores all hashes for `data`.
pub fn update_hashes(data: &mut [u8], details: &FileHashDetails) {
    for r in details.regions {
        if r.end() > data.len() {
            continue;
        }
        let h = murmur3(&data[r.begin()..r.end()], 0);
        data[r.offset..r.offset + 4].copy_from_slice(&h.to_le_bytes());
    }
}

/// Lists regions whose stored hash does not match the data.
pub fn invalid_hashes(data: &[u8], details: &FileHashDetails) -> Vec<HashRegion> {
    details
        .regions
        .iter()
        .filter(|r| {
            r.end() > data.len()
                || murmur3(&data[r.begin()..r.end()], 0)
                    != u32::from_le_bytes([
                        data[r.offset],
                        data[r.offset + 1],
                        data[r.offset + 2],
                        data[r.offset + 3],
                    ])
        })
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn murmur_matches_standard_x86_32() {
        // Standard MurmurHash3_x86_32 test vectors (inputs with length % 4 == 0).
        assert_eq!(murmur3(b"", 0), 0);
        assert_eq!(murmur3(b"", 1), 0x514E_28B7);
        assert_eq!(murmur3(&[0, 0, 0, 0], 0), 0x2362_F9DE);
        assert_eq!(murmur3(&[0x21, 0x43, 0x65, 0x87], 0), 0xF55B_516B);
        assert_eq!(murmur3(&[0x21, 0x43, 0x65, 0x87], 0x5082_EDEE), 0x2362_F9DE);
    }
}
