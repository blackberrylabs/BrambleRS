# JOURNAL — bbl-ui
<!-- Project journal (French), newest first. -->

---

## 2026-10-26, passation à OpenCode

- `AGENTS.md` ajouté : source unique des règles pour OpenCode et tout autre agent. OpenCode ne charge pas `CLAUDE.md` quand `AGENTS.md` existe (documentation d'OpenCode lue le 2026-10-26). `CLAUDE.md` devient un pointeur (`@AGENTS.md`) avec deux lignes propres à Claude. Le bloc commun d'`AGENTS.md` est identique dans les douze dépôts de la série.
- `opencode.json` ajouté : tout ce qui pousse, tague, supprime, installe ou touche une minuterie demande confirmation ; les fichiers `.env` sont refusés ; `BRIEF.md` et `LICENSE` demandent confirmation à l'édition.
- Preuve : état de la série daté du 2026-10-26 dans `Logiciels/PASSATION.md`. Demandé par Alexandre le 2026-10-26.

---

## 2026-10-16, extraction

- Crate ecrite en une passe ; une seule surprise : `card`/`inner` en `impl Into<Element>`
  rendaient les `.into()` des apps ambigus → signatures en `Element` comme avant.
  `contrast_text` garde la formule de Cleanator a l'identique (c'est l'original de la
  famille). Les trois apps migrees : Permitt et MCProster a 0 pixel pres, Cleanator pareil
  hors treemap (le faux home avait gagne un `.cache` entre les deux lancements). Le disque
  du cloud etait plein (44 Go de `target/`) : nettoye.
- Alexandre, pendant la migration : « a terme, Blackberrylabs doit etre la tour de controle
  d'un developpeur ou d'un vibe codeur sur son poste — gerer son disque, etc. » Note pour
  le BRIEF de la tour.

## 2026-10-16, naissance

- Soir de Permitt 0.1.0. Alexandre : Blackberrylabs est « un vrai petit projet avec plein
  d'outils MVP qu'on va ameliorer avec le temps » ; la tour de controle en est la porte
  d'entree. Avant de la construire — quatrieme app — on extrait le visage commun des trois
  premieres, comme la regle du matin le demande. Nom verifie libre, `bbl-ui`.
