//! Obfuscated 32-bit integers (bells, Nook Miles, …).
use crate::util::bytes::{put_u16, put_u32, u16le, u32le};
use serde::{Deserialize, Serialize};

const ENCRYPTION_CONSTANT: u32 = 0x80E3_2B11;
const SHIFT_BASE: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedInt32 {
    pub value: u32,
    pub adjust: u16,
    pub shift: u8,
    pub checksum: u8,
}

impl EncryptedInt32 {
    pub fn checksum_of(enc: u32) -> u8 {
        let sum = enc
            .wrapping_add(enc >> 16)
            .wrapping_add(enc >> 24)
            .wrapping_add(enc >> 8);
        (sum.wrapping_sub(0x2D)) as u8
    }

    pub fn decrypt(enc: u32, shift: u8, adjust: u16) -> u32 {
        enc.rotate_right(shift as u32 + SHIFT_BASE)
            .wrapping_add(ENCRYPTION_CONSTANT)
            .wrapping_sub(adjust as u32)
    }

    pub fn encrypt(value: u32, shift: u8, adjust: u16) -> u32 {
        value
            .wrapping_add(adjust as u32)
            .wrapping_sub(ENCRYPTION_CONSTANT)
            .rotate_left(shift as u32 + SHIFT_BASE)
    }

    pub fn read(d: &[u8], o: usize) -> Self {
        let enc = u32le(d, o);
        let adjust = u16le(d, o + 4);
        let shift = d[o + 6];
        let checksum = d[o + 7];
        Self {
            value: Self::decrypt(enc, shift, adjust),
            adjust,
            shift,
            checksum,
        }
    }

    /// True if the stored checksum matches the stored value.
    pub fn is_valid(d: &[u8], o: usize) -> bool {
        Self::checksum_of(u32le(d, o)) == d[o + 7]
    }

    pub fn write(&self, d: &mut [u8], o: usize) {
        let enc = Self::encrypt(self.value, self.shift, self.adjust);
        put_u32(d, o, enc);
        put_u16(d, o + 4, self.adjust);
        d[o + 6] = self.shift;
        d[o + 7] = Self::checksum_of(enc);
    }

    /// Reads the value at `o`, writes `value` back with the same obfuscation parameters.
    /// Untouched values are left byte-identical (no re-encryption).
    pub fn set_value(d: &mut [u8], o: usize, value: u32) {
        let mut e = Self::read(d, o);
        if e.value == value {
            return;
        }
        e.value = value;
        e.write(d, o);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_reference() {
        let data = [0x8Au8, 0xC4, 0xE3, 0xCF, 0x37, 0xD5, 0x1A, 0xD3];
        assert!(EncryptedInt32::is_valid(&data, 0));
        let v = EncryptedInt32::read(&data, 0);
        assert_eq!(v.value, 31_280);
        assert_eq!(
            EncryptedInt32::encrypt(31_280, v.shift, v.adjust),
            u32le(&data, 0)
        );
        let mut out = [0u8; 8];
        v.write(&mut out, 0);
        assert_eq!(out, data);
    }
}
