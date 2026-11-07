//! Interface preferences remembered between launches: language and theme for every app,
//! plus what one app adds (`X`). Stored in `$XDG_CONFIG_HOME/<app>/settings.json` (default
//! `~/.config/<app>/`) — the only file this crate ever writes.
//!
//! The language starts in **English** whatever the system language; French is a choice
//! the user makes once. `<APP>_LANG` / `<APP>_THEME` override for one launch.

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::i18n::Lang;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound(deserialize = "X: DeserializeOwned", serialize = "X: Serialize"))]
pub struct Prefs<X = ()> {
    #[serde(skip)]
    app: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    /// The app's own preferences, flattened into the same JSON object.
    #[serde(flatten)]
    pub extra: X,
}

impl<X: Default + Serialize + DeserializeOwned> Prefs<X> {
    /// `~/.config/<app>/settings.json`, or `None` when no home can be found.
    pub fn path_for(app: &str) -> Option<PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
        Some(base.join(app).join("settings.json"))
    }

    /// The saved preferences of `app` (its lowercase short name: `permitt`), or defaults
    /// when there is no file or it cannot be read.
    pub fn load(app: &str) -> Prefs<X> {
        let mut p: Prefs<X> = Self::path_for(app)
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_else(|| Prefs {
                app: String::new(),
                lang: None,
                theme: None,
                extra: X::default(),
            });
        p.app = app.to_string();
        p
    }

    pub fn app(&self) -> &str {
        &self.app
    }

    /// Best effort: a failure to save never bothers the user (the toggle still applies).
    pub fn save(&self) {
        let Some(p) = Self::path_for(&self.app) else {
            return;
        };
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(p, json + "\n");
        }
    }

    fn env(&self, suffix: &str) -> Option<String> {
        let name = format!(
            "{}_{suffix}",
            self.app.to_ascii_uppercase().replace('-', "_")
        );
        std::env::var(name).ok()
    }

    /// `<APP>_LANG`, then the saved preference, then English.
    pub fn initial_lang(&self) -> Lang {
        if let Some(l) = self.env("LANG").as_deref().and_then(Lang::from_code) {
            return l;
        }
        self.lang
            .as_deref()
            .and_then(Lang::from_code)
            .unwrap_or(Lang::En)
    }

    /// `<APP>_THEME`, then the saved preference, then light.
    pub fn initial_dark(&self) -> bool {
        match self.env("THEME").as_deref() {
            Some("dark") => return true,
            Some("light") => return false,
            _ => {}
        }
        self.theme.as_deref() == Some("dark")
    }

    /// Record the current language and theme (then `save`).
    pub fn remember(&mut self, lang: Lang, dark: bool) {
        self.lang = Some(lang.code().to_string());
        self.theme = Some(if dark { "dark" } else { "light" }.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
    struct Extra {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ask_before_probe: Option<bool>,
    }

    #[test]
    fn english_is_the_default_whatever_the_system_says() {
        std::env::remove_var("TESTAPP_LANG");
        let p: Prefs<()> = Prefs::load("testapp");
        assert_eq!(p.initial_lang(), Lang::En);
        assert!(!p.initial_dark());
    }

    #[test]
    fn an_existing_file_of_an_app_is_read_and_written_back_identically() {
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());
        let dir = tmp.path().join("mcproster");
        std::fs::create_dir_all(&dir).unwrap();
        let text =
            "{\n  \"lang\": \"fr\",\n  \"theme\": \"dark\",\n  \"ask_before_probe\": false\n}\n";
        std::fs::write(dir.join("settings.json"), text).unwrap();
        let p: Prefs<Extra> = Prefs::load("mcproster");
        assert_eq!(p.initial_lang(), Lang::Fr);
        assert!(p.initial_dark());
        assert_eq!(p.extra.ask_before_probe, Some(false));
        p.save();
        let back = std::fs::read_to_string(dir.join("settings.json")).unwrap();
        assert_eq!(back, text);
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn env_overrides_for_one_launch() {
        std::env::set_var("OTHERAPP_LANG", "fr");
        std::env::set_var("OTHERAPP_THEME", "dark");
        let p: Prefs<()> = Prefs::load("otherapp");
        assert_eq!(p.initial_lang(), Lang::Fr);
        assert!(p.initial_dark());
        std::env::remove_var("OTHERAPP_LANG");
        std::env::remove_var("OTHERAPP_THEME");
    }
}
