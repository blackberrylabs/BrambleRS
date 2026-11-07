# JOURNAL — bbl-report
<!-- Project journal (French). Newest entry last. -->

## 2026-10-16 — Naissance

Le BRIEF de la tour de controle (valide ce soir) fixe le contrat `bbl-report/1` et decide
(T4) qu'il vit dans sa propre crate : ni dans `hostrules` (lecture d'hotes), ni dans `bbl-ui`
(interface). Troisieme fondation de la famille. BRIEF rev. 1 ecrit a partir de la section 4
du BRIEF de la tour ; deux decisions propres (P1 gravite en nombre nommee en mots, P2 pas de
dependance de date).

## 2026-10-16 — BRIEF valide, crate ecrite

BRIEF valide le soir meme. Crate ecrite dans la foulee : `src/lib.rs` (types, `Severity` en
nombre nomme en mots, `Tone` aux noms de la palette, validation qui liste tous les defauts,
`peek_schema`, `now_utc` sans dependance — la constante de test etait fausse d'un an, pas
l'algorithme, verifiee avec `date`), `src/conformance.rs` (`check`, `example`),
`tests/promises.rs`. 13 tests, clippy propre. Commit 1 prepare.

## 2026-10-16 — publie, trois modules le parlent

Depot public et tag v0.1.0 pousses par Alexandre (remote SSH). Dans la foulee, les trois
modules produisent un rapport : `permitt report`, `mcproster report`, `cleanator-cli report`,
chacun avec son test de conformite (`conformance::check` sur sa propre sortie) et son
mapping de gravite dans son CHANGELOG. Deux grains differents, assumes : Permitt et MCProster
un constat par constat ; Cleanator un constat par groupe de regle (sinon des dizaines de
lignes). Les trois apps prennent `--select`. Le contrat n'a pas bouge d'un champ : bon signe.
Question d'Alexandre : « on aurait pu mettre bbl-hostrules aussi ? » — recommande de garder
`hostrules` (crate generale, nom du metier) et de reserver `bbl-` a ce qui n'a de sens que
dans la famille ; a trancher par lui.

---

## 2026-10-26, passation à OpenCode

- `AGENTS.md` ajouté : source unique des règles pour OpenCode et tout autre agent. OpenCode ne charge pas `CLAUDE.md` quand `AGENTS.md` existe (documentation d'OpenCode lue le 2026-10-26). `CLAUDE.md` devient un pointeur (`@AGENTS.md`) avec deux lignes propres à Claude. Le bloc commun d'`AGENTS.md` est identique dans les douze dépôts de la série.
- `opencode.json` ajouté : tout ce qui pousse, tague, supprime, installe ou touche une minuterie demande confirmation ; les fichiers `.env` sont refusés ; `BRIEF.md` et `LICENSE` demandent confirmation à l'édition.
- PLAN : description et sujets du dépôt cochés (API publique de GitHub : description posée, 5 sujets).
- Preuve : état de la série daté du 2026-10-26 dans `Logiciels/PASSATION.md`. Demandé par Alexandre le 2026-10-26.
