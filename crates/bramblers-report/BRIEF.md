# BRIEF — bbl-report
<!-- Project brief (French). Code, comments, commits and README are in English. -->

Rev. 1 — 2026-10-16. Troisieme fondation commune de Blackberrylabs, apres `hostrules` (le
coeur) et `bbl-ui` (le visage) : **le contrat**. Decoule du BRIEF de la tour (section 4,
valide le 2026-10-16, decisions T3 et T4).

## 1. Objet

**`bbl-report` : les types Rust du rapport `bbl-report/1` qu'un module Blackberrylabs produit
avec `<module> report --json`, et que la tour lit — serialisation, validation, echelle de
gravite en mots, test de conformite.**

Une crate minuscule, sans logique metier, sans interface, sans lecture de fichiers : elle
dit la forme du rapport, rien d'autre. Elle est la seule chose que la tour et les modules
partagent. Ajouter un module a la famille = produire ce rapport.

## 2. Le rapport (rappel du contrat, BRIEF de la tour § 4.2)

Un document JSON `bbl-report/1` : `module` (id, nom, version, verbe, accroche, binaires
`app` / `cli`, page d'accueil), `generated_at`, `duration_ms`, `summary` (2 a 4 chiffres,
ceux des cartes KPI, avec un ton), `findings` (id stable, gravite 1–4, kind, titre, sujet,
detail, preuves, actions affichees, `open` = l'app au bon endroit), `notes`, `errors`.

Quatre gravites communes, **definies en mots dans la crate** (T3) :

| Valeur | Nom | Sens | Exemples |
|---|---|---|---|
| 1 | `act-now` | agir maintenant | perte de donnees possible, secret en clair, contournement dans un shell |
| 2 | `review` | a regarder | agit sans demander, acces large, gros volume recuperable |
| 3 | `good-to-know` | bon a savoir | doublon, version qui differe, liste de refus seule |
| 4 | `info` | information | desactive, pas en cache, non lisible |

Chaque module mappe ses propres constats sur cette echelle (le mapping est ecrit dans son
CHANGELOG) ; la tour ne reinterprete pas.

## 3. Regles

- **R1 Aucune logique metier, aucune lecture, aucune ecriture, aucun reseau** : des types,
  `to_json` / `from_json`, `validate`, et un helper d'horodatage UTC. Dependances : `serde`,
  `serde_json`, rien d'autre.
- **R2 Compatibilite avant** : un lecteur ignore les champs inconnus (jamais
  `deny_unknown_fields`) ; les listes et les champs facultatifs ont une valeur par defaut.
  Un champ ajoute = meme version ; un champ retire ou qui change de sens = `bbl-report/2`.
  `peek_schema(json)` lit la version avant tout parsing type.
- **R3 La validation est dans la crate**, pas dans la tour : `Report::validate()` rend la
  liste complete des defauts (schema connu, `module.id` en slug, ids de constats uniques et
  non vides, titres non vides, `summary` de 1 a 4 entrees, `open.app` non vide s'il est la,
  `generated_at` en RFC 3339 UTC). `conformance::check(json)` enchaine parsing et validation ;
  chaque module l'execute dans ses tests sur sa propre sortie.
- **R4 Rien d'inedit** : la crate ne masque rien et ne filtre rien — c'est au module de ne
  mettre dans le rapport que ce qu'il affiche deja (secrets masques en amont). La crate le
  rappelle dans sa doc, elle ne peut pas le verifier.
- **R5 Publication** : depot `blackberrylabs/bbl-report`, MIT, dependance git par tag ;
  crates.io plus tard, avec les deux autres fondations.
- **R6 Anglais partout** (code, doc, README) ; BRIEF, PLAN, JOURNAL, CLAUDE.md en francais.

## 4. API visee (0.1)

```rust
bbl_report::SCHEMA                                  // "bbl-report/1"
bbl_report::{Report, Module, Summary, Tone, Finding, Severity, Subject, SubjectKind,
             Evidence, Action, ActionKind, Open}
Report::new(module) · to_json() · from_json(&str) · validate() -> Result<(), Vec<String>>
Severity::{ActNow, Review, GoodToKnow, Info}        // serialise en 1..4 ; name(), meaning()
Tone::{Neutral, Good, Amber, Alert}                 // les noms de la palette bbl-ui
SubjectKind::{Host, Server, Path, File, Setting, Other}
ActionKind::{Hint, OpenFile, OpenUrl, Command}      // affichees, jamais executees
bbl_report::now_utc()                               // horodatage RFC 3339 sans dependance
bbl_report::peek_schema(&str) -> Option<String>
bbl_report::conformance::check(&str) -> Result<Report, Vec<String>>
```

## 5. Etapes

0. BRIEF valide ; depot local, identite noreply avant le premier commit, `.gitignore`,
   LICENSE, README.
1. Crate : types, gravite, validation, conformite, `now_utc` ; tests (aller-retour JSON,
   champs inconnus ignores, schema 2 reconnu par `peek_schema` et refuse par `check`, chaque
   regle de validation declenchee une fois, horodatage correct sur des dates connues) ;
   promesse « aucun appel `std::fs` ni `std::net` dans le code » ; clippy propre.
2. Depot `blackberrylabs/bbl-report`, tag v0.1.0 — sur accord. Puis `report --json` dans
   Permitt, MCProster, Cleanator (BRIEF de la tour, etape 0).

## 6. Hors perimetre

L'ecran, le cache, la decouverte des modules (tour) ; la lecture des hotes (`hostrules`) ;
l'interface (`bbl-ui`) ; tout mapping de gravite (chaque module).

## 7. Decisions

- Les decisions de forme sont celles de la tour : **T3** (quatre gravites communes, mappees
  par les modules) et **T4** (`bbl-report` crate a part), prises le 2026-10-16.
- **P1 2026-10-16** — Gravite serialisee en **nombre** (1–4), nommee en mots dans la crate :
  le nombre trie, le mot explique ; les deux restent stables.
- **P2 2026-10-16** — Pas de dependance de date (`chrono`, `time`) : un horodatage UTC se
  calcule en vingt lignes a partir de `SystemTime`, teste sur des dates connues.
