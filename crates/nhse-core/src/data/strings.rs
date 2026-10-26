//! Localized game strings (item names, villager names, …) for the 9 supported languages.
use crate::data::tables::tables;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Supported language codes, in display order.
pub const LANGUAGES: [(&str, &str); 9] = [
    ("en", "English"),
    ("jp", "日本語"),
    ("de", "Deutsch"),
    ("es", "Español"),
    ("fr", "Français"),
    ("it", "Italiano"),
    ("ko", "한국어"),
    ("zhs", "简体中文"),
    ("zht", "繁體中文"),
];

fn raw(ident: &str, lang: &str) -> &'static str {
    match (ident, lang) {
        ("villager", "en") => include_str!("../../resources/text/en/text_villager_en.txt"),
        ("phrase", "en") => include_str!("../../resources/text/en/text_phrase_en.txt"),
        ("item", "en") => include_str!("../../resources/text/en/text_item_en.txt"),
        ("mail", "en") => include_str!("../../resources/text/en/text_mail_en.txt"),
        ("body_parts", "en") => include_str!("../../resources/text/en/text_body_parts_en.txt"),
        ("body_color", "en") => include_str!("../../resources/text/en/text_body_color_en.txt"),
        ("fabric_parts", "en") => include_str!("../../resources/text/en/text_fabric_parts_en.txt"),
        ("fabric_color", "en") => include_str!("../../resources/text/en/text_fabric_color_en.txt"),
        ("internal", "en") => include_str!("../../resources/text/en/internal_en.txt"),
        ("villager", "jp") => include_str!("../../resources/text/jp/text_villager_jp.txt"),
        ("phrase", "jp") => include_str!("../../resources/text/jp/text_phrase_jp.txt"),
        ("item", "jp") => include_str!("../../resources/text/jp/text_item_jp.txt"),
        ("mail", "jp") => include_str!("../../resources/text/jp/text_mail_jp.txt"),
        ("body_parts", "jp") => include_str!("../../resources/text/jp/text_body_parts_jp.txt"),
        ("body_color", "jp") => include_str!("../../resources/text/jp/text_body_color_jp.txt"),
        ("fabric_parts", "jp") => include_str!("../../resources/text/jp/text_fabric_parts_jp.txt"),
        ("fabric_color", "jp") => include_str!("../../resources/text/jp/text_fabric_color_jp.txt"),
        ("internal", "jp") => include_str!("../../resources/text/jp/internal_jp.txt"),
        ("villager", "de") => include_str!("../../resources/text/de/text_villager_de.txt"),
        ("phrase", "de") => include_str!("../../resources/text/de/text_phrase_de.txt"),
        ("item", "de") => include_str!("../../resources/text/de/text_item_de.txt"),
        ("mail", "de") => include_str!("../../resources/text/de/text_mail_de.txt"),
        ("body_parts", "de") => include_str!("../../resources/text/de/text_body_parts_de.txt"),
        ("body_color", "de") => include_str!("../../resources/text/de/text_body_color_de.txt"),
        ("fabric_parts", "de") => include_str!("../../resources/text/de/text_fabric_parts_de.txt"),
        ("fabric_color", "de") => include_str!("../../resources/text/de/text_fabric_color_de.txt"),
        ("internal", "de") => include_str!("../../resources/text/de/internal_de.txt"),
        ("villager", "es") => include_str!("../../resources/text/es/text_villager_es.txt"),
        ("phrase", "es") => include_str!("../../resources/text/es/text_phrase_es.txt"),
        ("item", "es") => include_str!("../../resources/text/es/text_item_es.txt"),
        ("mail", "es") => include_str!("../../resources/text/es/text_mail_es.txt"),
        ("body_parts", "es") => include_str!("../../resources/text/es/text_body_parts_es.txt"),
        ("body_color", "es") => include_str!("../../resources/text/es/text_body_color_es.txt"),
        ("fabric_parts", "es") => include_str!("../../resources/text/es/text_fabric_parts_es.txt"),
        ("fabric_color", "es") => include_str!("../../resources/text/es/text_fabric_color_es.txt"),
        ("internal", "es") => include_str!("../../resources/text/es/internal_es.txt"),
        ("villager", "fr") => include_str!("../../resources/text/fr/text_villager_fr.txt"),
        ("phrase", "fr") => include_str!("../../resources/text/fr/text_phrase_fr.txt"),
        ("item", "fr") => include_str!("../../resources/text/fr/text_item_fr.txt"),
        ("mail", "fr") => include_str!("../../resources/text/fr/text_mail_fr.txt"),
        ("body_parts", "fr") => include_str!("../../resources/text/fr/text_body_parts_fr.txt"),
        ("body_color", "fr") => include_str!("../../resources/text/fr/text_body_color_fr.txt"),
        ("fabric_parts", "fr") => include_str!("../../resources/text/fr/text_fabric_parts_fr.txt"),
        ("fabric_color", "fr") => include_str!("../../resources/text/fr/text_fabric_color_fr.txt"),
        ("internal", "fr") => include_str!("../../resources/text/fr/internal_fr.txt"),
        ("villager", "it") => include_str!("../../resources/text/it/text_villager_it.txt"),
        ("phrase", "it") => include_str!("../../resources/text/it/text_phrase_it.txt"),
        ("item", "it") => include_str!("../../resources/text/it/text_item_it.txt"),
        ("mail", "it") => include_str!("../../resources/text/it/text_mail_it.txt"),
        ("body_parts", "it") => include_str!("../../resources/text/it/text_body_parts_it.txt"),
        ("body_color", "it") => include_str!("../../resources/text/it/text_body_color_it.txt"),
        ("fabric_parts", "it") => include_str!("../../resources/text/it/text_fabric_parts_it.txt"),
        ("fabric_color", "it") => include_str!("../../resources/text/it/text_fabric_color_it.txt"),
        ("internal", "it") => include_str!("../../resources/text/it/internal_it.txt"),
        ("villager", "ko") => include_str!("../../resources/text/ko/text_villager_ko.txt"),
        ("phrase", "ko") => include_str!("../../resources/text/ko/text_phrase_ko.txt"),
        ("item", "ko") => include_str!("../../resources/text/ko/text_item_ko.txt"),
        ("mail", "ko") => include_str!("../../resources/text/ko/text_mail_ko.txt"),
        ("body_parts", "ko") => include_str!("../../resources/text/ko/text_body_parts_ko.txt"),
        ("body_color", "ko") => include_str!("../../resources/text/ko/text_body_color_ko.txt"),
        ("fabric_parts", "ko") => include_str!("../../resources/text/ko/text_fabric_parts_ko.txt"),
        ("fabric_color", "ko") => include_str!("../../resources/text/ko/text_fabric_color_ko.txt"),
        ("internal", "ko") => include_str!("../../resources/text/ko/internal_ko.txt"),
        ("villager", "zhs") => include_str!("../../resources/text/zhs/text_villager_zhs.txt"),
        ("phrase", "zhs") => include_str!("../../resources/text/zhs/text_phrase_zhs.txt"),
        ("item", "zhs") => include_str!("../../resources/text/zhs/text_item_zhs.txt"),
        ("mail", "zhs") => include_str!("../../resources/text/zhs/text_mail_zhs.txt"),
        ("body_parts", "zhs") => include_str!("../../resources/text/zhs/text_body_parts_zhs.txt"),
        ("body_color", "zhs") => include_str!("../../resources/text/zhs/text_body_color_zhs.txt"),
        ("fabric_parts", "zhs") => include_str!("../../resources/text/zhs/text_fabric_parts_zhs.txt"),
        ("fabric_color", "zhs") => include_str!("../../resources/text/zhs/text_fabric_color_zhs.txt"),
        ("internal", "zhs") => include_str!("../../resources/text/zhs/internal_zhs.txt"),
        ("villager", "zht") => include_str!("../../resources/text/zht/text_villager_zht.txt"),
        ("phrase", "zht") => include_str!("../../resources/text/zht/text_phrase_zht.txt"),
        ("item", "zht") => include_str!("../../resources/text/zht/text_item_zht.txt"),
        ("mail", "zht") => include_str!("../../resources/text/zht/text_mail_zht.txt"),
        ("body_parts", "zht") => include_str!("../../resources/text/zht/text_body_parts_zht.txt"),
        ("body_color", "zht") => include_str!("../../resources/text/zht/text_body_color_zht.txt"),
        ("fabric_parts", "zht") => include_str!("../../resources/text/zht/text_fabric_parts_zht.txt"),
        ("fabric_color", "zht") => include_str!("../../resources/text/zht/text_fabric_color_zht.txt"),
        ("internal", "zht") => include_str!("../../resources/text/zht/internal_zht.txt"),
        _ => "",
    }
}

