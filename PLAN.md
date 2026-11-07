# PLAN — BrambleRS (ex-Blackberrylabs)
<!-- Project plan (French). -->

**2026-11-07 : restructuration validée.** Le produit s'appelle BrambleRS. Les étapes 0 à 7 ci-dessous sont l'historique de la tour ; la suite est l'étape 8.

## Etape 0. Le contrat
- [x] BRIEF rev. 1 ; T0 nom, T1 tableau de bord « liste de constats », T2 rafraichissement — 2026-10-16
- [x] Validation du BRIEF (T3 gravites, T4 crate a part) — 2026-10-16
- [x] Crate `bbl-report` 0.1.0 : types, echelle de gravite en mots, validation, test de conformite ; depot, tag v0.1.0 — 2026-10-16
- [x] `report` dans Permitt, MCProster, Cleanator (mapping des constats sur les quatre gravites dans chaque CHANGELOG) ; `--select` dans les trois apps ; un commit par module (Permitt 4, MCProster 14, Cleanator 21) — 2026-10-16
- [x] `.gitignore`, LICENSE MIT, README, CHANGELOG ecrits ; depot git local et commit 1 avec le coeur — 2026-10-17

## Etape 1. Coeur et CLI (`bbl`, `bbl modules`, `bbl refresh`, `--json`) ; fixtures de rapports ; tests R2
- [x] `bbl-core` : fiches `modules/*.toml`, decouverte sur PATH, lancement avec limite de temps, rapports enregistres, assemblage et tri — 2026-10-17
- [x] `bbl` : liste, `refresh`, `modules`, `--json`, `--min-severity`, `--module` — 2026-10-17
- [x] Tests sur faux modules (les trois vrais rapports en fixtures) : module absent, rapport enregistre avec son age, R2 (rien hors `state/`), schema 2 et champs en plus, module en echec — 2026-10-17
- [x] Passage reel dans le cloud avec les trois vrais binaires — 2026-10-17
- [x] Commit 1 — 2026-10-17
## Etape 2. App (maquette validee avant le code)
- [x] Maquette HTML (en marche + premier lancement), validee d'un mot — 2026-10-17
- [x] `bbl-app` (iced, bbl-ui) : phrase, quatre gravites, cartes de modules, liste, detail, ouvrir dans le module ; rafraichissement en arriere-plan (T2) ; EN/FR ; clair/sombre — 2026-10-17
- [x] Captures docs/screenshots (clair, sombre) — 2026-10-17
- [x] Commit 2 (f4f780a) — 2026-10-17
## Etape 3. Paquets, depot, release 0.1.0 — sur accord ; README d'organisation
- [x] `packaging/build.sh`, `.desktop`, icone (radar + quatre gravites), metadonnees deb/rpm — 2026-10-17
- [x] Passage reel sur la machine d'Alexandre (binaires reconstruits dans ~/.local/bin) : Cleanator 22 constats en 5,1 s, MCProster 1, Permitt 12 ; 0 a agir, 17 a regarder, 7 bon a savoir, 11 informations — 2026-10-17
- [x] Depot `blackberrylabs/blackberrylabs`, tag v0.1.0, release avec les trois paquets + SHA256SUMS ; verifiee depuis le cloud : somme OK, `bbl 0.1.0`, le `.deb` publie lit les trois `.deb` publies des modules — 2026-10-17
- [x] Les trois modules publies avant : Permitt 0.2.0, MCProster 0.2.1, Cleanator 0.2.0 — 2026-10-17
- [x] README d'organisation publie dans `blackberrylabs/.github` (`profile/README.md`), visible sur github.com/blackberrylabs — 2026-10-17

**Etape 3 close le 2026-10-17. Blackberrylabs 0.1.0 est publiee, avec ses trois modules.**

