#!/bin/sh
# Vérifie le nom d'une branche. Usage : scripts/check-branch-name.sh [nom]
branch="${1:-$(git rev-parse --abbrev-ref HEAD)}"
pattern='^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)/[a-z0-9][a-z0-9._-]*$|^claude/[a-z0-9][a-z0-9._-]*$|^release/v?[0-9]+\.[0-9]+\.[0-9]+$|^dependabot/.+$'
if printf '%s\n' "$branch" | grep -Eq "$pattern"; then
  exit 0
fi
cat >&2 <<MSG
✖ Nom de branche invalide : $branch
Format attendu : <type>/<description-courte-en-kebab-case>
Exemples : feat/minuteur-pomodoro, fix/crash-demarrage, docs/readme
Voir CONTRIBUTING.md
MSG
exit 1
