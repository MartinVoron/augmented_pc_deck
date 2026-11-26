#!/bin/sh
# Active les hooks Git du dépôt (à lancer une fois après le clone).
git config core.hooksPath .githooks
echo "✔ Hooks Git activés (.githooks)"
