# JOURNAL — BrambleRS (ex-Blackberrylabs)
<!-- Project journal (French), newest first. -->

---

## 2026-11-07, Cleanator : le bug venait du backend GL, pas du treemap

- Le treemap n'était pas en cause. Avec `WGPU_BACKEND=gl` (le chemin logiciel OpenGL), iced/wgpu 0.19 dessine mal les canvas : tuiles manquantes, textes flottants, marges ignorées. Avec Vulkan logiciel (lavapipe), la carte d'origine s'affiche correctement, titre et légende dans la carte.
- Les commits `c6c66b0` et `4a9cddf` (marges internes, titre hors carte) sont donc annulés par `650bb6d` : retour à la disposition d'origine, sans rustine.
- Le lancement matériel plante à cause d'un reset du pilote AMD (`amdgpu ... GPU reset(8)`), qui emporte X et OpenCode. Pour vérifier sans GPU : `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json WGPU_BACKEND=vulkan BRAMBLERS_NO_REFRESH=1 BRAMBLERS_MODULE=cleanator ./target/debug/bramblers`.
- Lastrun n'était pas un bug : 137,8 s en debug sur 201 Mo de transcriptions, environ 8 s en release.
- Preuves : `cargo test --workspace` passe 247 tests, clippy et fmt propres ; capture de la carte d'origine sous lavapipe, titre, tuiles et légende à leur place.

---

## 2026-11-07, vérification des vues : Cleanator corrigé, Lastrun lent en debug

- Cleanator : le canvas iced 0.13 dessine à l'origine de son arbre de disposition ; le treemap recouvrait le titre et la légende. Le titre et la légende sont sortis de la carte, le treemap garde ses propres marges (48 px en haut, 64 en bas, 14 sur les côtés) et le clic utilise le même rectangle. Test des marges. Commit `c6c66b0`.
- Lastrun : pas un blocage. Le scan a pris 137,8 s dans le build de debug sur 201 Mo de transcriptions, puis a affiché 128 sessions et 14 401 commandes. Le binaire release est construit et se lance en environ 8 s pour le même travail ; c'est celui qui sert à la vérification.
- Le premier lancement a été tué par un reset du GPU AMD (`amdgpu ... GPU reset(8)`), pas par le code. La vérification tourne en rendu logiciel (`WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1`), plus lent mais stable.
- Preuves : `cargo test --workspace` passe 248 tests, clippy et fmt propres ; l'app release est lancée sur Cleanator.
- Les six vues restent à vérifier écran par écran par Alexandre ; Cleanator est la première contrôlée.
- Correction le même jour : ce diagnostic était faux, voir l'entrée « le bug venait du backend GL » ci-dessus.

---

## 2026-11-07, audit de sécurité : F15 corrigé, audit clos

- Textes alignés sur le code : `lastrun/core/src/lib.rs`, `linstruct/core/src/lib.rs`, `mcproster/core/src/lib.rs`, `linstruct/core/src/projects.rs` et `bramblers-ui/src/lib.rs` disent les chemins réels (`~/.config/bramblers/<module>/`) et que le coeur n'écrit rien. Commit `23052cd`.
- Profil `.github` : « links are shown, never followed » devient « No network of its own. Nothing is sent anywhere; a link opens in your browser only when you click it. » Commit `cb485cc` dans `Blackberrylabs_github`, non poussé.
- Preuves : `cargo test --workspace` passe 247 tests, clippy et fmt propres. Les quinze constats du rapport sont corrigés ; F1 à F14 l'avaient été dans les commits précédents.
- Reste à pousser : BrambleRS et `.github`.

---

## 2026-11-07, audit de sécurité : F14 corrigé

- Décision d'Alexandre (« oui ») : garder `xdg-open` pour les fichiers et les liens, et filtrer seulement le schéma des liens. Remplacer l'ouvreur pour les fichiers aurait changé ses associations et pouvait ouvrir un éditeur terminal sans terminal.
- `bramblers_ui::open` : `url` n'accepte que `http://` et `https://` (casse ignorée), `file` passe le chemin à `xdg-open`. Les six vues et la coquille passent par ce module ; les anciens appels directs à `xdg-open` disparaissent.
- Test : `https` et `HTTP` acceptés, `file:`, `smb:`, `javascript:`, `ftp:` et `mailto:` refusés. Commit `1e4d7a3`.
- Preuves : `cargo test --workspace` passe 247 tests, clippy et fmt propres, `cargo check -p bramblers` compile. Le commit reste local.
- Reste le constat F15 (textes qui ne décrivent plus le code).

---

## 2026-11-07, audit de sécurité : F12 et F13 corrigés

- F12, HOME : `home_dir()` rend `HOME` s'il est valide, sinon l'entrée de `$USER`/`$LOGNAME` dans `/etc/passwd`, et `None` si rien ne donne un dossier. `Env::current` s'en sert et aligne `$HOME` ; `Cache` aussi. L'app et le CLI refusent de démarrer sans dossier personnel, et Cleanator refuse de scanner `/` en appel direct. Test du lecteur de passwd. Commit `8c0a89d`, plus `7950c68` pour le lien `bramblers-hostrules` dans le coeur et le binaire.
- F13, mutex : les quatre `lock().unwrap()` de Cleanator (`probes.rs`, `scan.rs`) deviennent `unwrap_or_else(|e| e.into_inner())` ; un thread paniqué ne fait plus tomber les suivants. Commit `5ecd710`.
- Preuves : `cargo test --workspace` passe 246 tests, `cargo clippy --workspace --all-targets` et `cargo fmt --check` propres, `cargo check -p bramblers` compile. Les trois commits restent locaux.
- Restent F14 (`xdg-open`) et F15 (textes).

