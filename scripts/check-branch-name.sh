#!/bin/sh
# Vérifie le nom d'une branche. Usage : scripts/check-branch-name.sh [nom]
branch="${1:-$(git rev-parse --abbrev-ref HEAD)}"
types='ajout|correction|doc|style|refonte|perf|test|build|ci|maintenance|annulation'
pattern="^($types)/[0-9]+-[a-z0-9][a-z0-9-]*\$|^claude/[a-z0-9][a-z0-9-]*\$|^version/v[0-9]+\.[0-9]+\.[0-9]+\$|^dependabot/.+\$"
if printf '%s\n' "$branch" | grep -Eq "$pattern"; then
  exit 0
fi
cat >&2 <<MSG
✖ Nom de branche invalide : $branch
Format attendu : <type>/<numéro-issue>-<description-en-français>
Exemples : ajout/12-minuteur-pomodoro, correction/31-plantage-au-demarrage
(minuscules, chiffres et tirets uniquement, sans accents)
Voir CONTRIBUTING.md
MSG
exit 1