## Etape 4. Integration facile (decision T5, 2026-10-17)
- [x] `docs/new-module.md` : template, contrat, gravites et mappings existants, `--select`, fiche, avant la release — 2026-10-17
- [x] Gabarit `blackberrylabs/template` (`Logiciels/Template`) : core/cli/app, promesses testees, `report` conforme, `rename.sh` — 2026-10-17
- [x] Mise a jour de toute la doc de la famille : README des trois modules (« Part of Blackberrylabs », statut), CLAUDE.md et CONTRIBUTING (fondations, mapping), README des trois fondations, CONVENTIONS et PROJETS 00_moi — 2026-10-17
- [x] Commits et pushes (docs seulement) — sur accord — les commits « docs: » des modules sont dans `origin/main` (constate le 2026-10-26)

## Etape 5. Tour 0.3 — « un logiciel, pas un lanceur » (decision T6, 2026-10-17) — faite, 0.3.0 et 0.3.1 publiees le 2026-10-17

Constat d'Alexandre au premier usage : un clic sur un module ouvre l'application du module
dans une autre fenetre ; « on a l'impression d'avoir un logiciel, mais il en ouvre un
autre ». Attendu : rester dans la tour, filtree sur le module — pas l'ecran complet du
module non plus.

- [x] Un clic sur la carte d'un module filtre la tour sur ce module et montre son resume
      dans le volet : les quatre chiffres, l'age du dernier rapport, ses constats.
- [x] L'ouverture de l'application du module devient un bouton secondaire, nomme sans
      ambiguite : « Open in <Module> ↗ » (fenetre separee) ; « Get it » inchange pour un
      module absent ; « Refresh » reste dans la tour (relance en arriere-plan).
