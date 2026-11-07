//! `bramblers-ui`: the shared face of the BrambleRS desktop tools (Cleanator, MCProster,
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
//!   English as the fallback, plus the `same_keys` check for tests;
//! - [`open`]: one policy for opening a file or a link in the desktop session: files go to
//!   the default application, links must be http or https.
//!
//! No business logic, no network request of its own, and the only file ever written is
//! `~/.config/<app>/settings.json` through [`settings::Prefs::save`]. [`open`] launches the
//! desktop's opener for a file or an http(s) link. iced is pinned to `0.13.1` here; apps do
//! not pin another.
#![forbid(unsafe_code)]

pub mod i18n;
pub mod open;
pub mod palette;
pub mod settings;
pub mod widgets;

pub use i18n::{Lang, Strings};
pub use settings::Prefs;
