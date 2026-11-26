#!/bin/sh
# Vérifie qu'une description de PR référence une issue ouverte du dépôt.
# Usage : scripts/check-issue-link.sh <fichier-description>
# Variables : GH_TOKEN et GITHUB_REPOSITORY (fournies par GitHub Actions).
body_file="$1"
nums=$(grep -Eio '(close[sd]?|fix(e[sd])?|resolve[sd]?) +#[0-9]+' "$body_file" | grep -Eo '[0-9]+' | sort -u)

if [ -z "$nums" ]; then
  cat >&2 <<MSG
✖ La PR n'est reliée à aucune issue.
Ajoutez dans la description une ligne comme :
    Closes #12
(mot-clé GitHub, en anglais, qui ferme l'issue à la fusion)
MSG
  exit 1
fi

status=0
for n in $nums; do
  if ! kind=$(gh api "repos/$GITHUB_REPOSITORY/issues/$n" --jq 'if .pull_request then "pr" else .state end' 2>/dev/null); then
    echo "✖ L'issue #$n n'existe pas." >&2; status=1; continue
  fi
  case "$kind" in
    open) echo "✔ Reliée à l'issue #$n" ;;
    pr) echo "✖ #$n est une PR, pas une issue." >&2; status=1 ;;
    *) echo "✖ L'issue #$n est fermée." >&2; status=1 ;;
  esac
done
exit $status
