//! UTF-16LE string helpers matching how the game stores text.

/// Reads a null-terminated UTF-16LE string of at most `max_chars` code units.
pub fn get_string(d: &[u8], offset: usize, max_chars: usize) -> String {
    let mut units = Vec::with_capacity(max_chars);
    for i in 0..max_chars {
        let o = offset + i * 2;
        if o + 1 >= d.len() {
            break;
        }
        let u = u16::from_le_bytes([d[o], d[o + 1]]);
        if u == 0 {
            break;
        }
        units.push(u);
    }
    String::from_utf16_lossy(&units)
}

/// Encodes `value` to UTF-16LE, truncated/padded with zeros to exactly `max_chars` code units.
pub fn get_bytes(value: &str, max_chars: usize) -> Vec<u8> {
    let mut out = vec![0u8; max_chars * 2];
    let units: Vec<u16> = value.encode_utf16().collect();
    let mut n = units.len().min(max_chars);
    // never split a surrogate pair
    if n > 0 && n < units.len() && (0xD800..0xDC00).contains(&units[n - 1]) {
        n -= 1;
    }
    for (i, u) in units[..n].iter().enumerate() {
        out[i * 2..i * 2 + 2].copy_from_slice(&u.to_le_bytes());
    }
    out
}

/// Writes `value` into `d` at `offset` as a zero-padded UTF-16LE field of `max_chars` units.
pub fn set_string(d: &mut [u8], offset: usize, max_chars: usize, value: &str) {
    let b = get_bytes(value, max_chars);
    d[offset..offset + b.len()].copy_from_slice(&b);
}

/// Number of UTF-16 code units in `s`.
pub fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// Removes characters that are invalid in file names on common platforms.
pub fn clean_file_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Writes `value` only if it differs from the string currently stored (avoids normalizing
/// bytes after the terminator or lossy UTF-16 sequences of untouched fields).
pub fn set_string_if_changed(d: &mut [u8], offset: usize, max_chars: usize, value: &str) {
    if get_string(d, offset, max_chars) != value {
        set_string(d, offset, max_chars, value);
    }
}
