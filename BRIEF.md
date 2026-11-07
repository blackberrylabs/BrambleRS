# BRIEF — BrambleRS
<!-- Rev. 2 validée le 2026-11-07 par Alexandre. Remplace la rev. 1 du 2026-10-16. Français : pilotage. Code, docs, commits, README en anglais. -->

Rev. 2 — 2026-11-07, validée le 2026-11-07 (« Oui, je valide »). Proposée après les décisions d'Alexandre du 2026-11-07 : le logiciel ne porte plus le nom de l'org, les six outils deviennent un seul logiciel, et la 0.1.0 garde l'écran actuel de la tour.

## 1. Objet

**BrambleRS est un logiciel unique qui montre ce que les agents de code IA font à la machine, le plus grave en premier, et ouvre chaque constat au bon endroit.**

Il remplace la tour Blackberrylabs et les six outils séparés (Cleanator, MCProster, Permitt, Linstruct, Billfold, Lastrun). Blackberrylabs reste le nom de l'organisation ; BrambleRS est son produit. Les projets sans rapport avec BrambleRS (MiningControlRS, etc.) restent des dépôts séparés de l'organisation. À terme, BrambleRS reprend toutes les fonctionnalités des six apps ; la 0.1.0 est la première étape, pas la cible finale. La 1.0 sera atteinte quand toutes les fonctionnalités des six apps seront dans BrambleRS.

Accroche : *BrambleRS — everything your AI coding agents do to your machine, on one screen. Read-only.*

## 2. Décisions de restructuration

- **D1 2026-11-07** : le logiciel s'appelle **BrambleRS**. Le suffixe RS distingue la version Rust, comme MiningControlRS. Blackberrylabs reste l'organisation. Décidé par Alexandre.
- **D2 2026-11-07** : un seul logiciel, un seul dépôt, un seul paquet. Les six outils deviennent des modules internes, plus six logiciels séparés. Décidé par Alexandre.
- **D3 2026-11-07** : la 0.1.0 reprend l'écran actuel de la tour (liste de constats, résumés, détail). Les six apps détaillées ne sont pas reprises en 0.1.0 ; la cible 0.x est de reprendre toutes leurs fonctionnalités (D7). Décidé par Alexandre.
- **D4 2026-11-07** : les anciens dépôts sont archivés, pas supprimés. Les releases restent téléchargeables. Proposition de l'agent, validée avec la rev. 2.
- **D5 2026-11-07** : les projets sans rapport avec BrambleRS restent des dépôts séparés de l'org. Décidé par Alexandre.
- **D6 2026-11-07** : une seule app iced, un binaire `bramblers`, un paquet `bramblers`, des crates internes `bramblers-*`. Proposition de l'agent, validée avec la rev. 2.
- **D7 2026-11-07** : la cible 0.x est de reprendre **toutes les fonctionnalités des six apps** dans BrambleRS, vue par vue, jusqu'à ce qu'aucune app séparée n'ait de raison d'exister. La 0.1.0 n'attend pas ce portage. La 1.0 est le jalon de parité. Décidé par Alexandre.

## 3. Ce que BrambleRS 0.1.0 fait

- Un écran d'accueil : quatre chiffres, une liste de constats tous modules confondus, triée par gravité. Chaque constat ouvre le détail (preuves, actions affichées) et le module concerné.
- Six modules internes, chacun fournit ses chiffres et ses constats :
  - **Cleanator** : ce que l'outillage IA laisse sur le disque.
  - **MCProster** : les serveurs MCP déclarés, doublons, commandes mortes, jetons en clair.
  - **Permitt** : ce que chaque agent peut faire sans demander, et quel réglage le dit.
  - **Linstruct** : les fichiers d'instructions tels que chaque hôte les lit.
  - **Billfold** : les comptes et clés présents, les jetons consommés, prix aux tarifs API.
  - **Lastrun** : ce que les agents ont réellement fait, session par session.
- Quatre gravités : 1 agir maintenant, 2 à regarder, 3 bon à savoir, 4 information.
- Read-only : aucune modification de la machine. Les actions correctives sont affichées, jamais exécutées. Aucune sortie réseau.
- Anglais par défaut, français en option.
- **Limite de la 0.1.0** : les inventaires complets des six apps (tableaux, sessions, disques, comptes) ne sont pas encore dans l'écran ; le rapport ne transporte que les constats. Le portage des six vues détaillées est la feuille de route 0.x (D7), pas une option.

## 4. Architecture

