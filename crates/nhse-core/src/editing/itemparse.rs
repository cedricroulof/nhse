//! Parses user-provided item lists (hex codes or item names).
use crate::data::strings::GameStrings;
use crate::structs::Item;

/// Parses one entry per line (or comma): a hex value (`0A54`, `0000000500000A54`) or an item name.
/// Returns the items and the entries that could not be understood.
pub fn parse_items(text: &str, s: &GameStrings) -> (Vec<Item>, Vec<String>) {
    let mut items = Vec::new();
    let mut errors = Vec::new();
    for raw in text.split(['\n', ',', ';']) {
        let entry = raw.trim();
        if entry.is_empty() {
            continue;
        }
        let hex = entry.trim_start_matches("0x").trim_start_matches("0X");
        if !hex.is_empty() && hex.len() <= 16 && hex.chars().all(|c| c.is_ascii_hexdigit()) && !is_name(entry, s) {
            if let Ok(v) = u64::from_str_radix(hex, 16) {
                items.push(Item::from_raw(v));
                continue;
            }
        }
        match find_by_name(entry, s) {
            Some(id) => items.push(Item::new(id)),
            None => errors.push(entry.to_string()),
        }
    }
    (items, errors)
}

fn norm(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(|c| c.to_lowercase()).collect()
}

fn is_name(entry: &str, s: &GameStrings) -> bool {
    let n = norm(entry);
    s.items.iter().any(|x| norm(x) == n)
}

/// Exact (case/punctuation insensitive) match, falling back to a unique prefix match.
pub fn find_by_name(name: &str, s: &GameStrings) -> Option<u16> {
    let n = norm(name);
    if n.is_empty() {
        return None;
    }
    if let Some(i) = s.items.iter().position(|x| norm(x) == n) {
        return Some(i as u16);
    }
    let mut hits = s.items.iter().enumerate().filter(|(_, x)| norm(x).starts_with(&n));
    match (hits.next(), hits.next()) {
        (Some((i, _)), None) => Some(i as u16),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_hex_and_names() {
        let s = crate::data::strings("en");
        let name = s.items[3082].clone(); // "flimsy shovel"
        let (items, errs) = parse_items(&format!("0A54\n0000000500000A54, {name}\nnot a real item zzz"), s);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].id, 0x0A54);
        assert_eq!(items[1].count(), 5);
        assert_eq!(items[2].id, 3082);
        assert_eq!(errs.len(), 1);
    }
}
