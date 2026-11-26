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
| `npm test`            | Lance tous les tests : interface (Vitest) puis Rust (`cargo test`) |
| `npm run lint`        | Vérifie le style : ESLint, Prettier, `cargo fmt` et `cargo clippy` |
| `npm run format`      | Met en forme tout le code (Prettier et `cargo fmt`)              |
| `npm run build`       | Compile seulement l'interface (dossier `dist/`)                  |

Après `npm run tauri build`, on trouve :

- l'exécutable : `src-tauri/target/release/augmented-pc-deck.exe` ;
- les installateurs : `src-tauri/target/release/bundle/nsis/` et `bundle/msi/`.

La première compilation Rust prend quelques minutes, les suivantes sont bien plus rapides.

## Organisation du code

Chaque fonctionnalité a sa place, côté Rust et côté interface, avec le même nom anglais :
`launcher` (lanceur), `processes` (processus), `timers` (minuteurs), `todos` (tâches),
`music` (musique), `github`, `gmail`, `claude` et `settings` (paramètres).

```
src/
  App.svelte              écran principal
  lib/tauri.ts            fonction call() pour appeler le Rust
  lib/modules/<module>/   composants et appels Tauri d'un module
src-tauri/
  src/error.rs            erreur commune renvoyée à l'interface
  src/modules/<module>.rs commandes Tauri d'un module
  tauri.conf.json         configuration de la fenêtre et de l'installateur
public/                   fichiers statiques de l'interface
```

### Convention pour les commandes Tauri

- Côté Rust, une commande s'appelle `<module>_<action>` (par exemple `timers_start`),
  renvoie toujours un `AppResult<T>` et est déclarée dans `src-tauri/src/lib.rs`.
- En cas d'échec, l'interface reçoit `{ "code": "not_found", "message": "…" }` :
  le `code` est stable pour le code, le `message` est en français, prêt à afficher.
- Côté interface, on n'utilise jamais `invoke` directement mais `call()` de
  `src/lib/tauri.ts`, qui transforme l'échec en `AppError` (avec `code` et `message`).

## Tests

- **Interface** : fichiers `*.test.ts` à côté du code testé (exemples : `src/lib/tauri.test.ts`,
  `src/App.test.ts`). Ils tournent avec Vitest et Testing Library, sans Tauri : on remplace
  `invoke` par une fausse fonction avec `vi.mock`. `npm run test:front` les lance seuls.
- **Rust** : un bloc `#[cfg(test)] mod tests` à la fin du fichier testé (exemple :
  `src-tauri/src/error.rs`). `npm run test:rust` les lance seuls.

## Intégration continue

À chaque PR, le workflow [CI](.github/workflows/ci.yml) vérifie le projet sur une machine
Windows : `npm run lint`, `npm run check`, les tests de l'interface et du Rust, puis la
compilation de l'application. Il échoue dès qu'une étape échoue. Les dépendances npm et la
compilation Rust sont mises en cache pour aller plus vite.

## Contribuer

Les règles de travail (issues, branches, commits en français, PR) sont dans
[CONTRIBUTING.md](CONTRIBUTING.md).

## Licence

[MIT](LICENSE)