- [x] Les cinq modules presents dans les fiches embarquees (Billfold ajoute : 0.1.0 publie
      le 2026-10-17 ; `report` conforme ; tester sur la machine d'Alexandre).
- [x] Coherence publique de la famille (audit SEO du 2026-10-17, `Logiciels/audit.md`) :
      phrase de famille validee — « Read-only tools that show what AI coding agents do to
      your machine — and one screen that gathers them. » — portee par la description
      d'organisation, le README de la tour et la section « Part of Blackberrylabs » de
      chaque module ; descriptions et topics par `gh repo edit` depuis un fichier du gabarit
      (`packaging/github.toml`) et une ligne du rituel de release ; capture en tete de
      chaque README ; Billfold dans les deux hubs ; release 0.2.0 de la tour nettoyee des
      assets 0.1.0.
- [x] Ecarte : Discussions auto-posees, wiki, une page GitHub Pages par depot, crates.io
      pour bbl-ui et bbl-report (bbl-hostrules seule a du sens, plus tard).
- [x] « Get it » ouvre le paquet exact de la machine (`/etc/os-release`, `latest` dans la
      fiche) ; le navigateur telecharge, la tour ne touche pas au reseau — 2026-10-17
- [x] Paquet complet `blackberrylabs-full` (tour + cinq modules, binaires publies ramasses
      par `packaging/bundle.sh`), modules cherches a cote du binaire, entrees de menu des
      modules masquees (0.3.1) ; tour legere conservee — 2026-10-17
- [ ] Captures d'ecran de Billfold et Linstruct dans leur README (a faire sur l'ecran
      d'Alexandre) ; aperçus sociaux GitHub des six depots (reglage manuel).
      Au 2026-10-26 : Billfold est fait (`docs/screenshots/light.png`) ; il manque Linstruct et Lastrun. Les aperçus sociaux restent un réglage manuel (Settings, Social preview).

## Etape 6. Sixieme module : Lastrun — tour 0.4.0

- [x] Fiche `modules/lastrun.toml`, `spec.rs`, rapport de test (faux HOME de Lastrun),
      comptes 5 → 6 ; paquet complet avec `lastrun` (provides/conflicts/replaces)
- [x] Hubs : README de la tour, profil de l'organisation, `github_meta.sh`, `new-module.md`
- [x] Release de Lastrun 0.1.0 d'abord, puis tour 0.4.0 (paquet seul et complet) — publiees
      et verifiees depuis l'exterieur le 2026-10-17
- [x] Passage reel de `bbl refresh` avec Lastrun installe (complet 0.4.0) : 7 constats en
      0,8 s, six modules installes

## Etape 7. Promotion et passation (2026-10-26)
- [x] Plan de promotion module par module, exporte dans `00_cadrage/promotion.md` (hors git) — 2026-10-26
- [x] Passation a OpenCode : `AGENTS.md` (source unique des regles, `CLAUDE.md` en pointeur) et `opencode.json` dans les douze depots ; etat de la serie dans `Logiciels/PASSATION.md` — 2026-10-26
- [ ] Publications (X, Reddit, autres) : decision d'Alexandre, module par module, selon le plan

> Suspendue par la restructuration du 2026-11-07 : le plan de promotion sera réécrit pour BrambleRS après la 0.1.0.

## Etape 8. Restructuration BrambleRS (0.1.0) — décision du 2026-11-07

> Branche de travail : `bramble-rs` (locale, pas poussée). `main` garde le commit de décision.

- [x] BRIEF rev. 2 validé (« Oui, je valide ») ; nom BrambleRS, radical technique `bramblers`, org Blackberrylabs — 2026-11-07
- [x] Trois fondations importées (subtree) et renommées `bramblers-*` ; tour renommée `bramblers` ; `cargo test --workspace` passe (63 tests), fmt et clippy propres — 2026-11-07, commit `8190546`
- [x] Six cores importés avec leur historique (`crates/modules/<id>/core`), données `hosts/`, `rules/`, `providers/`, `prices/` comprises ; dépendances par chemin ; schéma interne `bramblers-report/1` — 2026-11-07, commits `b133f27` et `57aedb2`
- [x] Registre interne `registry` : six fournisseurs statiques, `runner` en processus avec limite de temps, plus de PATH ni de sous-processus pour les modules ; app et CLI adaptés ; `Dashboard::from_saved` sans découverte — 2026-11-07, commit `7b3c115`
- [x] Preuve du registre : `cargo test --workspace` passe (202 tests), fmt et clippy propres ; `bramblers-cli refresh permitt` sur la machine : 16 constats en 0,1 s, un seul fichier écrit (`<state>/permitt/report.json`) — 2026-11-07
- [ ] Binaire et CLI `bramblers` : `report --json`, `report <module> --json` ; sans argument, l'app
- [ ] Paquet `bramblers` : .deb/.rpm/AppImage, menu `BrambleRS`, icône `bramblers`, shim config/état
- [ ] GitHub : renommer le dépôt en `BrambleRS`, archiver les dix anciens dépôts (pas supprimer), profil d'org, release 0.1.0, tag `bramblers-v0.1.0`
- [ ] Nettoyage local : dossier `BrambleRS`, AGENTS/CLAUDE/PLAN/JOURNAL
- [ ] 0.2.0+ : portage des six vues détaillées (Cleanator, MCProster, Permitt, Linstruct, Billfold, Lastrun)
- [ ] 1.0.0 : parité, toutes les fonctionnalités des six apps dans BrambleRS

> Plus compilés depuis le registre, encore sur disque et à supprimer avec l'accord d'Alexandre : `crates/core/src/discover.rs`, `spec.rs`, `run.rs`, `packages.rs` ; les six `modules/*.toml` ; `docs/new-module.md` (contrat des modules externes). Le `README.md` racine décrit encore l'ancienne tour et sera réécrit avec le paquet 0.1.0.

## Historique — idees de la tour 0.x (remplacees par l'etape 8)

> Ces idees appartiennent a la tour 0.x. La restructuration du 2026-11-07 les remplace ; elles restent ici comme historique.
- Lire aussi des fiches deposees dans `~/.config/blackberrylabs/modules/` (modules tiers), en plus des fiches embarquees.
- La tour comme porte d'entree de la famille (idee d'Alexandre, 17/10) : « Get it » ouvre directement le bon paquet de la bonne release (format detecte via `/etc/os-release`), le navigateur telecharge ; R3 intact. README d'organisation qui presente la tour en premier.
- Verifier l'installation d'un module apres « Get it » (relecture du PATH a la demande).
