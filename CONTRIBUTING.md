# Contribuer à Augmented PC Deck

Ce document fixe les règles de branches, de commits et de push du projet.
Les mêmes règles s'appliquent aux humains et à Claude (voir `CLAUDE.md`).

## 1. Mise en place (une fois après le clone)

```sh
git config core.hooksPath .githooks
```

Cela active deux hooks locaux :

- `commit-msg` : refuse un message qui ne suit pas Conventional Commits ;
- `pre-push` : refuse un push direct sur `main`.

Sous Windows, Git for Windows exécute ces hooks via son shell intégré : rien d'autre à installer.

## 2. Branches

- `main` est toujours stable. **On ne commit ni ne pousse jamais directement dessus.**
- Tout travail se fait sur une branche courte, créée depuis `main` à jour :

```sh
git switch main && git pull
git switch -c feat/minuteur-pomodoro
```

Format : `<type>/<description-en-kebab-case>` (minuscules, chiffres, `-`, `.`, `_`).

| Préfixe      | Usage                                   | Exemple                     |
|--------------|-----------------------------------------|-----------------------------|
| `feat/`      | nouvelle fonctionnalité                  | `feat/widget-spotify`       |
| `fix/`       | correction de bug                        | `fix/crash-au-demarrage`    |
| `docs/`      | documentation                            | `docs/installation-windows` |
| `refactor/`  | restructuration sans changement visible  | `refactor/store-taches`     |
| `chore/`, `build/`, `ci/`, `test/`, `perf/`, `style/` | voir types de commit | `ci/build-windows` |
| `release/`   | préparation d'une version                | `release/v0.2.0`            |
| `claude/`    | branches créées par Claude Code          | `claude/ajout-minuteur`     |

Le nom de branche est vérifié par la CI (`scripts/check-branch-name.sh`).

## 3. Commits : Conventional Commits

Format de l'en-tête (72 caractères max) :

```
<type>(<scope>)!: <description>
```

- **type** (obligatoire) : `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`
- **scope** (facultatif, minuscules) : la zone touchée. Scopes conseillés :
  `ui`, `tauri`, `launcher`, `processes`, `timer`, `todo`, `music`, `github`, `gmail`, `claude`, `settings`, `deps`
- **`!`** : signale un changement cassant (à détailler avec `BREAKING CHANGE:` dans le pied)
- **description** : à l'impératif, sans majuscule initiale ni point final

Exemples :

```
feat(launcher): ajouter les raccourcis d'applications
fix(processes): corriger le tri par usage mémoire
docs: décrire l'installation sous Windows
chore(deps): mettre à jour tauri en 2.1
feat(settings)!: changer le format du fichier de configuration

BREAKING CHANGE: l'ancien config.json doit être migré.
```

Bonnes pratiques :

- un commit = un changement logique ; on commit souvent et petit ;
- corps du message (facultatif) après une ligne vide : le *pourquoi*, pas le *comment* ;
- référencer une issue dans le pied : `Closes #12`.

## 4. Push et Pull Requests

1. Pousser sa branche : `git push -u origin feat/ma-branche`
2. Ouvrir une Pull Request vers `main`, **avec un titre au format Conventional Commits**
   (c'est lui qui devient le message du commit sur `main`).
3. La CI `Conventions` vérifie : nom de branche, titre de PR, message de chaque commit.
4. Fusion en **Squash and merge** uniquement, puis suppression de la branche.

Interdits :

- push direct sur `main` ;
- `git push --force` sur `main` (sur sa propre branche, préférer `--force-with-lease`) ;
- contourner les hooks avec `--no-verify`, sauf urgence justifiée dans la PR.

## 5. Versions

Les versions suivent [SemVer](https://semver.org/lang/fr/) et se déduisent des commits :
`fix` → patch, `feat` → mineure, `!` / `BREAKING CHANGE` → majeure. Les tags sont au format `v0.1.0`.
