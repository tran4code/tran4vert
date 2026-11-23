//! The reader-visible strings. They live in `data/strings.json`, not in
//! source. The core checks the file before the page draws anything: every key
//! the screens use must be there and not empty, so a missing string is found
//! when the page starts, not when a reader reaches the screen.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The keys that end in a number or a name, which the code builds.
pub const KINDS: [&str; 5] = ["jump", "lift", "general", "iso", "rest"];

/// Every string the screens use, by key. `{name}` marks a value the core fills.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Strings(pub BTreeMap<String, String>);

impl Strings {
    /// The string for a key. A missing key shows as `[key]` so it is seen.
    pub fn t(&self, key: &str) -> String {
        match self.0.get(key) {
            Some(s) => s.clone(),
            None => format!("[{key}]"),
        }
    }

    /// The string for a key with each `{name}` replaced by its value.
    pub fn f(&self, key: &str, values: &[(&str, &str)]) -> String {
        fill(&self.t(key), values)
    }

    pub fn has(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
}

/// Replaces each `{name}` in a template with its value.
pub fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (name, value) in values {
        out = out.replace(&format!("{{{name}}}"), value);
    }
    out
}

/// Keys the code builds from a number or a name, listed so the file is checked
/// for them too.
pub fn built_keys() -> Vec<String> {
    let mut keys = Vec::new();
    for n in 0..7 {
        keys.push(format!("dow_{n}"));
        keys.push(format!("dow_long_{n}"));
    }
    for n in 1..=12 {
        keys.push(format!("month_{n}"));
    }
    for k in KINDS {
        keys.push(format!("kind_{k}"));
    }
    for n in 1..=3 {
        keys.push(format!("own_done_{n}"));
        keys.push(format!("rival_first_{n}"));
        keys.push(format!("rival_heavy_{n}"));
    }
    for c in crate::program::CATEGORIES {
        keys.push(format!("cat_{c}"));
    }
    keys
}

/// Why a strings file cannot be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringsError {
    NotJson(String),
    Empty(String),
    Missing(String),
}

impl StringsError {
    /// The message the page shows. Wording that depends on the rule stays
    /// beside the rule.
    pub fn message(&self) -> String {
        match self {
            StringsError::NotJson(detail) => {
                format!("The strings file could not be read: {detail}")
            }
            StringsError::Empty(field) => format!("The strings file has an empty \"{field}\"."),
            StringsError::Missing(field) => format!("The strings file has no \"{field}\"."),
        }
    }
}

/// Parses and checks the strings file. A value that is only whitespace counts
/// as empty, because the page would draw nothing. `required` lists the keys
/// the screens use.
pub fn parse_strings(json: &str, required: &[String]) -> Result<Strings, StringsError> {
    let raw: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(json).map_err(|e| StringsError::NotJson(e.to_string()))?;
    let mut map = BTreeMap::new();
    for (key, value) in raw {
        // Keys starting with `_` hold notes for people.
        if key.starts_with('_') {
            continue;
        }
        let text = value
            .as_str()
            .ok_or_else(|| StringsError::NotJson(format!("\"{key}\" is not text")))?;
        if text.trim().is_empty() {
            return Err(StringsError::Empty(key));
        }
        map.insert(key, text.to_string());
    }
    for key in required {
        if !map.contains_key(key) {
            return Err(StringsError::Missing(key.clone()));
        }
    }
    Ok(Strings(map))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_complete_file_is_accepted() {
        let s = parse_strings(r#"{"title": "T", "intro": "I"}"#, &keys(&["title"])).unwrap();
        assert_eq!(s.t("title"), "T");
        assert_eq!(s.t("intro"), "I");
    }

    #[test]
    fn an_empty_value_is_rejected_by_name() {
        let e = parse_strings(r#"{"title": "  "}"#, &keys(&["title"])).unwrap_err();
        assert_eq!(e, StringsError::Empty("title".into()));
        assert_eq!(e.message(), "The strings file has an empty \"title\".");
    }

    #[test]
    fn a_missing_key_is_rejected_by_name() {
        let e = parse_strings(r#"{"title": "T"}"#, &keys(&["title", "intro"])).unwrap_err();
        assert_eq!(e, StringsError::Missing("intro".into()));
        assert_eq!(e.message(), "The strings file has no \"intro\".");
    }

    #[test]
    fn notes_for_people_and_non_text_values_are_handled() {
        let s = parse_strings(r#"{"_note": "for people", "title": "T"}"#, &[]).unwrap();
        assert!(!s.has("_note"));
        assert!(matches!(
            parse_strings(r#"{"title": 3}"#, &[]),
            Err(StringsError::NotJson(_))
        ));
    }

    #[test]
    fn text_that_is_not_json_is_rejected() {
        assert!(matches!(
            parse_strings("title: T", &[]),
            Err(StringsError::NotJson(_))
        ));
    }

    #[test]
    fn fill_replaces_each_named_value_and_a_missing_key_shows_in_brackets() {
        let s = parse_strings(r#"{"a": "{x} and {y} and {x}"}"#, &[]).unwrap();
        assert_eq!(s.f("a", &[("x", "1"), ("y", "2")]), "1 and 2 and 1");
        assert_eq!(s.t("nope"), "[nope]");
    }
}
