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

Les versions suivent [SemVer](https://semver.org/lang/fr/) : `MAJEUR.MINEUR.CORRECTIF`,
avec des étiquettes au format `v0.1.0`. **Personne ne change le numéro à la main** : il est
calculé à partir des messages de commit (en pratique, des titres de PR fusionnées).

| Commit                                   | Effet sur la version                     |
|------------------------------------------|------------------------------------------|
| `ajout`                                  | mineure : 0.2.0 → 0.3.0                  |
| `correction`, `perf`, `annulation`       | corrective : 0.2.0 → 0.2.1               |
| `!` ou `CHANGEMENT CASSANT :`            | majeure : 1.2.0 → 2.0.0 (mineure en 0.x) |
| `doc`, `style`, `refonte`, `test`, `build`, `ci`, `maintenance` | aucune            |

Tant que l'application est en 0.x, un changement cassant fait seulement monter la version
mineure. Le passage en 1.0.0 se décide à la main (voir plus bas).

**Le cycle**

1. À chaque fusion sur `main`, le workflow « Version » ouvre ou met à jour une PR
   `version/vX.Y.Z` (titre `maintenance: publier la version X.Y.Z`). Elle change le numéro
   dans `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` et les fichiers
   de verrouillage, et ajoute les notes de version dans `CHANGELOG.md`.
2. On fusionne cette PR quand on veut sortir la version (Squash and merge).
3. Le workflow crée alors l'étiquette `vX.Y.Z`, compile l'application sous Windows et
   publie les installateurs (`.exe` NSIS et `.msi`) dans une Release GitHub.

La première version publiée est celle déjà écrite dans les fichiers (0.1.0).

**Forcer un numéro** (par exemple pour la 1.0.0) : onglet *Actions* → « Version » →
*Run workflow*, puis saisir le numéro. La PR de version est ouverte avec ce numéro.

**En local**

```sh
node scripts/version/version.mjs etat       # prochaine version et notes, sans rien changer
node scripts/version/version.mjs verifier   # même numéro dans tous les fichiers ?
node --test scripts/version/version.test.mjs
```

**Réglages GitHub nécessaires (une fois)**

- *Settings → Actions → General → Workflow permissions* : cocher
  « Allow GitHub Actions to create and approve pull requests ».
- Facultatif : un jeton personnel (droits *Contents* et *Pull requests* en écriture) dans le
  secret `VERSION_TOKEN`. Sans lui, les vérifications « Conventions » ne se lancent pas sur
  la PR de version (limite de GitHub pour le jeton par défaut) ; avec lui, elles tournent
  normalement.
