# Contribuer à Augmented PC Deck

Ce document fixe les règles de travail avec Git. Elles valent pour tout le monde,
humains comme Claude (voir `CLAUDE.md`).

**Principe : tout est en français, simple et lisible.** Issues, noms de branches,
messages de commit, titres et descriptions de PR.

## 1. Mise en place (une fois après le clone)

```sh
git config core.hooksPath .githooks
```

Cela active deux vérifications locales :

- `commit-msg` refuse un message qui ne suit pas la convention ;
- `pre-push` refuse un envoi direct sur `main`.

Sous Windows, Git for Windows exécute ces hooks tout seul : rien d'autre à installer.

## 2. Le cycle de travail

1. **Une issue d'abord.** Tout travail part d'une issue GitHub qui décrit le besoin.
   Pas d'issue, pas de branche.
2. **Une branche par issue**, créée depuis `main` à jour.
3. **Des petits commits** au format ci-dessous.
4. **Une Pull Request reliée à l'issue**, fusionnée en « Squash and merge ».
   La fusion ferme l'issue automatiquement.

## 3. Branches

On ne travaille jamais directement sur `main`.

Format : `<type>/<numéro-issue>-<description>`

- en minuscules, mots séparés par des tirets ;
- **sans accents ni espaces** (Git et Windows les gèrent mal) ;
- courte : 2 à 5 mots.

```sh
git switch main && git pull
git switch -c ajout/12-minuteur-pomodoro
```

Exemples : `ajout/12-minuteur-pomodoro`, `correction/31-plantage-au-demarrage`,
`doc/4-installation-windows`.

Exceptions : `version/v0.2.0` (préparation d'une version), `claude/…` (branches créées
automatiquement par Claude Code).

## 4. Messages de commit

Format (72 caractères maximum) :

```
<type>(<zone>): <description>
```

**Types**

| Type          | Quand l'utiliser                               |
|---------------|------------------------------------------------|
| `ajout`       | nouvelle fonctionnalité                        |
| `correction`  | correction d'un bug                            |
| `doc`         | documentation uniquement                       |
| `style`       | mise en forme du code, sans changer le fonctionnement |
| `refonte`     | réorganisation du code, sans changement visible |
| `perf`        | amélioration des performances                  |
| `test`        | ajout ou modification de tests                 |
| `build`       | compilation, dépendances, empaquetage          |
| `ci`          | intégration continue (GitHub Actions)          |
| `maintenance` | tâches diverses (configuration, nettoyage)     |
| `annulation`  | annule un commit précédent                     |

**Zone** (facultative, en minuscules sans accents) : la partie de l'application touchée.
`interface`, `tauri`, `lanceur`, `processus`, `minuteur`, `taches`, `musique`, `github`,
`gmail`, `claude`, `reglages`, `deps`.

**Description** : une phrase courte en français, qui commence par un verbe à l'infinitif,
sans majuscule au début ni point à la fin. Les accents sont permis.

Exemples :

```
ajout(lanceur): ajouter les raccourcis d'applications
correction(processus): corriger le tri par mémoire utilisée
doc: expliquer l'installation sous windows
build(deps): mettre à jour tauri en 2.1
```

**Changement cassant** : ajouter `!` après le type ou la zone, et l'expliquer dans le corps :

```
ajout(reglages)!: changer le format du fichier de configuration

CHANGEMENT CASSANT : l'ancien fichier config.json doit être migré.
```

Conseils :

- un commit = un seul changement logique ;
- le corps du message (facultatif, après une ligne vide) explique le *pourquoi*.

## 5. Pull Requests

1. Envoyer sa branche : `git push -u origin ajout/12-minuteur-pomodoro`
2. Ouvrir une Pull Request vers `main` :
   - **titre** au même format qu'un commit, en français : il devient le message final sur `main` ;
   - **description** : remplir le modèle, et garder la ligne `Closes #12` avec le numéro de l'issue.
     `Closes` est le mot-clé de GitHub (il n'existe pas en français) : c'est lui qui relie
     la PR à l'issue et la ferme à la fusion.
3. La CI « Conventions » vérifie le nom de branche, le titre, l'issue liée (elle doit exister
   et être ouverte) et chaque message de commit.
4. Fusion en **Squash and merge** uniquement, puis suppression de la branche.

Interdits :

- envoyer directement sur `main` ;
- `git push --force` sur `main` (sur sa propre branche, utiliser `--force-with-lease`) ;
- contourner les vérifications avec `--no-verify`.

## 6. Versions

Les versions suivent [SemVer](https://semver.org/lang/fr/) : `correction` → version corrective,
`ajout` → version mineure, `!` → version majeure. Les étiquettes sont au format `v0.1.0`.