- Un workspace Rust unique, un `Cargo.lock`, une version.
- Un seul radical technique : `bramblers`. Le nom affiché est `BrambleRS`, l'org reste `Blackberrylabs`.
- `crates/bramblers-core` : registre des six modules, orchestration, cache, tri, modèle commun.
- `crates/bramblers-report` : types du rapport et des gravités (ancien `bbl-report`), schéma interne `bramblers-report/1`.
- `crates/bramblers-hostrules` : fiches d'hôtes et lecteurs (ancien `bbl-hostrules`).
- `crates/bramblers-ui` : palette et briques iced (ancien `bbl-ui`).
- `crates/bramblers-app` : l'app iced unique, l'écran actuel de la tour.
- `crates/bramblers-cli` : le CLI `bramblers`, avec `report --json` et les sous-commandes des six modules.
- `crates/modules/<id>/core` (crate `bramblers-<id>-core`) : la logique des six outils, réutilisée telle quelle. Les six apps ne sont pas importées en 0.1.0.
- **Un binaire `bramblers`** : sans argument, l'app ; `bramblers report --json` pour le rapport global ; `bramblers report <module> --json` pour un module.
- **Un paquet `bramblers`**, une entrée de menu `BrambleRS` (`Exec=bramblers`, `Icon=bramblers`), une icône. Le paquet remplace et est en conflit avec `blackberrylabs`, `blackberrylabs-full` et les six paquets de modules.
- **Chemins** : `~/.config/bramblers/` et `~/.local/state/bramblers/`. Un shim lit les anciens chemins `~/.config/blackberrylabs/` et `~/.local/state/blackberrylabs/` au premier lancement.
- **Dépôt et dossier local** : `blackberrylabs/BrambleRS` et `Logiciels/BrambleRS`. GitHub est insensible à la casse ; le nom affiché garde `BrambleRS`.

## 5. Après la 0.1.0 : toutes les fonctionnalités des six apps

La cible n'est pas un tableau de bord qui laisse six apps à côté. Chaque fonctionnalité des six apps doit vivre dans BrambleRS :

- **Cleanator** : le découpage du disque, les assistants, les projets, l'explorateur, le panneau de recommandations et le script de nettoyage.
- **MCProster** : le tableau des serveurs, les filtres, le détail, les sondes, les constats.
- **Permitt** : le tableau des hôtes, les filtres, la matrice de permissions, les preuves, les constats.
- **Linstruct** : les dossiers, les hôtes, la liste des fichiers, le détail, les constats.
- **Billfold** : les comptes, le câblage, les clés, le compteur, les constats.
- **Lastrun** : les sessions, les suppressions, les paquets, les refus, les diffs, les constats.

Le portage se fait vue par vue, en 0.x, sans bloquer la 0.1.0. Une vue est terminée quand l'app correspondante n'a plus de fonctionnalité que BrambleRS n'a pas. Les six apps restent archivées comme référence, jamais comme produit parallèle. La 1.0 est atteinte quand cette liste est vide côté apps.

## 6. Ce qui disparaît

- La découverte des modules sur le PATH, les fiches `modules/*.toml`, le champ `latest`, `bundle.sh`, `github_meta.sh` multi-dépôts, le gabarit `template`.
- Le `report --json` par module, le bouton « Get it », le bouton « Open in <Module> ».
- Les six apps séparées et leurs écrans détaillés en 0.1.0 ; leurs fonctionnalités reviennent en 0.x (section 5).
- Le contrat externe `bbl-report/1` ; il devient le schéma interne `bramblers-report/1`. Les champs de distribution (`app`, `cli`, `home`, `open`) ne sont plus lus.
- Les six dépôts actifs et leurs releases séparées. Ils sont archivés après la 0.1.0.

## 7. Règles

- **R1 Read-only** : BrambleRS ne modifie rien ; il lit, agrège, affiche.
- **R2 Écriture** : uniquement `~/.config/bramblers/` et `~/.local/state/bramblers/`.
- **R3 Réseau** : aucune sortie réseau.
- **R4 Actions** : les actions correctives sont affichées, jamais exécutées.
- **R5 Modularité interne** : la logique d'un module vit dans son crate ; l'app ne connaît que le registre et les types communs.
- **R6 Langue** : anglais par défaut, français en option.
- **R7 Une version** : un `CHANGELOG`, un `README`, un paquet, une release.

## 8. Versions

- **0.1.0** : la restructuration. Un dépôt, une app, l'écran actuel, six cores, un paquet.
- **0.2.0 à 0.x** : le portage des six vues détaillées, une ou plusieurs par release.
- **1.0.0** : la parité. Toutes les fonctionnalités des six apps vivent dans BrambleRS ; aucune app séparée n'a de fonctionnalité propre.

Les tags de la nouvelle ligne sont `bramblers-v0.1.0`, `bramblers-v0.2.0`, etc. : le dépôt renommé garde les anciens tags `v0.1.0` à `v0.4.0` de la tour, et un nouveau `v0.1.0` entrerait en collision. La version du paquet est `0.1.0`.

## 9. Migration

Le détail est dans `00_cadrage/2026-11-07_plan_restructuration.md`. Ordre : nom et BRIEF validés, monodépôt avec les six cores, registre interne (plus de PATH ni de sous-processus), app actuelle renommée BrambleRS, paquet unique, archivage GitHub, release 0.1.0. Chaque étape est livrable et testée ; aucune suppression de dépôt.

## 10. Hors périmètre (0.1.0)

- Modules externes, plugins tiers, découverte sur le PATH.
- Croisements entre modules, historique et tendances, notifications.
- Exécution d'actions correctives.
- Site web et pages de documentation publiques (nécessaires à la découverte, mais hors de cette rev).

Les six vues détaillées ne sont pas hors périmètre : elles sont planifiées en 0.x (section 5).

## 11. Ce qui reste de la rev. 1

Les promesses des six outils, les quatre gravités, le read-only, l'absence de réseau, les actions affichées seulement, la phrase de famille, le soin du contrat (devenu interne), la découverte par recherche (à porter par la documentation). Le reste (tour séparée, modules sur le PATH, releases par module, six apps) est remplacé. La cible 0.x reprend les six apps ; la 1.0 marque la parité.
