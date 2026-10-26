//! Bit flag helpers (LSB-first within each byte, as the game stores them).

#[inline]
pub fn get_flag(d: &[u8], offset: usize, bit_index: usize) -> bool {
    let b = d[offset + (bit_index >> 3)];
    (b >> (bit_index & 7)) & 1 == 1
}

#[inline]
pub fn set_flag(d: &mut [u8], offset: usize, bit_index: usize, value: bool) {
    let i = offset + (bit_index >> 3);
    let mask = 1u8 << (bit_index & 7);
    if value {
        d[i] |= mask;
    } else {
        d[i] &= !mask;
    }
}
