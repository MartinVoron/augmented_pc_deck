# CLAUDE.md

Augmented PC Deck : tableau de bord de bureau local pour Windows, façon Stream Deck
(raccourcis d'applications, gestionnaire de tâches, minuteurs, to-do, musique en cours,
GitHub, Gmail, notifications Claude). Stack : Tauri 2 (Rust) + Svelte.

Langue : on échange et on documente en français. Le code (identifiants) reste en anglais.

## Règles Git (obligatoires)

Le détail est dans `CONTRIBUTING.md`. En résumé, Claude doit :

1. **Ne jamais commit ni pousser sur `main`.** Toujours travailler sur une branche
   `<type>/<kebab-case>` (ou `claude/<kebab-case>`) créée depuis `main` à jour.
2. **Écrire chaque commit au format Conventional Commits** :
   `<type>(<scope>)!: <description>`, en-tête ≤ 72 caractères, description à l'impératif,
   en minuscule, sans point final.
   - Types : `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`
   - Scopes : `ui`, `tauri`, `launcher`, `processes`, `timer`, `todo`, `music`, `github`, `gmail`, `claude`, `settings`, `deps`
   - Changement cassant : `!` après le type/scope + pied `BREAKING CHANGE: …`
3. **Faire des commits petits et atomiques** : un changement logique par commit.
4. **Passer par une Pull Request** vers `main` dont le titre suit Conventional Commits
   (fusion en squash). Remplir `.github/pull_request_template.md`.
5. **Ne jamais utiliser** `--no-verify`, `git push --force` sur une branche partagée,
   ni réécrire l'historique de `main`. Sur sa propre branche : `--force-with-lease` seulement.
6. Ne jamais commit de secrets (`.env`, tokens GitHub/Gmail, identifiants OAuth).

Avant de commit, vérifier un message avec :

```sh
printf '%s\n' "feat(timer): ajouter un minuteur" > /tmp/msg && .githooks/commit-msg /tmp/msg
```

Activer les hooks dans un nouveau clone : `git config core.hooksPath .githooks`.
