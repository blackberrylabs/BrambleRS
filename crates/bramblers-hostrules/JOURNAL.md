# JOURNAL — bbl-hostrules
<!-- Project journal (French), newest first. -->

---

## 2026-10-26, passation à OpenCode

- `AGENTS.md` ajouté : source unique des règles pour OpenCode et tout autre agent. OpenCode ne charge pas `CLAUDE.md` quand `AGENTS.md` existe (documentation d'OpenCode lue le 2026-10-26). `CLAUDE.md` devient un pointeur (`@AGENTS.md`) avec deux lignes propres à Claude. Le bloc commun d'`AGENTS.md` est identique dans les douze dépôts de la série.
- `opencode.json` ajouté : tout ce qui pousse, tague, supprime, installe ou touche une minuterie demande confirmation ; les fichiers `.env` sont refusés ; `BRIEF.md` et `LICENSE` demandent confirmation à l'édition.
- PLAN : étape 6 cochée (dépôt renommé, tag v0.3.0, Permitt et MCProster sur v0.3.0, constatés dans leurs `Cargo.toml` et par `git tag`). Étape 7 ajoutée pour les versions 0.4.0 et 0.5.0, relevées dans le CHANGELOG.
- Preuve : état de la série daté du 2026-10-26 dans `Logiciels/PASSATION.md`. Demandé par Alexandre le 2026-10-26.

---

## 2026-10-16, tard — bbl-hostrules

- Question d'Alexandre apres la naissance de `bbl-report` : « pour faire propre, on aurait pu
  mettre bbl-hostrules aussi ? ». Recommandation de garder `hostrules` (nom du metier, crate
  utilisable seule) ; il tranche l'inverse : « Je veux ca : bbl-hostrules ». Regularite de la
  famille d'abord — et c'est le moment ou ca ne coute rien (deux consommateurs, tous a nous,
  pas encore sur crates.io).
- 0.3.0 : seul le nom change (`use bbl_hostrules::…`). Dossier local renomme, depot renomme
  sur GitHub par Alexandre, Permitt et MCProster migrent dans la foulee.

---

## 2026-10-16, deux consommateurs

- Permitt 0.1.0 publie sur hostrules 0.2.0 (`HostRule<PermLocation, PermHost>`), MCProster 0.2.1 sur
  0.1.0. La condition posee dans le BRIEF pour crates.io — deux outils, une version d'API
  tenue — est remplie ; publication a decider.

## 2026-10-16, 0.2.0 le soir meme

- Permitt (schema des regles D11) a besoin de champs par hote — defauts, sens des reglages,
  precedence — la ou MCProster n'avait besoin que de champs par emplacement. Deuxieme parametre
  `H` avec defaut `NoExt` : MCProster compile sans changer une ligne. C'est le premier besoin
  qu'un seul outil ne montrait pas ; l'API a tenu.

## 2026-10-16, publie

- Commit 1, depot public, tag v0.1.0 le soir meme. MCProster 0.2.1 construit dessus depuis
  GitHub, passage reel identique. Permitt peut demarrer.

## 2026-10-16, extraction

- Faite dans la soiree : `rules` (generique `HostRule<X>`, extension flattenee, `NoExt` pour
  les tests), `env`, `read`, `secrets` copies tels quels, `resolve` sorti d'`inventory.rs`
  avec `project_roots`. 22 tests, dont deux sur les promesses (aucune ecriture, aucun
  reseau) qui lisent le code source de la crate — le premier a d'abord attrape les
  fixtures des tests unitaires, exclues depuis.
- MCProster migre dans le cloud en une passe : `hosts.rs` reduit a `McpLocation` + regles
  embarquees, trois modules supprimes, re-exports `env`/`read`/`secrets` pour garder les
  chemins. Sorties `--json` identiques sauf `hosts` (`hints`). Clippy propre.

## 2026-10-16, naissance

- Troisieme outil Blackberrylabs (Permitt) → regle de la convention du 16/10 : la fondation
  commune s'extrait au troisieme outil, pas avant. Option C choisie par Alexandre : le coeur
  maintenant, l'UI apres Permitt.
- Perimetre : regles d'hotes sourcees, env, lecteurs, masquage, resolution d'emplacements.
  Les regles elles-memes restent a chaque outil. Nom verifie libre sur crates.io.
