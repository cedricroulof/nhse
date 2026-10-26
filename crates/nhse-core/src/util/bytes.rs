//! Little-endian accessors over byte slices.

#[inline]
pub fn u16le(d: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([d[o], d[o + 1]])
}
#[inline]
pub fn i16le(d: &[u8], o: usize) -> i16 {
    i16::from_le_bytes([d[o], d[o + 1]])
}
#[inline]
pub fn u32le(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}
#[inline]
pub fn i32le(d: &[u8], o: usize) -> i32 {
    u32le(d, o) as i32
}
#[inline]
pub fn u64le(d: &[u8], o: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&d[o..o + 8]);
    u64::from_le_bytes(b)
}
#[inline]
pub fn put_u16(d: &mut [u8], o: usize, v: u16) {
    d[o..o + 2].copy_from_slice(&v.to_le_bytes());
}
#[inline]
pub fn put_i16(d: &mut [u8], o: usize, v: i16) {
    d[o..o + 2].copy_from_slice(&v.to_le_bytes());
}
#[inline]
pub fn put_u32(d: &mut [u8], o: usize, v: u32) {
    d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
#[inline]
pub fn put_i32(d: &mut [u8], o: usize, v: i32) {
    d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
#[inline]
pub fn put_u64(d: &mut [u8], o: usize, v: u64) {
    d[o..o + 8].copy_from_slice(&v.to_le_bytes());
}

/// Reads a little-endian `i16` array.
pub fn read_i16_array(d: &[u8]) -> Vec<i16> {
    d.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect()
}
/// Writes a little-endian `i16` array into `d`.
pub fn write_i16_array(d: &mut [u8], v: &[i16]) {
    for (c, x) in d.chunks_exact_mut(2).zip(v) {
        c.copy_from_slice(&x.to_le_bytes());
    }
}
pub fn read_u16_array(d: &[u8]) -> Vec<u16> {
    d.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect()
}
pub fn write_u16_array(d: &mut [u8], v: &[u16]) {
    for (c, x) in d.chunks_exact_mut(2).zip(v) {
        c.copy_from_slice(&x.to_le_bytes());
    }
}

/// Replaces every occurrence of `find` in `data` with `replace` (same length). Returns the count.
pub fn replace_occurrences(data: &mut [u8], find: &[u8], replace: &[u8]) -> usize {
    assert_eq!(find.len(), replace.len());
    if find.is_empty() || find.len() > data.len() {
        return 0;
    }
    let mut count = 0;
    let mut i = 0;
    while i + find.len() <= data.len() {
        if &data[i..i + find.len()] == find {
            data[i..i + find.len()].copy_from_slice(replace);
            count += 1;
            i += find.len();
        } else {
            i += 1;
        }
    }
    count
}

/// Finds every index of `find` in `data`.
pub fn find_all(data: &[u8], find: &[u8]) -> Vec<usize> {
    let mut out = Vec::new();
    if find.is_empty() || find.len() > data.len() {
        return out;
    }
    let mut i = 0;
    while i + find.len() <= data.len() {
        if &data[i..i + find.len()] == find {
            out.push(i);
            i += find.len();
        } else {
            i += 1;
        }
    }
    out
}

pub fn to_hex(d: &[u8]) -> String {
    let mut s = String::with_capacity(d.len() * 2);
    for b in d {
        s.push_str(&format!("{b:02X}"));
    }
    s
}

pub fn from_hex(s: &str) -> Option<Vec<u8>> {
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

/// Writes a boolean byte only if its truthiness changed (preserves non-0/1 values).
#[inline]
pub fn put_bool(d: &mut [u8], o: usize, v: bool) {
    if (d[o] != 0) != v {
        d[o] = v as u8;
    }
}
