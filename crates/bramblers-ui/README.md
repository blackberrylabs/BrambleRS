# bbl-ui

**The shared face of the Blackberrylabs desktop tools.**

[Cleanator](https://github.com/blackberrylabs/cleanator), [MCProster](https://github.com/blackberrylabs/mcproster),
[Permitt](https://github.com/blackberrylabs/permitt) and [Linstruct](https://github.com/blackberrylabs/linstruct)
are built on the same principle —
show, explain, never act in the user's place — and look the same: the same palette, the same
cards and pills, the same footer with `EN · FR` and the theme toggle, the same way of
remembering a language. This crate holds that common face, so each app keeps only what is
its own.

## What is in it

- **`palette`** — Cleanator's colours: green for "fine", orange for "look at this", grey for
  information, amber for "a model decides"; `contrast_text` and `blend`.
- **`widgets`** — the building blocks every window is made of, generic over the app's
  message type: `card`, `inner`, `swatch`, `pill`, `small_button`, `kpi`, `lang_picker`,
  `footer`.
- **`settings`** — `Prefs<X>`: language and theme remembered between launches in
  `~/.config/<app>/settings.json`, plus `X` for what one app adds. **English by default**,
  whatever the system language; `<APP>_LANG` / `<APP>_THEME` override one launch.
- **`i18n`** — `Lang` and `Strings`: every label comes from the app's own `locales/*.json`
  (embedded by the app), English as the fallback; `same_keys` for the test that keeps both
  files in step.

No business logic, no network. The only file this crate ever writes is the app's
`settings.json`, through `Prefs::save`. iced is pinned to `0.13.1` here.

## Part of Blackberrylabs

One of the three shared crates of the [Blackberrylabs](https://github.com/blackberrylabs)
family — with [`bbl-hostrules`](https://github.com/blackberrylabs/bbl-hostrules) (the files of AI hosts) and [`bbl-report`](https://github.com/blackberrylabs/bbl-report) (the tower's contract) — behind the control tower and its modules (Cleanator, MCProster,
Permitt). A new module starts from the [module template](https://github.com/blackberrylabs/template);
the procedure is [`docs/new-module.md`](https://github.com/blackberrylabs/blackberrylabs/blob/main/docs/new-module.md)
in the tower.

## Use

```toml
[dependencies]
bbl-ui = { git = "https://github.com/blackberrylabs/bbl-ui", tag = "v0.1.0" }
```

```rust
use bbl_ui::{Lang, Prefs, Strings};
use bbl_ui::widgets::{card, kpi, pill, footer};
use bbl_ui::palette::{good, alert};

const EN: &str = include_str!("../locales/en.json");
const FR: &str = include_str!("../locales/fr.json");

let mut prefs: Prefs<()> = Prefs::load("myapp");
let lang = prefs.initial_lang();
let strings = Strings::load(EN, FR, lang);
// … views use card(...), kpi(...), pill(...), footer(status, lang, Message::SetLang, label, Message::ToggleTheme)
prefs.remember(lang, false);
prefs.save();
```

Status: **0.1.0**, used by Cleanator, MCProster, Permitt and Linstruct. Not on crates.io.

By [Alexandre Do](https://github.com/blackberrylabs). MIT — see [LICENSE](LICENSE).
