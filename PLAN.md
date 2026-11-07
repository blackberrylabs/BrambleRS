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

> Restructuration faite et publiée le 2026-11-07 : `main` poussé, tag `bramblers-v0.1.0`, release et paquets en ligne.

- [x] BRIEF rev. 2 validé (« Oui, je valide ») ; nom BrambleRS, radical technique `bramblers`, org Blackberrylabs — 2026-11-07
- [x] Trois fondations importées (subtree) et renommées `bramblers-*` ; tour renommée `bramblers` ; `cargo test --workspace` passe (63 tests), fmt et clippy propres — 2026-11-07, commit `8190546`
- [x] Six cores importés avec leur historique (`crates/modules/<id>/core`), données `hosts/`, `rules/`, `providers/`, `prices/` comprises ; dépendances par chemin ; schéma interne `bramblers-report/1` — 2026-11-07, commits `b133f27` et `57aedb2`
- [x] Registre interne `registry` : six fournisseurs statiques, `runner` en processus avec limite de temps, plus de PATH ni de sous-processus pour les modules ; app et CLI adaptés ; `Dashboard::from_saved` sans découverte — 2026-11-07, commit `7b3c115`
- [x] Preuve du registre : `cargo test --workspace` passe (202 tests), fmt et clippy propres ; `bramblers-cli refresh permitt` sur la machine : 16 constats en 0,1 s, un seul fichier écrit (`<state>/permitt/report.json`) — 2026-11-07
- [x] Binaire et CLI `bramblers` : `report --json`, `report <module> --json`, `list`, `refresh` ; sans argument, l'app — 2026-11-07, commit `a49babc` ; `--help` et `report permitt --json` vérifiés
- [x] `packaging/build.sh` réécrit pour le paquet unique : un binaire `bramblers`, `.desktop` et icônes `bramblers`, AppImage `bramblers-<version>-<arch>`, `Conflicts`/`Replaces`/`Provides` des anciens paquets dans les métadonnées deb et rpm — 2026-11-07
- [x] `packaging/build.sh` lancé : `.deb`, `.rpm`, AppImage et `SHA256SUMS` dans `dist/` ; `sha256sum -c` passe sur les trois — 2026-11-07
- [x] `.deb` installé : `bramblers 0.1.0`, `blackberrylabs-full` 0.4.0 retiré, `bramblers report permitt --json` rend 16 constats, l'app lancée 8 s écrit ses rapports dans `~/.local/state/bramblers/`, `desktop-file-validate` propre — 2026-11-07
- [x] AppImage lancée 8 s et extraite : `AppRun`, `bramblers.desktop`, icône — 2026-11-07
- [x] Shim : les réglages de l'ancienne app `blackberrylabs` sont lus une fois vers `bramblers` ; les anciens rapports `bbl-report/1` sont ignorés — 2026-11-07
- [x] `README.md` et `CHANGELOG.md` réécrits pour BrambleRS 0.1.0 ; fichiers morts supprimés : 4 sources de l'ancienne tour, 6 fiches `modules/*.toml`, `docs/new-module.md`, `packaging/bundle.sh`, 27 fichiers de pilotage importés, `dist/` et `packaging/.bundle/` — 2026-11-07
- [x] Interface à rail gauche, sur le modèle de MiningControlRS : en-tête, rail de 210 px avec « All findings » et les six modules en onglets, chiffres de gravité et liste/détail à droite ; les cartes de modules et les puces de filtre disparaissent — 2026-11-07
- [ ] `.rpm` installé sur une machine RPM (rpm n'est pas installé ici ; le paquet est construit)
- [x] GitHub : dépôt renommé `blackberrylabs/BrambleRS`, dix anciens dépôts avec bandeau puis archivés (pas supprimés), profil d'org et métadonnées réécrits, release 0.1.0 avec tag `bramblers-v0.1.0` — 2026-11-07 ; `gh release download` et `sha256sum -c` vérifiés depuis l'extérieur
- [x] Nettoyage local : dossier `Logiciels/BrambleRS`, AGENTS/CLAUDE/PLAN/JOURNAL mis à jour — 2026-11-07
- [x] Arborescence : `crates/bramblers`, `crates/bramblers-core`, `crates/bramblers-cli`, un dossier par module — 2026-11-07
- [ ] Étape 9 : les six vues détaillées, une par onglet, voir ci-dessous

> Réglages GitHub manuels restants : épingler BrambleRS sur le profil de l'organisation, aperçu social. Anciennes captures dans `docs/screenshots/` et `assets/blackberrylabs.desktop` à retirer ou remplacer.

## Étape 9. Les six vues détaillées (0.1.1 à 1.0) — décision du 2026-11-07

> Objectif : chaque onglet retrouve **toutes** les fonctionnalités de son ancienne app. Le plan détaillé, avec l'inventaire écran par écran, est dans `00_cadrage/2026-11-07_plan_vues_modules.md` (hors git). Les sources des anciennes apps sont dans `00_cadrage/references/<id>/` (hors git).

- [ ] 0.1.1 : Permitt — tableau hôtes × capacités, filtres, détail, preuves, constats. Vue portée, branchée, testée et capturée le 2026-11-07 (commits `aa8670e`, `5750408`, `1c2040a` ; capture `docs/screenshots/permitt.png` ; 212 tests) ; version du workspace passée à 0.1.1. Restent la vérification par Alexandre, la suppression du checkout et les paquets (accord requis).
- [ ] 0.1.2 : MCProster — tableau des serveurs, filtres, détail, sondes, coût en contexte. Vue portée, branchée, testée et capturée le 2026-11-07 (commit `aa98290` ; capture `docs/screenshots/mcproster.png` ; 218 tests) ; version du workspace passée à 0.1.2. Restent la vérification par Alexandre, la suppression du checkout et les paquets (accord requis).
- [ ] 0.1.3 : Linstruct — dossiers, hôtes, piles de fichiers, détail, constats. Vue portée, branchée, testée et capturée le 2026-11-07 (commit `1add611` ; capture `docs/screenshots/linstruct.png` ; 222 tests) ; version du workspace passée à 0.1.3. Restent la vérification par Alexandre, la suppression du checkout et les paquets (accord requis).
- [ ] 0.1.4 : Cleanator — arborescence, assistants, projets, explorateur, recommandations, script
- [ ] 0.1.5 : Billfold — comptes, câblage, clés, compteur, constats
- [ ] 0.1.6 : Lastrun — sessions, actions, suppressions, paquets, refus, diffs
- [ ] 1.0.0 : parité, aucune fonctionnalité des six apps ne manque

Une vue est finie quand :

- son dossier `crates/modules/<id>/view` existe et l'app l'affiche dans son onglet ;
- chaque écran et chaque action de l'ancienne app est présent, ou explicitement abandonné par Alexandre ;
- les tests de la vue passent et une capture sur un jeu de démonstration est ajoutée ;
- l'ancien checkout `Logiciels/<Module>` est supprimé (avec accord).

## Historique — idees de la tour 0.x (remplacees par l'etape 8)

> Ces idees appartiennent a la tour 0.x. La restructuration du 2026-11-07 les remplace ; elles restent ici comme historique.
- Lire aussi des fiches deposees dans `~/.config/blackberrylabs/modules/` (modules tiers), en plus des fiches embarquees.
- La tour comme porte d'entree de la famille (idee d'Alexandre, 17/10) : « Get it » ouvre directement le bon paquet de la bonne release (format detecte via `/etc/os-release`), le navigateur telecharge ; R3 intact. README d'organisation qui presente la tour en premier.
- Verifier l'installation d'un module apres « Get it » (relecture du PATH a la demande).