fn lines(ident: &str, lang: &str) -> Vec<String> {
    let mut s = raw(ident, lang);
    if s.is_empty() {
        s = raw(ident, "en");
    }
    let s = s.strip_prefix('\u{feff}').unwrap_or(s);
    s.trim_end_matches(['\r', '\n']).split('\n').map(|l| l.trim_end_matches('\r').to_string()).collect()
}

fn map(lines: &[String], sep: char) -> HashMap<String, String> {
    lines
        .iter()
        .filter_map(|l| l.split_once(sep))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// All strings for one language.
#[derive(Debug)]
pub struct GameStrings {
    pub lang: &'static str,
    /// Raw item names indexed by item id (may be empty).
    pub items: Vec<String>,
    /// Display names: unique and never empty (`(Item #123)` placeholders, duplicates suffixed).
    pub items_display: Vec<String>,
    /// Villager internal name (e.g. `cat00`) → display name, in file order.
    pub villagers: Vec<(String, String)>,
    pub villager_map: HashMap<String, String>,
    pub phrases: HashMap<String, String>,
    pub mail: Vec<(String, String)>,
    pub body_parts: HashMap<String, String>,
    pub body_color: HashMap<String, String>,
    pub fabric_parts: HashMap<String, String>,
    pub fabric_color: HashMap<String, String>,
    /// Translations of internal identifiers (achievement / flag names).
    pub internal: HashMap<String, String>,
}

fn pairs(lines: &[String]) -> Vec<(String, String)> {
    lines.iter().filter_map(|l| l.split_once('\t')).map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

impl GameStrings {
    fn load(lang: &'static str) -> Self {
        let items = lines("item", lang);
        let items_display = display_list(&items);
        let villagers = pairs(&lines("villager", lang));
        let villager_map = villagers.iter().cloned().collect();
        Self {
            lang,
            items,
            items_display,
            villagers,
            villager_map,
            phrases: map(&lines("phrase", lang), '\t'),
            mail: pairs(&lines("mail", lang)),
            body_parts: map(&lines("body_parts", lang), '\t'),
            body_color: map(&lines("body_color", lang), '\t'),
            fabric_parts: map(&lines("fabric_parts", lang), '\t'),
            fabric_color: map(&lines("fabric_color", lang), '\t'),
            internal: map(&lines("internal", lang), '='),
        }
    }

    /// Name of an item id, including field items (60000+).
    pub fn item_name(&self, id: u16) -> String {
        if id == 0xFFFE {
            return self.items.first().cloned().unwrap_or_else(|| "(None)".into());
        }
        if let Some(n) = self.items_display.get(id as usize) {
            return n.clone();
        }
        if let Some(f) = tables().field_items.get(&id) {
            return f.name.clone();
        }
        if id == 63_005 {
            return "Lloyd".into();
        }
        format!("??? (0x{id:04X})")
    }

    pub fn villager_name(&self, internal: &str) -> String {
        self.villager_map.get(internal).cloned().unwrap_or_else(|| internal.to_string())
    }

    pub fn villager_phrase(&self, internal: &str) -> String {
        self.phrases.get(internal).cloned().unwrap_or_default()
    }

    pub fn translate_internal<'a>(&'a self, name: &'a str) -> &'a str {
        self.internal.get(name).map(|s| s.as_str()).unwrap_or(name)
    }
}

/// Builds the unique display list (mirrors the original editor's behaviour).
pub fn display_list(items: &[String]) -> Vec<String> {
    let mut out: Vec<String> = items.to_vec();
    if let Some(first) = out.first_mut() {
        first.clear();
    }
    let mut seen = std::collections::HashSet::new();
    for (i, item) in out.iter_mut().enumerate() {
        if item.is_empty() {
            *item = format!("(Item #{i:03})");
        } else if !seen.insert(item.clone()) {
            item.push_str(&format!(" (#{i:03})"));
        }
    }
    out
}

static CACHE: [OnceLock<GameStrings>; 9] = [
    OnceLock::new(), OnceLock::new(), OnceLock::new(),
    OnceLock::new(), OnceLock::new(), OnceLock::new(),
    OnceLock::new(), OnceLock::new(), OnceLock::new(),
];

/// Returns the (cached) strings for a language code; unknown codes fall back to English.
pub fn strings(lang: &str) -> &'static GameStrings {
    let idx = LANGUAGES.iter().position(|(c, _)| *c == lang).unwrap_or(0);
    CACHE[idx].get_or_init(|| GameStrings::load(LANGUAGES[idx].0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn english_loads() {
        let s = strings("en");
        assert!(s.items.len() > 10_000);
        assert_eq!(s.item_name(0xFFFE), "(None)");
        assert!(!s.villagers.is_empty());
        assert_eq!(s.villager_name("cat00"), s.villager_map["cat00"]);
        assert!(s.items_display.iter().all(|x| !x.is_empty()));
        let f = strings("fr");
        assert_eq!(f.lang, "fr");
    }
}