---

## 2026-11-07, audit de sécurité : F10 et F11 corrigés

- F10, paquet AppImage : `build.sh` épingle `appimagetool` 1.9.1 et vérifie son SHA256 publié avant de l'exécuter ; un outil déjà présent dans `packaging/.tools/` est vérifié aussi et remplacé s'il ne correspond pas. Les sommes des quatre architectures sont dans le script. Commit `7c5d843`. `build.sh` n'a pas été lancé (accord requis) ; `bash -n` passe et la somme du fichier 1.9.1 téléchargé correspond.
- F11, écritures : les rapports, `probes.json`, les réglages et `cleanup.sh` passent par un fichier temporaire créé en `create_new` et renommé, en 0600. Un lien symbolique posé à la place du fichier final est remplacé, pas suivi. Tests de lien symbolique et de mode dans `cache` et `probe`. Commit `492a22c`.
- Preuves : `cargo test --workspace` passe 245 tests, `cargo clippy --workspace --all-targets` et `cargo fmt --check` propres, `cargo check -p bramblers` compile l'app.
- Restent les constats F12 à F15 du rapport.

---

## 2026-11-07, audit de sécurité : F8 et F9 corrigés

- F8, lectures bornées : Billfold et Lastrun refusent un journal de plus de 256 Mio et une décompression zstd de plus de 256 Mio (le fichier est lu par `File::take`, jamais par `fs::read` entier) ; Cleanator plafonne stdout et stderr de chaque sonde externe à 16 Mio. Commit `d4a8299`.
- F9, texte non ASCII : `strip_jsonc` parcourt les caractères au lieu des octets, donc `/home/rémi` reste `/home/rémi` dans un fichier JSONC ; `unescape` de `/proc/mounts` reconstruit les octets avant de décoder l'UTF-8. Tests ajoutés. Commit `6a6b0b7`.
- Preuves : `cargo test --workspace` passe 243 tests, `cargo clippy --workspace --all-targets` et `cargo fmt --check` propres. Les deux commits de code restent locaux, avec le commit de journal.
- Restent les constats F10 à F15 du rapport.

---

## 2026-11-07, audit de sécurité : F7 corrigé

- Décision d'Alexandre (« go avec ta reco ») : garder le scan des invites et des réponses pour trouver les identifiants, et corriger les textes plutôt que restreindre le scan.
- Textes alignés : `lastrun/core/src/lib.rs`, `lastrun/core/src/model.rs`, description du registre, `CHANGELOG.md` 0.1.6. La promesse dit maintenant que le texte est lu en mémoire pour chercher des identifiants, et jamais conservé, montré ou écrit ; seules les formes masquées entrent dans un constat. Commit `b181508`.
- Preuves : `cargo test --workspace` passe 241 tests, `cargo clippy --workspace --all-targets` et `cargo fmt --check` propres. Le commit reste local.
- Restent les constats F8 à F15 du rapport.

---

## 2026-11-07, audit de sécurité : F5 et F6 corrigés

- F6, CLI : `safe` remplace les caractères de contrôle par des espaces dans la liste humaine et dans les erreurs de lancement ; la sortie JSON n'est pas touchée, `serde_json` l'échappe déjà. Test avec une séquence OSC 52 dans un titre. Commit `6decda2`.
- F5, sonde MCProster : environnement réduit (`env_clear`, puis `PATH`, `HOME`, `TMPDIR`, `LANG`, `LC_*`, `USER`, `LOGNAME`, `XDG_RUNTIME_DIR` et les variables déclarées), groupe de processus (`process_group(0)` puis `kill -KILL -- -pid`), stdout plafonné à 32 Mio, stderr jeté. La confirmation affiche un avertissement en anglais et en français. Commit `b646c9f`.
- Preuves : `cargo test --workspace` passe 241 tests, `cargo clippy --workspace --all-targets` et `cargo fmt --check` propres. Les deux commits restent locaux.
- Restent les constats F7 à F15 du rapport.

---

## 2026-11-07, audit de sécurité : F3 et F4 corrigés

