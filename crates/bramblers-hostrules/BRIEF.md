# BRIEF — bbl-hostrules (ne `hostrules`)
<!-- Project brief (French). Code, comments, commits and README are in English. -->

Rev. 1 — 2026-10-16. Decision D6 de Permitt : le troisieme outil extrait le coeur commun.
**Validee le 2026-10-16** (H2 et `resolve` dans la crate acceptes).

## 1. Objet

**`bbl-hostrules` (jusqu'au 16/10 au soir : `hostrules`) : une crate Rust qui sait ou chaque assistant IA range ses fichiers sur une
machine, et comment les lire sans rien ecrire — avec la source de chaque affirmation.**

Une regle d'hote = un fichier TOML : identite de l'hote, emplacements par OS et par portee
(user / project / system), format (JSON, JSONC, TOML, YAML), cles a lire, et pour chaque
emplacement la **source** (documentation, code source ou observe ; URL, date, citation).
La crate charge, valide (refuse une regle sans source), resout les chemins sur une machine
donnee (home, OS, variables, racines de projet), lit les fichiers vers `serde_json::Value`,
descend dans les cles (`dig`, jokers, points echappes), et masque les secrets.

## 2. Ce qui sort de MCProster (`crates/core`) → `hostrules`

| Module MCProster | Devient | Changement |
|---|---|---|
| `hosts.rs` : `HostRule`, `Location`, `Source`, `SourceKind`, `Scope`, `Format`, `PathSpec`, `OneOrMany`, `NotRead`, `Rules::load`, `validate` | `hostrules::rules` | Les champs propres a MCP (`servers_key`, `entry`, `name_key`, `inputs_key`, `map`, `command_is_array`, `remove_hint`, `list_hint`, `role`) deviennent une **extension typee** : `HostRule<X>` ou `X` est la charge utile de l'outil (`McpLocation` pour MCProster, `PermissionLocation` pour Permitt). La validation commune (id, nom, chemin, source) reste dans la crate ; chaque outil ajoute la sienne. `EMBEDDED` (`include_str!`) reste dans chaque outil : les regles sont a lui. |
| `env.rs` : `Env{home, os, vars, path, roots}`, `expand` (`~`, `${VAR}`, `%VAR%`, `$VAR`), `which` | `hostrules::env` | Inchange. |
| `read.rs` : lecture JSON/JSONC/TOML/YAML, `dig`, `split_path`, `get`, `line_of` | `hostrules::read` | Inchange. |
| `secrets.rs` : `classify`, `Masked{shown, kind}`, references, `scrub_text` | `hostrules::secrets` | Inchange. |
| `inventory.rs` : `resolve` (chemins par OS/portee/`project_key`), globs, `FileStatus` | `hostrules::resolve` | La resolution d'emplacements en fichiers concrets devient commune ; **tout le reste** (`entries`, `merge`, identites de serveurs, constats, `package_cached`) reste a MCProster. |

Reste a MCProster : `model.rs`, le gros d'`inventory.rs`, `probe.rs`, `cost.rs`,
`actions.rs`, `hosts/*.toml`, les tests d'inventaire et de sonde. MCProster garde ses 19
regles ; Permitt ecrit les siennes, avec le meme schema de base.

## 3. Regles

- **R1 Lecture seule** : la crate n'ecrit jamais un fichier ; aucune API d'ecriture.
- **R2 Aucun reseau** : aucune dependance reseau, aucun acces.
- **R3 Une regle sans source est refusee** a la validation, comme aujourd'hui.
- **R4 Pas de regles embarquees dans la crate** : les `hosts/*.toml` appartiennent aux
  outils ; la crate fournit le schema, le chargeur et la validation. (Un depot commun de
  regles d'identite d'hotes — nom, editeur, type — pourra venir ensuite si deux outils le
  dupliquent vraiment.)
- **R5 Pas d'interface** : ni iced, ni i18n. L'UI commune est un autre chantier, apres Permitt.
- **R6 Compatibilite** : MCProster 0.2.1 migre dessus **sans changement visible** (memes
  sorties, memes 33 tests verts, meme `--json`).
- **R7 Publication** : depot `blackberrylabs/hostrules`, MIT, `#![forbid(unsafe_code)]` ;
  d'abord dependance git (`hostrules = { git = … , tag = "v0.1.0" }`), publication sur
  crates.io quand deux outils l'utilisent et que l'API a tenu une version.
- **R8 Anglais partout** (code, doc, README) ; BRIEF, PLAN, JOURNAL, CLAUDE.md en francais.

## 4. API visee (0.1)

```rust
hostrules::rules::{HostRule<X>, Location<X>, Source, SourceKind, Scope, Format, PathSpec, OneOrMany, NotRead, Status, Rules<X>, RuleError}
hostrules::env::{Env, expand, which}
hostrules::read::{read_value, dig, split_path, get, line_of, ReadError}
hostrules::secrets::{classify, Masked, Kind, scrub_text}
hostrules::resolve::{resolve, Resolved{host_id, loc_idx, path, project}, FileStatus}
```

`Rules::<X>::load(files)` avec `X: DeserializeOwned + Validate` ; `Validate::validate(&self,
file, loc_idx) -> Result<(), String>` pour la part propre a l'outil.

## 5. Etapes

0. BRIEF valide. Depot local, identite noreply avant le premier commit, `.gitignore`,
   LICENSE, README minimal.
1. Extraction : copier les modules, generaliser `HostRule<X>`, tests unitaires deplaces
   (`expand`, `dig`, `classify`, validation), `cargo test`, `cargo clippy -- -D warnings`.
2. Migration MCProster : `McpLocation`, dependance git, suppression des modules deplaces,
   33 tests verts, `mcproster --json list` identique avant/apres sur le faux home et sur la
   machine d'Alexandre. Version 0.2.1, release — sur accord.
3. Tag `v0.1.0` de hostrules — sur accord. Permitt demarre dessus.

## 6. Hors perimetre

Interface commune, i18n, paquets (un outil = ses paquets), regles d'hotes partagees,
publication crates.io en 0.1.

## 7. Decisions

- **D6 Permitt, 2026-10-16** — extraction du coeur maintenant ; UI apres Permitt.
- **H1 2026-10-16** — nom `hostrules` (crates.io libre ; `host-rules`, `bbl-hosts` libres
  aussi) ; depot propre dans l'org.
- **H2 2026-10-16** — `HostRule<X>` generique plutot que deux schemas
  separes : un seul validateur de sources, un seul chargeur, chaque outil son extension.

- **H3 2026-10-16** — `resolve` (emplacements → fichiers concrets) dans la crate.
- **H4 2026-10-16** — Renommee **`bbl-hostrules`** (« Je veux ca : bbl-hostrules »). Regle posee
  dans `00_moi/CONVENTIONS.md` : toute fondation commune de la famille porte le prefixe
  `bbl-` (`bbl-hostrules`, `bbl-ui`, `bbl-report`) ; les outils gardent leur nom propre.
  Depot renomme sur GitHub (l'ancienne adresse redirige), dossier local `Logiciels/Bbl_hostrules`,
  version 0.3.0 : seul le nom change.
