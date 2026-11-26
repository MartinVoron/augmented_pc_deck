# Augmented PC Deck

Tableau de bord de bureau, en local, pour Windows, façon Stream Deck : raccourcis
d'applications, gestionnaire de tâches, minuteurs, liste de tâches, musique en cours,
GitHub, Gmail et notifications de Claude Code.

Construit avec [Tauri 2](https://v2.tauri.app/) (Rust) et [Svelte 5](https://svelte.dev/)
(TypeScript, Vite).

## Prérequis (Windows 10 ou 11)

1. **Microsoft C++ Build Tools** : installer
   [Visual Studio Build Tools](https://visualstudio.microsoft.com/fr/visual-cpp-build-tools/)
   et cocher « Développement Desktop en C++ ».
2. **WebView2** : déjà présent sous Windows 10 (version 1803 et plus) et Windows 11.
   Sinon, installer le [runtime WebView2](https://developer.microsoft.com/fr-fr/microsoft-edge/webview2/).
3. **Rust** (version 1.77.2 ou plus) : installer avec [rustup](https://rustup.rs/), en
   gardant la chaîne `stable-msvc` proposée par défaut.
4. **Node.js** (version 20.19 ou plus, la version LTS conseillée) : [nodejs.org](https://nodejs.org/fr).

Le détail officiel est sur la page [Prérequis de Tauri](https://v2.tauri.app/start/prerequisites/).

## Démarrer

```sh
git clone https://github.com/MartinVoron/augmented_pc_deck.git
cd augmented_pc_deck
git config core.hooksPath .githooks
npm install
```

## Commandes

| Commande              | Ce qu'elle fait                                                  |
|-----------------------|------------------------------------------------------------------|
| `npm run tauri dev`   | Lance l'application en mode développement (rechargement à chaud) |
| `npm run tauri build` | Produit l'exécutable et les installateurs Windows                |
| `npm run check`       | Vérifie les types TypeScript et Svelte                           |
| `npm run build`       | Compile seulement l'interface (dossier `dist/`)                  |

Après `npm run tauri build`, on trouve :

- l'exécutable : `src-tauri/target/release/augmented-pc-deck.exe` ;
- les installateurs : `src-tauri/target/release/bundle/nsis/` et `bundle/msi/`.

La première compilation Rust prend quelques minutes, les suivantes sont bien plus rapides.

## Organisation du code

```
src/          interface Svelte (TypeScript)
src-tauri/    application Tauri (Rust) : commandes, configuration, icônes
public/       fichiers statiques de l'interface
```

## Contribuer

Les règles de travail (issues, branches, commits en français, PR) sont dans
[CONTRIBUTING.md](CONTRIBUTING.md).

## Licence

[MIT](LICENSE)
