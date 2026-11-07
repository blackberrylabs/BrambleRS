# JOURNAL — BrambleRS (ex-Blackberrylabs)
<!-- Project journal (French), newest first. -->

---

## 2026-11-07, six cores internes, registre en processus

- Les six cores (Billfold, Cleanator, Lastrun, Linstruct, MCProster, Permitt) entrent dans le workspace sous `crates/modules/<id>/core`, crates `bramblers-<id>-core`, avec leurs données `hosts/`, `rules/`, `providers/` et `prices/`. Import par `git subtree split` sur `crates/core` puis sur chaque dossier de données : l'historique des cores est conservé. Commits `b133f27` (import) et `7b3c115` (registre).
- Le registre `bramblers-core::registry` remplace la découverte sur le PATH : six `Provider` statiques (id, nom, verbe, accroche, description, limite de temps, fonction `run`). `runner::report` appelle la fonction du core sur un thread et attend au plus la limite du registre. Plus de `Command::new` pour lancer un module, plus de fiche TOML, plus de sous-processus. L'app et la CLI lisent les rapports enregistrés puis relancent les six modules en processus.
- L'écran perd « Get it », « Open in <Module> » et l'état « non installé ». Les six cartes sont toujours là ; la ligne de statut dit « 6 modules · N rapport(s) lu(s) ». `xdg-open` reste pour les actions que la personne déclenche (ouvrir un fichier, un lien), jamais pour lancer un module.
- Le schéma interne devient `bramblers-report/1` (commit `57aedb2`) ; les anciens rapports `bbl-report/1` ne sont plus lus, et l'ancien dossier d'état `blackberrylabs` n'est pas consulté.
- Deux défauts de lint importés corrigés sans changer le comportement : une assertion tautologique dans un test Permitt, un booléen simplifié dans le compteur Claude de Billfold.
- Preuves : `cargo test --workspace` passe (202 tests), `cargo fmt --all --check` et `cargo clippy --workspace --all-targets` propres ; `bramblers-cli refresh permitt` sur la machine rend 16 constats en 0,1 s et n'écrit qu'un fichier, `<state>/permitt/report.json`.
- Restent sur disque, plus compilés et à supprimer avec l'accord d'Alexandre : `crates/core/src/{discover,spec,run,packages}.rs`, les six `modules/*.toml`, `docs/new-module.md`. Le `README.md` racine décrit encore l'ancienne tour.
- Rien n'a été poussé ni modifié sur GitHub.

---

## 2026-11-07, BrambleRS : un seul logiciel, un seul dépôt

- Décision d'Alexandre : le logiciel ne s'appelle plus Blackberrylabs (c'est l'org) mais **BrambleRS** ; les six outils deviennent un seul logiciel, un seul dépôt, un seul paquet. MiningControlRS et les projets sans rapport restent séparés.
- Périmètre de la 0.1.0 : l'écran actuel de la tour (liste, résumés, détail des constats). Les six apps détaillées sortent du produit en 0.1.0 ; elles seront archivées avec leurs dépôts, et **toutes leurs fonctionnalités reviennent dans BrambleRS en 0.x**, une vue après l'autre. La 1.0 est le jalon de parité.
- Nomenclature validée : produit `BrambleRS`, dépôt `blackberrylabs/BrambleRS`, radical technique `bramblers` partout (paquet, binaire, crates, chemins, tags `bramblers-v0.1.0`). Org `Blackberrylabs` inchangée.
- `BRIEF.md` passe en rev. 2 ; `PLAN.md` gagne l'étape 8 ; le plan détaillé est dans `00_cadrage/2026-11-07_plan_restructuration.md` (hors git).
- Rien n'a été poussé ni modifié sur GitHub. Les anciens dépôts seront archivés, pas supprimés, après la 0.1.0.
- Premier morceau de code sur la branche locale `bramble-rs` : trois fondations importées par subtree, tour renommée `bramblers`, 63 tests passent, fmt et clippy propres ; commit `8190546`. La suite (six cores, registre) est dans PLAN, étape 8.
- Preuve : « Oui, je valide » du 2026-11-07 ; `BRIEF.md` rev. 2 ; étape 8 de `PLAN.md`.

