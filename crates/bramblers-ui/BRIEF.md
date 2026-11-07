# BRIEF — bbl-ui
<!-- Project brief (French). Code, comments, commits and README are in English. -->

Rev. 1 — 2026-10-16. Deuxieme fondation commune de Blackberrylabs, apres `hostrules`. **Validee le 2026-10-16** (U1 : anglais par defaut partout).

## 1. Objet

**`bbl-ui` : le visage commun des outils Blackberrylabs — la palette, les briques d'interface,
les reglages d'interface et les textes traduits, en une crate que chaque app utilise au lieu
de les recopier.**

Trois apps iced existent (Cleanator, MCProster, Permitt) et portent trois copies du meme
code : palette (`hex`, `good`, `alert`, `neutral`, `amber` depuis Permitt), briques `card` /
`inner` / `swatch` / `pill` / `small_button`, carte KPI, selecteur EN·FR, pied de fenetre,
module `settings` (langue + theme persistes dans `~/.config/<app>/settings.json`, variables
`<APP>_LANG` / `<APP>_THEME`), module `i18n` (`Lang`, `Strings`, locales JSON embarquees, test
« memes cles en EN et FR »). La regle du 16/10 dit : la fondation s'extrait au troisieme outil.
Nous y sommes, et la tour de controle serait la quatrieme copie.

## 2. Ce qui sort des apps → `bbl-ui`

| Dans les apps | Devient | Remarque |
|---|---|---|
| `hex`, `good(dark)`, `alert(dark)`, `amber(dark)`, `neutral()`, `unknown()`, `contrast_text` (Cleanator `treemap::contrast_text`), `blend` | `bbl_ui::palette` | Les valeurs exactes de Cleanator : 008300 / 2fa32f, eb6834 / d95926, 7a869a. `amber` (b98400 / d49a1a) et `muted` (9aa3b2) rejoignent la palette commune. |
| `card`, `inner`, `swatch`, `pill`, `small_button` | `bbl_ui::widgets` | Generiques sur le type de message de l'app (`<'a, M>`), memes paddings et rayons (14 / 10 ; 6·10 / 6 ; 2·8 / 8). |
| `kpi(label, value, sub, dot)`, `lang_picker(current, on_pick)`, `footer(status, lang_picker, theme_button)` | `bbl_ui::widgets` | Les trois apps les ecrivent a l'identique. |
| `settings.rs` | `bbl_ui::settings::Prefs<X>` | Langue + theme communs ; `X` = les reglages propres a l'app (MCProster : `ask_before_probe` ; Permitt : `read_shell` ; Cleanator : rien), flatten dans le meme JSON. Chemin `~/.config/<app>/settings.json`, variables `<APP>_LANG` / `<APP>_THEME` derivees du nom passe a `Prefs::load("permitt")`. **Anglais par defaut**, francais sur choix (regle de la maison). |
| `i18n.rs` | `bbl_ui::i18n::{Lang, Strings}` | `Strings::load(en: &str, fr: &str, lang)` : chaque app garde ses `locales/*.json` et les embarque elle-meme ; la crate fournit le chargeur, `t(key, args)` et `same_keys(en, fr) -> Result<(), String>` pour le test. |

Reste dans chaque app : toutes ses vues, ses messages, son chargement, le treemap de
Cleanator, les locales, le `main`.

## 3. Regles

- **R1 Aucun changement visible** apres migration : memes pixels a l'oeil sur les captures,
  memes tests verts, memes fichiers de reglages (le JSON existant se relit tel quel).
  Exception assumee, decision U1 ci-dessous.
- **R2 Pas de logique metier** : la crate ne connait ni serveurs, ni permissions, ni regles de
  nettoyage. Elle recoit des textes et des couleurs.
- **R3 Une seule version d'iced** (`=0.13.1`), fixee par la crate ; les apps n'en declarent pas
  une autre. Les features restent aux apps (Cleanator : canvas, image).
- **R4 Rien d'ecrit** hors `~/.config/<app>/settings.json`, et seulement par `Prefs::save`.
- **R5 Publication** : depot `blackberrylabs/bbl-ui`, MIT, dependance git par tag ; crates.io
  plus tard, comme hostrules.
- **R6 Anglais partout** (code, doc, README) ; BRIEF, PLAN, JOURNAL, CLAUDE.md en francais.

## 4. API visee (0.1)

```rust
bbl_ui::palette::{hex, good, alert, amber, neutral, muted, contrast_text, blend}
bbl_ui::widgets::{card, inner, swatch, pill, small_button, kpi, lang_picker, footer}
bbl_ui::settings::{Prefs<X>}        // load(app) · save() · initial_lang() · initial_dark() · extra: X
bbl_ui::i18n::{Lang, Strings}       // Lang::from_code, code ; Strings::load(en, fr, lang), t(key, args) ; same_keys(en, fr)
```

## 5. Etapes

0. BRIEF valide ; depot local, identite noreply avant le premier commit, `.gitignore`, LICENSE,
   README.
1. Crate : palette, widgets, settings, i18n ; tests unitaires (couleurs, `Prefs` lit un JSON
   d'une app existante, `same_keys`) ; clippy.
2. Migration **Permitt** (la plus fraiche), puis **MCProster**, puis **Cleanator** : tests
   verts, captures comparees, commit par app. Pas de changement de version ni de release :
   rien ne change pour l'utilisateur ; une entree « Unreleased » dans chaque CHANGELOG.
3. Tag `v0.1.0` — sur accord. La tour de controle demarre dessus.

## 6. Hors perimetre

Un theme a soi (on garde `Theme::Light` / `Theme::Dark` d'iced), des composants complexes
(tableaux, arbres), le treemap, la publication crates.io en 0.1.

## 7. Decisions

- **U0 2026-10-16** — Nom `bbl-ui` (crates.io libre ; `blackberry-ui` ecarte : collision avec la
  marque BlackBerry).
- **U1 2026-10-16** — Validee : anglais par defaut partout. Cleanator demarre aujourd'hui dans la langue du systeme ;
  MCProster et Permitt en anglais. La crate applique la regle de la maison (anglais par
  defaut) aux trois : seul changement visible, au premier lancement, pour un utilisateur
  francophone de Cleanator qui n'a jamais touche au selecteur. Je recommande d'aligner.
