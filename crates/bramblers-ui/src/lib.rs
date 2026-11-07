//! `bbl-ui`: the shared face of the Blackberrylabs desktop tools (Cleanator, MCProster,
//! Permitt, the control tower). What the three first apps carried as three copies now lives
//! here, and nothing else:
//!
//! - [`palette`]: Cleanator's colours — green for "fine", orange for "look at this", grey
//!   for information, amber for "a model decides" — and two helpers (`contrast_text`, `blend`);
//! - [`widgets`]: the building blocks every window is made of (`card`, `inner`, `swatch`,
//!   `pill`, `small_button`, `kpi`, `lang_picker`, `footer`), generic over the app's message;
//! - [`settings`]: interface preferences remembered between launches (`Prefs<X>`): language
//!   and theme for everyone, `X` for what one app adds; English by default, whatever the
//!   system says;
//! - [`i18n`]: `Lang` and `Strings` — every label comes from the app's own `locales/*.json`,
//!   English as the fallback, plus the `same_keys` check for tests.
//!
//! No business logic, no network, and the only file ever written is `~/.config/<app>/settings.json`
//! through [`settings::Prefs::save`]. iced is pinned to `0.13.1` here; apps do not pin another.
#![forbid(unsafe_code)]

pub mod i18n;
pub mod palette;
pub mod settings;
pub mod widgets;

pub use i18n::{Lang, Strings};
pub use settings::Prefs;