---

## 2026-10-26, passation à OpenCode

- `AGENTS.md` ajouté : source unique des règles pour OpenCode et tout autre agent. OpenCode ne charge pas `CLAUDE.md` quand `AGENTS.md` existe (documentation d'OpenCode lue le 2026-10-26). `CLAUDE.md` devient un pointeur (`@AGENTS.md`) avec deux lignes propres à Claude. Le bloc commun d'`AGENTS.md` est identique dans les douze dépôts de la série.
- `opencode.json` ajouté : tout ce qui pousse, tague, supprime, installe ou touche une minuterie demande confirmation ; les fichiers `.env` sont refusés ; `BRIEF.md` et `LICENSE` demandent confirmation à l'édition.
- PLAN : commits docs cochés (présents dans `origin/main`), ligne des captures complétée (Billfold fait, Linstruct et Lastrun manquent), étape 7 ajoutée. Le plan de promotion est exporté dans `00_cadrage/promotion.md`, hors git.
- Preuve : état de la série daté du 2026-10-26 dans `Logiciels/PASSATION.md`. Demandé par Alexandre le 2026-10-26.

---

## 2026-10-17, nuit — Lastrun, sixieme et dernier module (0.4.0)

- **Lastrun 0.1.0** (`replay`, « la boite noire des agents ») entre dans la tour en suivant
  `docs/new-module.md` : fiche `modules/lastrun.toml`, entree de `spec.rs`, rapport de son
  faux HOME dans `tests/fixtures/reports/` (chemins `/home/me`, aucune donnee de la machine
  d'Alexandre), comptes de modules 5 → 6. Essai a blanc fait avant la release du module :
  seuls ces comptes etaient a changer.
- **Paquet complet** : `blackberrylabs-full` fournit, remplace et entre en conflit avec
  `lastrun` comme avec les cinq autres ; `bundle.sh` prend le `.deb` de Lastrun dans
  `../Lastrun/dist/` (module publie d'abord, tour ensuite).
- **Hubs** : ligne Lastrun au README de la tour et au profil de l'organisation ; description
  et topics de `blackberrylabs/lastrun` dans `github_meta.sh` ; la description de la tour
  dit aussi « what they actually ran » ; mapping des gravites de Lastrun dans
  `docs/new-module.md`.
- **Publie** : Lastrun 0.1.0 puis la tour 0.4.0, verifiees depuis l'exterieur ; complet 0.4.0
  installe a la place du 0.3.1 ; `bbl refresh lastrun` : 7 constats en 0,8 s.
- **Accroc, de mon fait** : `github_meta.sh` a perdu son bit d'execution dans le commit 0.4.0
  (recopie depuis une copie qui ne le gardait pas) ; mode 755 rendu par un commit dedie.

---

## 2026-10-17, soir — un logiciel, pas un lanceur ; une installation, pas six

- **Premier usage par Alexandre** : « quand je clique sur un module, ca ouvre l'autre
  logiciel a cote ; on a l'impression d'avoir un logiciel, mais il en ouvre un autre ».
  Decision T6 : la carte d'un module est un filtre, le volet montre le module (chiffres,
  rapport, constats par gravite), son application ne s'ouvre que par « Open in … ↗ » dans
  le volet. Premiere correction insuffisante (le bouton restait sur la carte) ; retiree.
- **« Get it » ne telechargeait rien** : il ouvrait la page des releases. Desormais le
  paquet exact de la machine, derniere version connue par la fiche (`latest`) ; R3 intact.
- **Le modele d'Alexandre, ecrit partout** : « celui qui veut un seul module l'a tel quel ;
  celui qui en veut plusieurs va sur la tour et choisit ». Et : « il faut que la tour soit
  le seul logiciel installe » → paquet complet `blackberrylabs-full` (tour + binaires
  publies des cinq modules, `Provides`/`Conflicts` avec les unitaires), tour legere gardee ;
  les modules sont cherches a cote du binaire de la tour (AppImage). 0.3.1 le soir meme :
  les entrees de menu des modules masquees dans le complet (Alexandre les voyait encore).
- **Coherence publique** (audit SEO du jour, `Logiciels/audit.md`) : phrase de famille
  validee, six descriptions, topics corriges (`agents-md` mort → `agentsmd`, topic
  `blackberrylabs` garde comme hub, topics mensongers retires de Billfold), `homepage` des
  modules vers la tour, description d'organisation — tout par `packaging/github_meta.sh`,
  source de verite ; hubs (README de la tour, profil, « Part of Blackberrylabs » uniforme)
  avec Billfold. Ecarte : Discussions auto-posees, wiki, Pages par depot, crates.io pour
  bbl-ui et bbl-report.
- **Lecon** : deux fois ce soir ma copie en retard a failli ecraser des fichiers a jour
  (README, CHANGELOG) ; toujours partir du fichier de la machine, jamais de la copie.

## 2026-10-16, naissance

- Idee du 07/10 (une app pour piloter tous les projets d'Alexandre), redefinie le 16/10 au
  matin (le point d'entree des outils Blackberrylabs), tranchee le 16/10 au soir : « ce
  n'est pas tous mes projets, c'est la tour de controle pour Blackberrylabs et tous ses
  modules » ; « a terme, la tour de controle d'un developpeur ou d'un vibe codeur sur son
  poste ». Blackberrylabs est le produit ; les outils sont ses modules ; la tour porte le nom.
- Ordre tenu : la fondation d'abord (`hostrules` le soir, `bbl-ui` la nuit), la tour ensuite.
  Alexandre : « bien faire des le depart pour pouvoir integrer d'autres modules plus tard ».
  Traduction dans le BRIEF : soigner le **contrat** tour ↔ module (rapport `bbl-report/1`,
  gravites communes, decouverte declarative, cache ecrit par la tour), pas l'ecran.

## 2026-10-16 — BRIEF valide

BRIEF rev. 1 valide d'un mot (« Je valide ») : T0 a T4 prises. Etape 0 ouverte : la crate
`bbl-report` d'abord (dossier `Logiciels/Bbl_report`, son propre BRIEF), puis `report --json`
dans Cleanator, MCProster et Permitt, et `--select` dans les trois apps. Fiches 00_moi
(bbl-ui, Blackberrylabs) ecrites dans PROJETS.md ; l'ancienne idee `Tour_de_controle`
requalifiee.

## 2026-10-16 — le contrat existe et trois modules le parlent

`bbl-report` 0.1.0 publie ; `permitt report`, `mcproster report`, `cleanator-cli report`
produisent chacun un document conforme, teste sur leurs maisons factices ; les trois apps
prennent `--select`. Ce que l'exercice a appris pour la tour :
- Le grain differe selon le module (un constat par constat chez Permitt et MCProster, un
  par groupe de regle chez Cleanator) ; la tour ne doit rien supposer sur le nombre de
  constats d'un module.
- `open.args` n'est pas toujours `--select x` : Cleanator passe d'abord le chemin scanne.
  La tour lance `open.app` avec `open.args` tels quels, sans les interpreter (R5).
- Les `summary` ont des tons : `good` pour une bonne nouvelle (safe a liberer), `amber`,
  `alert`. La tour affichera le ton du module, pas une regle a elle.
- Les constats `unknown` de Permitt et `disabled` de MCProster arrivent en information : la
  tour doit pouvoir les replier par defaut (filtre « gravite minimale »).
Prochaine etape : le coeur de la tour et `bbl` en ligne de commande, avec les trois vrais
rapports en fixtures.

## 2026-10-17 — le coeur et `bbl`

- `bbl-core` en quatre morceaux : `spec` (les fiches de modules, TOML declaratif embarque :
  id, nom, verbe, accroche, binaires, arguments du rapport, limite de temps — 900 s pour
  Cleanator qui scanne le disque, 60 s pour les autres —, schema, page, releases),
  `discover` (`which` sur PATH, rien d'autre), `run` (lance `<cli> report`, vide stdout et
  stderr sur deux fils pour qu'un module bavard ne bloque pas, tue au-dela de la limite,
  verifie la sortie avec `conformance::check`), `cache` (`~/.local/state/blackberrylabs/
  <module>/report.json`, ecriture a cote puis renommage, age par la date du fichier,
  `$BBL_STATE_DIR` pour les tests), `dashboard` (etats par module : frais, enregistre avec
  age, sans rapport, non installe, en echec ; lignes triees gravite → ordre de la famille →
  titre).
- `bbl` ne lance rien tout seul : il montre les rapports enregistres ; `bbl refresh` lance.
  Choix delibere pour la ligne de commande — previsible ; c'est l'app qui relancera en
  arriere-plan (T2).
- Passage reel dans le cloud : les trois vrais binaires sur un PATH temporaire. Le
  `cleanator-cli` 0.1.1 encore installe dans le cloud a repondu « unrecognized subcommand
  'report' » — exactement le cas d'un module plus vieux que le contrat, affiche comme
  echec avec la premiere ligne de stderr. Avec le binaire a jour : 23 constats Cleanator,
  2 MCProster, 45 Permitt, tries ensemble.
- Decouverte au passage : Cleanator citait parfois une phrase comme source ; corrige dans
  Cleanator (commit 22 : une source qui n'est pas une URL devient une indication).
- Un echec de rafraichissement n'est pas persiste : au prochain `bbl`, le module est « sans
  rapport » ou garde son ancien rapport. L'app montrera l'erreur en direct.

## 2026-10-17 — maquette et app

- Consigne d'Alexandre avant la maquette : « la tour doit etre comprehensible et intuitive,
  pour les vibe codeurs comme pour les developpeurs ». C'est ce qui a dicte l'ordre de
  l'ecran : une phrase d'abord (« Your machine, AI side: 6 things to act on now, 32 to
  review »), les quatre gravites en mots avec leur definition sous le chiffre, les modules
  avec ce qu'ils repondent en une phrase, la liste groupee par gravite, le detail en trois
  questions (ce que ca veut dire / ou / ce que vous pourriez faire), et la phrase qui
  rassure a cote du bouton « Open in … ». Rien d'invente : « ce que ca veut dire » est le
  `detail` du module, omis quand il n'y en a pas. Maquette validee d'un mot (A1 : la phrase
  remplace la carte « findings »).
- App ecrite dans la foulee. Le coeur a gagne `launch` / `apply` pour que l'app lance un
  module sur un fil et reinjecte le resultat ; `bbl` garde `refresh` synchrone. Verifie a
  l'ecran dans le cloud : rapports enregistres affiches avec leur age, relance en
  arriere-plan avec « Running… (report from 26 min ago shown) », remplacement a la fin ;
  module absent en pointille avec « Get it » ; module trop vieux (le `cleanator-cli` 0.1.1
  du cloud) affiche son erreur sur sa carte, sans bloquer les autres.
- Icone : un radar (la tour) avec les quatre gravites en points, du centre vers l'exterieur.
- Reste pour la release : un passage reel sur la machine d'Alexandre, qui suppose que les
  trois modules y soient reconstruits avec `report` (leurs paquets installes sont les
  anciens).

## 2026-10-17 — passage reel

`bbl refresh` sur la machine d'Alexandre, les trois modules reconstruits avec `report` dans
`~/.local/bin` (les paquets installes sont encore les anciens) : Cleanator 22 constats en
5,1 s, MCProster 1, Permitt 12 — 0 a agir maintenant, 17 a regarder, 7 bon a savoir,
11 informations. Les gros volumes de Cleanator (target/ 31,8 Go, sauvegardes 27,9 Go,
archives 19,1 Go) arrivent bien en « a regarder » (≥ 1 Gio), avec le dossier a ouvrir et
« Open in Cleanator » sur le bon groupe. Rien a corriger dans la tour. Pour que ca marche
chez quelqu'un d'autre, il faut que les trois modules soient publies avec `report` :
c'est la prochaine decision (ordre des releases, numeros de version).

## 2026-10-17 — idee d'Alexandre : la tour comme porte d'entree

« Le mieux plus tard, ce ne serait pas de promouvoir la tour de controle et, depuis la tour,
choisir les modules dont on a besoin ? Et quand on clique, ca telecharge le module depuis
GitHub. » Oui pour la posture : la tour est le produit, les modules ses extensions, et la
vitrine (cartes en pointille, « Get it ») existe deja. Reserve sur le telechargement par la
tour elle-meme : R3 (aucune sortie reseau) est une des promesses qui rendent la famille
credible. Piste pour 0.2, a trancher : la tour ouvre directement le bon fichier de la bonne
release (deb / rpm / AppImage selon `/etc/os-release`), le navigateur telecharge — un clic,
zero reseau dans la tour. Et le README d'organisation presente la tour en premier.

## 2026-10-17 — 0.1.0 publiee, la famille complete

- Quatre releases dans la journee, dans l'ordre decide : Permitt 0.2.0, MCProster 0.2.1,
  Cleanator 0.2.0, puis Blackberrylabs 0.1.0. Chacune verifiee depuis le cloud (somme,
  version, `report` conforme) ; pour la tour, le `.deb` publie a lu les trois `.deb` publies
  des modules : la chaine complete tient sans rien de local.
- Deux accrocs, tous deux de mon fait : `.gitignore` sans `dist/` ni `packaging/.tools/`
  (un commit local a embarque les paquets ; defait avant le push, corrige par le commit 4) ;
  et le verrou `.git/index.lock` que le pont laisse derriere lui, a enlever avant tout git.
- `gh` installe sur la machine d'Alexandre a cette occasion : les releases passent par
  `gh release create` avec les notes de `00_cadrage/`.
- README d'organisation ecrit (tour en premier, modules, promesses, fondations) ; reste a
  le publier dans `blackberrylabs/.github`.

## 2026-10-17 — rendre l'integration facile, pas seulement possible

- Question d'Alexandre : « le dossier Blackberrylabs est bien structure pour des modules
  supplementaires qui s'integrent facilement ? ». Reponse honnete : oui pour l'architecture
  (trois obligations, aucune ligne dans la tour, R5 verifiee par trois modules differents),
  deux reserves — les fiches embarquees demandent une release de la tour, et « facile »
  n'etait ecrit nulle part : pas de gabarit, pas de marche a suivre.
- Decision T5. Gabarit `blackberrylabs/template` : un workspace qui compile et dont les
  tests passent avant qu'on y touche, et qui parle deja le contrat (le `scan` est un
  placeholder qui le dit). `rename.sh` verifie sur une copie : renomme, recompile, tests
  verts. `docs/new-module.md` : la page qui tient les trois obligations, la fiche, et ce
  qu'on fait avant une release (module d'abord, tour ensuite).
- Dans la foulee, toute la doc de la famille relue : les trois modules disent qu'ils sont
  des modules et pointent vers la tour ; Permitt affichait encore « Status 0.1.0 » (oubli de
  la release du matin) ; les CLAUDE.md citent les trois fondations et le mapping ; les
  CONTRIBUTING disent qu'un nouveau constat a sa place sur les quatre gravites.
