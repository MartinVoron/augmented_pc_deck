# CLAUDE.md

Augmented PC Deck : tableau de bord de bureau local pour Windows, façon Stream Deck
(raccourcis d'applications, gestionnaire de tâches, minuteurs, to-do, musique en cours,
GitHub, Gmail, notifications Claude). Stack : Tauri 2 (Rust) + Svelte.

## Langue

**Tout est en français, simple et lisible** : échanges, documentation, issues, noms de
branches, messages de commit, titres et descriptions de PR, commentaires GitHub.
Seuls les identifiants du code (variables, fonctions, fichiers) restent en anglais.

## Règles Git (obligatoires)

Le détail est dans `CONTRIBUTING.md`. En résumé, Claude doit :

1. **Partir d'une issue.** Pas de travail sans issue GitHub. Si aucune issue ne couvre la
   demande, en proposer une (ou la créer si l'utilisateur l'a demandé) avant de coder.
2. **Ne jamais commit ni envoyer sur `main`.** Branche `<type>/<numéro-issue>-<description>`
   en minuscules, sans accents, par exemple `ajout/12-minuteur-pomodoro`.
   Si l'environnement impose une branche `claude/…`, l'utiliser.
3. **Messages de commit en français** : `<type>(<zone>): <description>`, 72 caractères maximum,
   description à l'infinitif, sans majuscule initiale ni point final.
   - Types : `ajout`, `correction`, `doc`, `style`, `refonte`, `perf`, `test`, `build`, `ci`, `maintenance`, `annulation`
   - Zones : `interface`, `tauri`, `lanceur`, `processus`, `minuteur`, `taches`, `musique`, `github`, `gmail`, `claude`, `reglages`, `deps`
   - Changement cassant : `!` après le type ou la zone, et `CHANGEMENT CASSANT : …` dans le corps
4. **Petits commits atomiques** : un changement logique par commit.
5. **Chaque PR est reliée à une issue** : titre au format d'un commit, modèle
   `.github/pull_request_template.md` rempli, ligne `Closes #<numéro>` obligatoire.
   Fusion en squash.
6. **Ne jamais utiliser** `--no-verify`, `git push --force` sur une branche partagée, ni
   réécrire l'historique de `main`. Sur sa propre branche : `--force-with-lease` seulement.
7. Ne jamais commit de secrets (`.env`, jetons GitHub/Gmail, identifiants OAuth).

Vérifier un message avant de commit :

```sh
printf '%s\n' "ajout(minuteur): ajouter un minuteur pomodoro" > /tmp/msg && .githooks/commit-msg /tmp/msg
```

Activer les hooks dans un nouveau clone : `git config core.hooksPath .githooks`.