- F3, masquage dans `bramblers-hostrules` : `PASSWORD=12345678` est masqué, `API_PORT=8080` reste lisible. `scrub_text` masque désormais les formes connues partout dans un texte, les affectations `KEY=value` et `KEY: value`, et les paramètres d'URL. Formes ajoutées : Stripe `sk_live_` et `rk_live_`, Slack `xapp-`, Shopify `shpat_`, SendGrid `SG.`, Twilio `SK…`. Commit `36bc3ed`.
- F4, MCProster : `command` passe par le masquage par drapeau, et chaque argument est classé avec le drapeau qui le précède. `--api-key VALUE` masque VALUE même sans forme connue. Test d'inventaire avec un jeton de 32 caractères hors formes connues. Commit `6a6cc20`.
- Preuves : `cargo test --workspace` passe (240 tests, dont 3 dans `secrets` et 1 dans l'inventaire), `cargo clippy --workspace --all-targets` et `cargo fmt --check` propres. Les deux commits de ce lot restent locaux ; le lot précédent est sur `origin/main`.
- Restent les constats F5 à F15 du rapport.

---

## 2026-11-07, audit de sécurité : F1 et F2 corrigés

- Audit statique des 18 crates (43 201 lignes) : rapport hors git dans `00_cadrage/2026-11-07_audit_securite.md`, 15 constats. Pas de faille critique ; en tête, les secrets de la sonde MCProster et le script généré par Cleanator.
- F1, sonde MCProster : `ProbeResult` ne porte plus la ligne de commande ni `stderr`, qui pouvaient contenir un `${VAR}` développé ; l'erreur est masquée, et la commande n'est développée que pour le processus enfant. Tests : jeton dans un argument, commande en échec. Commit `18d8dee`.
- F2, script Cleanator : `display` et `rule_id` passent par `one_line` (contrôles remplacés, une seule ligne), et tous les champs substitués sont cités. Tests : nom avec saut de ligne, nom avec `';`. Commit `059c1ce`.
- Preuves : `cargo test --workspace` passe (236 tests, dont 2 dans la sonde et 1 dans le script), `cargo clippy -p bramblers --all-targets` et les deux cores propres, `cargo fmt --check` propre. Rien n'a été poussé.
- Restent les constats F3 à F15 du rapport, dont le masquage des valeurs numériques et des jetons sans forme connue.

---

## 2026-11-07, About du dépôt et profil .github à niveau

- About de `blackberrylabs/BrambleRS` refait avec l'accord d'Alexandre (« go ») : description de 195 caractères (« Shows what AI coding agents do to a Linux machine, most serious first, on one screen: ... Read-only, one binary, six modules. Rust. »), page d'accueil vers la page des releases, topics curés à 18 (retirés `blackberrylabs`, `bramble-rs`, `bramblers` ; ajoutés `instruction-files`, `cost-tracking`, `session-history`). Preuve : `gh repo view blackberrylabs/BrambleRS --json description,homepageUrl,repositoryTopics`.
- `.github` : `AGENTS.md` suit `BrambleRS/PLAN.md` et ne parle plus de `github_meta.sh` (supprimé) ; le profil et le README corrigent la promesse d'écriture (préférences des vues, sondes MCProster, script Cleanator) et mentionnent les six écrans. Commit `a0b4031`.
- BrambleRS : le README liste les écritures réelles, commit `c121177`. Ce journal suit dans le même push.
- Aperçu social : réglage manuel restant, `assets/social-preview.png` fait 1280 × 640.

---

## 2026-11-07, docs vérifiées pour la passation

- Reprise une par une des cinq demandes : AGENTS (état, dernier commit de code `0c46011`, six vues, `target/` supprimé), PLAN (note des six vues, 1.0 à constater), JOURNAL (entrée de passation), BRIEF (D8 remplace D4), `Logiciels/AGENTS.md` et `Logiciels/PASSATION.md`, README, CHANGELOG et profil `.github`. Tout était déjà en place et poussé (`07c3ada`, `51e3fcc`).
- Trois formulations devenues fausses corrigées : « `view` (à créer en 0.x) » dans `AGENTS.md`, « 0.1.6 sans paquet ni release » (vrai pour cette version seule), et la règle multi-dépôts de `Logiciels/AGENTS.md` qui ne vaut plus pour `.github`. Les deux titres qui nommaient encore la série sont neutralisés. Le README liste les six crates de vue ; `PASSATION.md` signale que sa section 7 vise des dépôts supprimés.
- Preuve : `git diff --stat` (AGENTS.md, BRIEF.md, README.md) ; aucun `.rs` touché, `cargo` non relancé.
- BRIEF corrigé avec son accord (« go, corriges ») : la crate de l'app est `crates/bramblers`, le portage est daté en 0.1.1 à 0.1.6 (workspace, sans paquet ni tag), les anciens tags `v0.1.0` à `v0.4.0` sont dits supprimés (D8) et l'ordre de migration ne parle plus d'archivage. Amendement daté en tête du BRIEF, sections 4, 8 et 9.

---

## 2026-11-07, passation

- État : workspace 0.1.6, six vues portées (Permitt, MCProster, Linstruct, Cleanator, Billfold, Lastrun), 233 tests, fmt et clippy propres. Dépôt unique de 60 Mo, `target/` et `dist/` supprimés.
- Publié : seulement la 0.1.0 (`bramblers-v0.1.0`). Les versions 0.1.1 à 0.1.6 ne sont ni packagées ni taguées. Les six captures sont dans `docs/screenshots/`.
- GitHub : `blackberrylabs/BrambleRS` et `blackberrylabs/.github` ; `MiningControlRS` est un projet privé sans rapport. Les dix anciens dépôts et les cinq releases de la tour ont été supprimés (D8).
- Prochaine session : vérifier les six vues à l'écran, constater la parité 1.0, puis construire les paquets, le tag et la release. Le premier build sera long, `target/` ayant été supprimé.
- Réglages manuels restants : épingler BrambleRS sur le profil, choisir l'aperçu social. Restes techniques : `.rpm` sur une machine RPM, anciennes captures de `docs/screenshots/`, `assets/blackberrylabs.desktop`.

---

## 2026-11-07, dépôt unique : anciens dépôts et checkouts supprimés

- Décision d'Alexandre, mots écrits : « le repo local et github doivent être remis au propre sans archive ». D8 remplace D4 : les dix anciens dépôts, leurs releases et les cinq tags de la tour sont supprimés, pas archivés. L'historique du code reste dans BrambleRS, les sources des apps restent dans `00_cadrage/references/`.
- Local, 71 Go libérés : les dix checkouts (`Bbl_hostrules`, `Bbl_report`, `Bbl_ui`, `Cleanator`, `Mcproster`, `Permitt`, `Linstruct`, `Billfold`, `Lastrun`, `Template`, tous propres), le dossier vide `Hostrules`, `target/` (46 Go), `dist/` (22 Mo), la branche `bramble-rs` (fusionnée) et les tags locaux `v0.1.0` à `v0.4.0`. Gardés : `BrambleRS`, `Blackberrylabs_github` (profil `.github`), `Eliot`, `00_cadrage/`.
- GitHub : les dix dépôts archivés (`bbl-hostrules`, `bbl-report`, `bbl-ui`, `cleanator`, `mcproster`, `permitt`, `linstruct`, `billfold`, `lastrun`, `template`) supprimés avec `gh repo delete`. Les cinq releases `Blackberrylabs 0.1.0` à `0.4.0` et leurs tags `v0.1.0` à `v0.4.0` supprimés de BrambleRS. Restent `BrambleRS`, `.github` et `MiningControlRS`.
- Textes corrigés avec leur date : BRIEF (D4 remplacée par D8, sections 5, 6, 9), PLAN (étape 8, notes des six vues), AGENTS, README, CHANGELOG, profil de l'organisation. Le `delete_repo` du jeton GitHub a été autorisé par Alexandre.
- Preuves : `gh repo list blackberrylabs` ne rend plus que `BrambleRS`, `MiningControlRS` et `.github` ; `gh release list -R blackberrylabs/BrambleRS` ne rend que `BrambleRS 0.1.0` ; `git ls-remote --tags origin` ne rend que `bramblers-v0.1.0` ; `ls /home/nushoko/Bureau/Logiciels` ne rend que `Blackberrylabs_github`, `BrambleRS` et `Eliot` ; le dépôt local fait 60 Mo.
- Rien n'a été poussé pour ces corrections de textes au moment de l'entrée. Elles ont ensuite été poussées dans BrambleRS (`75a4b75`) et dans `.github` (`51e3fcc`).

---

## 2026-11-07, la vue Lastrun portée dans son onglet

- La crate `crates/modules/lastrun/view` (`bramblers-lastrun-view`) reprend l'ancienne app (2 086 lignes) sans fenêtre : `View::new`, `update`, `view`, `set_lang`, `set_dark`, `refresh`. Quatre onglets à gauche (Sessions, Supprimés, Paquets, Refusés), les constats au milieu, le constat ou la session choisi à droite avec sa chronologie, ses filtres, le diff d'une édition à la demande.
- La préparation du scan reste dans le fil de lecture : quels fichiers supprimés existent de nouveau, quelles actions sortent du projet, quelle commande a produit une suppression déduite, quels constats destructifs par session. La vue ne touche pas au disque à chaque image.
- Agencement adapté au rail : en-tête sur deux lignes (phrase, puis 30/90 jours et « Relire »), puces d'hôtes en défilement horizontal, boutons d'action sur deux colonnes. À 1 280 px, chaque colonne perd environ 120 px par rapport à l'ancienne fenêtre.
- La fenêtre de 30 ou 90 jours va dans `~/.config/bramblers/lastrun/settings.json` ; l'ancien `~/.config/lastrun/settings.json` est lu une fois. Les invites, les réponses et la sortie des commandes ne sont jamais lues ; seules les actions le sont.
- Preuves : `cargo test --workspace` passe (233 tests, dont 3 dans la vue), fmt et clippy propres ; capture `docs/screenshots/lastrun.png` (1280x820) sur le HOME factice, avec une session `claude-jsonl` contenant deux commandes, une édition refusée et un paquet npm ; `BRAMBLERS_MODULE=lastrun` lancé sans panique.
- Version du workspace passée à 0.1.6. Commit `0c46011` pour la vue. Rien n'a été poussé.

---

## 2026-11-07, la vue Billfold portée dans son onglet

- La crate `crates/modules/billfold/view` (`bramblers-billfold-view`) reprend l'ancienne app (1 542 lignes) sans fenêtre : `View::new`, `update`, `view`, `set_lang`, `set_dark`, `refresh`. Trois onglets : Branchement (hôtes, comptes, crédit, quotas, expiration), Clés (masquées, fichier, état git, hôtes qui les lisent) et Compteur (jetons par hôte, modèle, projet ou jour, dollars équivalent API, part du cache, journaux à venir).
- La liste des constats garde les quatre gravités, les puces d'hôtes, la recherche et l'affichage des informations ; le détail donne le sens, l'emplacement, la commande de l'hôte et la source.
- Agencement adapté au rail : la phrase d'en-tête passe sur deux lignes avec les fenêtres 30/90/365 et « Relire » en dessous ; les puces d'hôtes défilent horizontalement au lieu de revenir à la ligne tous les cinq ; les pastilles d'une même ligne de compte s'empilent. À 1 280 px, la colonne de gauche perd 120 px par rapport à l'ancienne fenêtre.
- Le réglage de la fenêtre (30, 90 ou 365 jours) va dans `~/.config/bramblers/billfold/settings.json` ; l'ancien `~/.config/billfold/settings.json` est lu une fois. Les clés sont masquées à la lecture, rien n'est conservé.
- Preuves : `cargo test --workspace` passe (230 tests, dont 3 dans la vue), fmt et clippy propres ; capture `docs/screenshots/billfold.png` (1280x820) sur le HOME factice, avec `.claude/.credentials.json`, deux lignes d'usage `claude-jsonl` et une clé `ANTHROPIC_API_KEY` dans `.bashrc` ; `BRAMBLERS_MODULE=billfold` lancé sans panique.
- Version du workspace passée à 0.1.5. Commit `09078b2` pour la vue. Rien n'a été poussé.

---

## 2026-11-07, la vue Cleanator portée dans son onglet

- La crate `crates/modules/cleanator/view` (`bramblers-cleanator-view`) reprend l'ancienne app (1 858 lignes en deux fichiers) sans fenêtre : `View::new`, `update`, `view`, `set_lang`, `set_dark`, `refresh`, plus `subscription` pour le compteur de la fenêtre de lecture. L'explorateur est un treemap squarifié en canvas (Bruls, Huizing et van Wijk), avec la légende des catégories ; le panneau garde les recommandations sûres et à vérifier, les groupes de règles, la case par projet, le script copié ou enregistré.
- Les onglets Explorateur, Assistants IA et Projets sont repris tels quels. La coquille gagne un abonnement (`App::subscription`) qui transmet celui de la vue Cleanator quand son onglet est ouvert : sans lui, les compteurs de lecture resteraient figés pendant le scan.
- Le script enregistré va dans `~/.local/state/bramblers/cleanator/cleanup.sh`, pas dans le dossier personnel comme l'ancienne app : la promesse « rien hors de `~/.config/bramblers/` et `~/.local/state/bramblers/` » tient. Le chemin est affiché dans la ligne d'état.
- Écarté : `--select` et le dossier passé en argument (le choix d'onglet revient à `BRAMBLERS_MODULE`), `app.title` et `theme.toggle` (portés par la coquille). Le scan porte sur le dossier personnel.
- Bug corrigé dans le core : la sonde Docker soustrayait `unique_size` de `size` sans garde ; sur une machine où les couches partagées dépassent la taille de l'image, le scan paniquait (`probes.rs:425`). `saturating_sub` remplace la soustraction.
- Preuves : `cargo test --workspace` passe (227 tests, dont 3 dans la vue et 2 dans le treemap), fmt et clippy propres ; capture `docs/screenshots/cleanator.png` (1280x820) sur le HOME factice, avec `node_modules`, `target`, `.venv` et `.cache/pip` ; `BRAMBLERS_MODULE=cleanator` lancé sans panique.
- Version du workspace passée à 0.1.4. Commit `a5a34f6` pour la vue. Rien n'a été poussé.

---

## 2026-11-07, la vue Linstruct portée dans son onglet

- La crate `crates/modules/linstruct/view` (`bramblers-linstruct-view`) reprend l'écran de l'ancienne app (1 423 lignes) sans fenêtre : `View::new`, `update`, `view`, `set_lang`, `set_dark`, `refresh`. Trois colonnes : les cartes d'hôtes (fichiers dans l'ordre de l'hôte, jauge de budget, tailles, états `loaded`, `on demand`, `cut`, `skipped`, `shadowed`, `ignored`, section « et si » pour les hôtes absents), la liste des constats (gravités, puces d'hôtes, recherche, informations) et le détail (sens, chemin, commande de vérification de l'hôte, source).
- Les dossiers de projets viennent des préférences de la vue, passées en mémoire au scan : `projects::roots_from_dirs` est ajouté au core, et `default_roots` s'appuie dessus. L'ancien `~/.config/linstruct/settings.json` est lu une fois par `Prefs::load_with_fallback("bramblers/linstruct", "linstruct")`.
- Écarté : `--select` et les racines passées en arguments (le choix d'onglet revient à `BRAMBLERS_MODULE`), `app.title` et `theme.*` (portés par la coquille). Les trois colonnes passent en 6/5/4 au lieu de 4/4/4 : à 1280 px, le rail laisse 1 042 px et les 340 px par colonne cassaient les libellés d'état.
- Preuves : `cargo test --workspace` passe (222 tests, dont 4 dans la vue), fmt et clippy propres ; capture `docs/screenshots/linstruct.png` (1280x820) sur le HOME factice, avec `~/.claude/CLAUDE.md` (import manquant, clé en clair), le `CLAUDE.md` et l'`AGENTS.md` du projet, `.cursor/rules/style.mdc` et `.github/copilot-instructions.md` ; `BRAMBLERS_MODULE=linstruct` lancé sans panique.
- Version du workspace passée à 0.1.3. Commit `1add611` pour la vue. Rien n'a été poussé.

---

## 2026-11-07, la vue MCProster portée dans son onglet

- La crate `crates/modules/mcproster/view` (`bramblers-mcproster-view`) reprend l'écran de l'ancienne app (1 367 lignes) sans fenêtre : `View::new`, `update`, `view`, `set_lang`, `set_dark`, `refresh`. Contenu : quatre chiffres, filtres (recherche nom, paquet, URL ou fichier ; hôte ; constats ; désactivés et anciens), tableau serveurs × hôtes (une colonne par hôte jusqu'à six, sinon une colonne « déclaré dans »), détail des déclarations (fichier, commande, environnement, en-têtes, secrets masqués), sonde avec confirmation, coût en contexte par hôte, constats et actions.
- La sonde est portée telle quelle : un test à la fois, `initialize` et `tools/list`, dix secondes, arrêt du processus. La confirmation reste, avec « ne plus demander ».
- Écarté : `--select` (le choix d'onglet revient à `BRAMBLERS_MODULE`), `app.title` et `theme.*` (portés par la coquille). Les filtres passent sur deux lignes, comme Permitt, et le tableau prend 15/22 de la largeur ; les colonnes fixes sont réduites (transport 60, version 90, tokens 64) pour que les colonnes d'hôtes tiennent dans 1280 px.
- Les résultats de sonde ne vont plus dans `~/.config/mcproster/probes.json`, hors des chemins promis : ils s'écrivent dans `~/.config/bramblers/mcproster/probes.json`, via `load_results_at` et `save_results_at` ajoutés au core. L'ancien fichier est lu une fois.
- Preuves : `cargo test --workspace` passe (218 tests, dont 5 dans la vue et 1 nouveau dans le core), fmt et clippy propres ; capture `docs/screenshots/mcproster.png` (1280x820) sur le HOME factice, avec `mcpServers` dans `.claude.json`, `.cursor/mcp.json`, le `.mcp.json` du projet et `mcp` dans `opencode.json` ; `BRAMBLERS_MODULE=mcproster` lancé sans panique.
- Version du workspace passée à 0.1.2. Commit `aa98290` pour la vue. Rien n'a été poussé.

---

## 2026-11-07, la vue Permitt portée dans son onglet

- La crate `crates/modules/permitt/view` (`bramblers-permitt-view`) reprend l'écran de l'ancienne app (1 023 lignes) sans fenêtre ni `iced::application` : `View::new` rend l'état et la tâche de lecture, puis `update`, `view`, `set_lang`, `set_dark`, `refresh`. L'ancienne app n'est plus qu'une référence dans `00_cadrage/references/permitt/`.
- Contenu retrouvé : quatre chiffres d'en-tête, recherche, portée utilisateur/projet, « installés seulement », « constats seulement », case « fichiers shell », tableau hôtes × quatre capacités avec ligne de contexte, volet de détail (précédence, preuves de la plus forte à la plus faible, preuves ignorées, lignes de shell, conseil « ce que vous pouvez faire », autres capacités cliquables), liste des constats, copie et ouverture de fichier ou de lien. Le statut (« 12 fichiers lus, … ») reste au bas de la vue : la coquille n'a pas de pied de page.
- Branchement : la vue est construite à la première ouverture de l'onglet, garde son état ensuite, et reçoit la langue et le thème de la coquille (`set_lang`, `set_dark`). `Message::Permitt` délègue ; « Refresh » en en-tête relance aussi la vue quand son onglet est ouvert. `BRAMBLERS_MODULE=permitt` l'ouvre dès le démarrage, même avec `BRAMBLERS_NO_REFRESH=1`.
- La vue est rendue dans le contenu de la coquille, sous l'en-tête et à droite du rail : les onglets restent visibles et l'onglet Permitt reste actif. Un premier essai renvoyait la vue seule en plein écran ; corrigé par le commit `5750408`.
- Écarté : `--select` et `PERMITT_SELECT` (le choix d'onglet revient à `BRAMBLERS_MODULE`) ; `app.title`, `theme.dark` et `theme.light` sortent des `locales` de la vue, la coquille les porte. Les textes vivent dans `crates/modules/permitt/locales/` ; tous les autres sont repris mot pour mot.
- Réglages : `Prefs::load_with_fallback("bramblers/permitt", "permitt")` lit une fois l'ancien `~/.config/permitt/settings.json` ; `read_shell` est réécrit dans `~/.config/bramblers/permitt/settings.json`. La langue et le thème ne sont plus enregistrés par la vue.
- La rangée de filtres passe sur deux lignes (recherche et « Rescan » d'abord, portée et cases ensuite) : le rail retire 238 px à la colonne, la recherche n'avait plus que quelques dizaines de pixels. Même contenu, même comportement, seul l'agencement change (`1c2040a`).
- Capture ajoutée : `docs/screenshots/permitt.png` (1280x820), prise sur un HOME factice dans `/tmp`, jamais sur les projets d'Alexandre. Le jeu de démonstration contient `.claude/settings.json` (`acceptEdits`, une liste autorisée), `.claude.json` avec un projet, `work/demo-app/.claude/settings.json`, `.config/opencode/opencode.json` et un `.bashrc` avec l'alias `--dangerously-skip-permissions`. Rendu logiciel (`WGPU_BACKEND=gl`) : après l'arrêt forcé de la première instance, le GPU AMD perdait le contexte au démarrage suivant.
- Preuves : `cargo test --workspace` passe (212 tests, dont 5 dans la vue : clés des locales, combinaison des filtres, portée d'un constat de projet, choix de la preuve affichée, chaque type de constat a son texte), `cargo fmt --check` et `cargo clippy --all-targets` propres. La capture montre le rail, l'onglet Permitt actif, les quatre chiffres, le tableau, les constats et la ligne de statut.
- Version : Alexandre tranche le 2026-11-07, la vue Permitt répare une fonction perdue de la 0.1.0, la prochaine version est donc `0.1.1`. Le workspace, le `CHANGELOG` et le plan sont en 0.1.1 ; les vues suivantes sont décalées en révisions : MCProster 0.1.2, Linstruct 0.1.3, Cleanator 0.1.4, Billfold 0.1.5, Lastrun 0.1.6. Accord écrit pour réécrire `main` et pousser en force (« je veux un historique propre ») : les commits de la 0.2.0 et des captures sont fondus dans ce commit `0.1.1`. Paquets, tag et release ne sont pas faits.

---

## 2026-11-07, un dossier par module, plan des six vues

- Décision d'Alexandre : tout le produit vit dans `BrambleRS/`, avec un dossier par module, et chaque onglet doit retrouver toutes les fonctionnalités de son ancienne app. L'app devient une coquille (en-tête, rail, routeur) ; chaque module reçoit un `view/` à côté de son `core/`.
- Arborescence renommée pour que chaque dossier porte le nom de son paquet : `crates/bramblers` (app), `crates/bramblers-core`, `crates/bramblers-cli`. Les trois fondations et les six dossiers de modules ne bougent pas. Les chemins de dépendance, les fixtures du CLI et `packaging/build.sh` suivent.
- Les sources des six anciennes apps et leurs `locales` sont copiées dans `00_cadrage/references/<id>/` (hors git) avant tout ménage. Les anciens checkouts restent dans `Logiciels/` jusqu'au portage de leur vue, puis seront supprimés un par un, avec accord.
- Le plan des six vues est écrit dans `00_cadrage/2026-11-07_plan_vues_modules.md` : contrat d'une vue, ordre (Permitt, MCProster, Linstruct, Cleanator, Billfold, Lastrun), inventaire écran par écran et critères de fin. `PLAN.md` gagne l'étape 9.
- Aucun changement de comportement pour la 0.1.0 : `cargo test --workspace` passe (207 tests), fmt et clippy propres. Le binaire installé et la release restent la 0.1.0.
- Rien n'a été poussé pour ce lot.

---

## 2026-11-07, 0.1.0 publiée, série archivée, dossier renommé

- `main` poussé sur `blackberrylabs/blackberrylabs`, puis le dépôt renommé `blackberrylabs/BrambleRS` (le remote local pointe sur `git@github.com:blackberrylabs/BrambleRS.git`). Tag annoté `bramblers-v0.1.0` poussé.
- Release `BrambleRS 0.1.0` publiée avec `bramblers_0.1.0-1_amd64.deb`, `bramblers-0.1.0-1.x86_64.rpm`, `bramblers-0.1.0-x86_64.AppImage` et `SHA256SUMS`. Vérification depuis l'extérieur : `gh release download` puis `sha256sum -c` passe sur les trois fichiers.
- Les dix anciens dépôts (trois fondations, six modules, template) ont reçu un bandeau en tête de README, poussé sur `main`, puis ont été archivés. Ils ne sont pas supprimés et leurs releases restent téléchargeables.
- Profil de l'organisation et dépôt `.github` réécrits pour BrambleRS ; description et blog de l'org mis à jour ; description et topics du dépôt `BrambleRS` mis à jour. `vibe-coding` retiré des topics. Restent deux réglages manuels : épingler BrambleRS sur le profil et choisir l'aperçu social.
- `packaging/github_meta.sh`, écrit pour les douze dépôts, est supprimé. Son rôle est terminé.
- Le dossier local `Logiciels/Blackberrylabs` est renommé `Logiciels/BrambleRS` ; la session a suivi. `git status`, le remote et `bramblers --version` vérifiés au nouvel emplacement.
- Preuves : `gh repo view blackberrylabs/BrambleRS`, `gh release view bramblers-v0.1.0`, `gh repo view` des dix dépôts (`isArchived: true`), `sha256sum -c` sur les fichiers téléchargés.

---

## 2026-11-07, interface à rail gauche

- Demande d'Alexandre : la même structure que MiningControlRS, avec les modules en onglet à gauche. L'écran passe d'une pile de cartes à un en-tête, un rail de 210 px et un contenu à droite.
- Le rail porte « All findings » puis les six modules, avec le nombre de constats et un point à la gravité la plus forte. L'onglet actif est une pastille discrète. Le pied du rail garde la langue, le thème et le nombre de rapports lus.
- En haut : le nom BrambleRS, la version, l'état des rapports, le bouton de rafraîchissement. Dans le contenu : les quatre chiffres de gravité, la liste des constats et le détail. Sur un onglet de module, les chiffres, la liste et le détail sont limités à ce module ; le détail montre ses chiffres, ses constats par gravité, l'âge du rapport et « Refresh ».
- Disparaissent : les six cartes de modules, les puces de filtre et le pied de page global. `BRAMBLERS_MODULE=<id>` ouvre directement un onglet (captures) ; `BRAMBLERS_SELECT` et `BRAMBLERS_NO_REFRESH` restent.
- La palette noire et violette de MiningControlRS n'a pas été copiée : BrambleRS garde ses couleurs de gravité et les thèmes iced clair/sombre. La structure est la même, l'identité visuelle reste la sienne.
- Preuves : captures de l'aperçu en sombre et en clair, et de l'onglet Permitt ; `cargo test --workspace` passe (207 tests), fmt et clippy propres. Les paquets seront reconstruits après ce changement.
- Rien n'a été poussé.

---

## 2026-11-07, README, shim, suppressions, paquet installé

- `README.md` réécrit pour BrambleRS : six modules, read-only, installation, commandes, architecture du workspace. `CHANGELOG.md` remplacé par l'entrée 0.1.0, avec la limite explicite : les vues détaillées des six anciennes apps arrivent en 0.x. Contrôle de style : aucun tiret cadratin, aucun mot de la liste noire.
- Shim : `Prefs::load_with_fallback("bramblers", "blackberrylabs")` lit une fois les réglages de l'ancienne app et écrit ensuite dans `~/.config/bramblers/`. Les rapports `bbl-report/1` trouvés dans l'ancien dossier d'état sont ignorés au lieu d'apparaître comme des échecs.
- Suppressions avec l'accord d'Alexandre : `crates/core/src/{discover,spec,run,packages}.rs`, les six `modules/*.toml`, `docs/new-module.md`, `packaging/bundle.sh`, 27 fichiers de pilotage et README importés dans `crates/bramblers-*`, `dist/` (400 Mo) et `packaging/.bundle/` (152 Mo).
- `packaging/build.sh` lancé trois fois. Produits dans `dist/` : `bramblers_0.1.0-1_amd64.deb`, `bramblers-0.1.0-1.x86_64.rpm`, `bramblers-0.1.0-x86_64.AppImage`, `SHA256SUMS` ; `sha256sum -c` passe sur les trois.
- `.deb` installé par `apt install ./dist/bramblers_0.1.0-1_amd64.deb` : `blackberrylabs-full` 0.4.0 retiré, `bramblers 0.1.0` installé. `bramblers report permitt --json` rend 16 constats. L'app lancée 8 s écrit Billfold, Cleanator, MCProster et Permitt dans `~/.local/state/bramblers/` avant d'être arrêtée. L'AppImage lancée 8 s démarre aussi. `desktop-file-validate` propre après passage de `Categories` à `Utility`.
- Le `.rpm` est construit mais pas installé : `rpm` n'est pas présent sur cette machine. À vérifier sur une machine RPM.
- Reste : GitHub (renommer le dépôt, bandeaux et archivage des dix anciens, profil d'org, release 0.1.0, tag `bramblers-v0.1.0`), dossier local `BrambleRS`. `packaging/github_meta.sh` est encore écrit pour les douze dépôts.
- Rien n'a été poussé.

---

## 2026-11-07, `build.sh` réécrit pour le paquet unique

- `packaging/build.sh` ne construit plus que `bramblers` : un binaire release, un `.deb`, un `.rpm`, une AppImage `bramblers-<version>-<arch>.AppImage`. Les variantes `full`, l'appel à `bundle.sh` et les chemins `bbl`/`blackberrylabs` ont disparu.
- Les métadonnées de `crates/app/Cargo.toml` déclarent `Conflicts`, `Replaces` et `Provides` sur `blackberrylabs`, `blackberrylabs-full` et les six anciens paquets de modules ; côté RPM, `Conflicts`, `Obsoletes` et `Provides`. Le `.desktop` et les icônes sont ceux de `bramblers`.
- Vérifié sans lancer : `bash -n packaging/build.sh`, `cargo metadata` et `cargo check --workspace`. Le script n'a pas été exécuté : il efface `target/appimage/bramblers.AppDir` et réécrit `dist/SHA256SUMS`, et son lancement demande l'accord d'Alexandre.
- Reste avant la 0.1.0 : lancer le script, installer le paquet, écrire le shim des anciens chemins de config et d'état, réécrire le `README.md` racine encore écrit pour l'ancienne tour.
- Rien n'a été poussé.

---

## 2026-11-07, un seul binaire `bramblers`

- L'app et la ligne de commande sont désormais le même exécutable : sans argument, l'écran iced ; avec un argument, la CLI. `bramblers report --json` produit le rapport global (tableau de bord), `bramblers report <module> --json` le rapport d'un module, `list` lit les rapports enregistrés, `refresh` relance les six modules. `modules` disparaît, les six modules étant fixes. Commit `a49babc`.
- `crates/cli` devient une bibliothèque (`bramblers-cli`, `src/lib.rs`) liée dans le binaire ; `src/main.rs` est déplacé, aucun contenu perdu (l'ancien fichier reste dans l'historique). Les métadonnées de paquet n'installent plus que `bramblers`.
- Mesure sur la machine : Lastrun en processus met 128 s en compilation debug sur 201 Mo de transcriptions, contre 8 s pour le binaire release installé. Sa limite passe de 120 à 300 s, avec la mesure en commentaire ; les autres modules restent sous leurs limites (Billfold 5 s, Linstruct 15 s, MCProster et Permitt sous la seconde).
- Preuves : `cargo test --workspace` (205 tests), fmt et clippy propres ; `bramblers --help` et `bramblers report --help` ; `report <module> --json` rendu conforme `bramblers-report/1` pour Billfold, Linstruct, MCProster et Permitt, un seul fichier d'état par module.
- Reste : paquet `bramblers` (build.sh, shim des anciens chemins), archivage GitHub, renommage du dossier local. Rien n'a été poussé.

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
