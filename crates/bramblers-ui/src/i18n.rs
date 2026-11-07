//! UI strings. Nothing is hard-coded in the views: every label comes from the app's
//! `locales/<lang>.json`, embedded by the app, English as the fallback.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    En,
    Fr,
}

impl Lang {
    /// `"en"` / `"fr"` (case-insensitive; a longer tag such as `fr_FR.UTF-8` works).
    pub fn from_code(code: &str) -> Option<Lang> {
        let c = code.trim().to_ascii_lowercase();
        if c.starts_with("fr") {
            Some(Lang::Fr)
        } else if c.starts_with("en") {
            Some(Lang::En)
        } else {
            None
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Fr => "fr",
        }
    }

    pub const ALL: [Lang; 2] = [Lang::En, Lang::Fr];

    /// The label of the picker button.
    pub fn label(self) -> &'static str {
        match self {
            Lang::En => "EN",
            Lang::Fr => "FR",
        }
    }
}

pub struct Strings {
    en: HashMap<String, String>,
    current: HashMap<String, String>,
}

impl Strings {
    /// Build from the two embedded JSON texts (flat `"key": "text"` objects).
    pub fn load(en: &str, fr: &str, lang: Lang) -> Strings {
        let en_map = parse(en);
        let current = match lang {
            Lang::En => en_map.clone(),
            Lang::Fr => parse(fr),
        };
        Strings {
            en: en_map,
            current,
        }
    }

    /// The string for `key`, with `{name}` placeholders replaced; English when the current
    /// language lacks it; `⟨key⟩` when both do, so a missing label is seen, not hidden.
    pub fn t(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut s = self
            .current
            .get(key)
            .or_else(|| self.en.get(key))
            .cloned()
            .unwrap_or_else(|| format!("⟨{key}⟩"));
        for (k, v) in args {
            s = s.replace(&format!("{{{k}}}"), v);
        }
        s
    }
}

fn parse(text: &str) -> HashMap<String, String> {
    serde_json::from_str::<HashMap<String, String>>(text).unwrap_or_default()
}

/// Both locale files carry exactly the same keys; the error names what differs.
pub fn same_keys(en: &str, fr: &str) -> Result<(), String> {
    let en = parse(en);
    let fr = parse(fr);
    let mut missing: Vec<&String> = en.keys().filter(|k| !fr.contains_key(*k)).collect();
    missing.sort();
    let mut extra: Vec<&String> = fr.keys().filter(|k| !en.contains_key(*k)).collect();
    extra.sort();
    if missing.is_empty() && extra.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "missing in fr: {missing:?}; extra in fr: {extra:?}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitutes_and_falls_back() {
        let s = Strings::load(
            r#"{"a": "{n} server(s)", "only_en": "x"}"#,
            r#"{"a": "{n} serveur(s)"}"#,
            Lang::Fr,
        );
        assert_eq!(s.t("a", &[("n", "3")]), "3 serveur(s)");
        assert_eq!(s.t("only_en", &[]), "x");
        assert_eq!(s.t("none", &[]), "⟨none⟩");
    }

    #[test]
    fn same_keys_names_the_difference() {
        assert!(same_keys(r#"{"a":"1"}"#, r#"{"a":"2"}"#).is_ok());
        let e = same_keys(r#"{"a":"1","b":"1"}"#, r#"{"a":"2","c":"3"}"#).unwrap_err();
        assert!(e.contains("\"b\"") && e.contains("\"c\""), "{e}");
    }

    #[test]
    fn codes() {
        assert_eq!(Lang::from_code("fr_FR.UTF-8"), Some(Lang::Fr));
        assert_eq!(Lang::from_code("EN"), Some(Lang::En));
        assert_eq!(Lang::from_code("de"), None);
    }
}
