<!--
  Barre de titre de l'application, à la place de celle de Windows.
  - On la saisit pour déplacer la fenêtre, et un double-clic l'agrandit
    (attribut data-tauri-drag-region).
  - Trois ronds façon macOS : rouge pour fermer (la fenêtre est masquée dans la
    zone de notification), jaune pour réduire, vert pour agrandir.
-->
<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { logError } from "../logging";

  let { title = "Augmented PC Deck" }: { title?: string } = $props();

  function run(action: "close" | "minimize" | "toggleMaximize") {
    try {
      getCurrentWindow()
        [action]()
        .catch((e: unknown) => logError("barre de titre", e));
    } catch (e) {
      // Hors de Tauri (navigateur, tests) : la fenêtre n'existe pas.
      void logError("barre de titre", e);
    }
  }
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="dots">
    <button
      type="button"
      class="dot close"
      aria-label="Fermer (masquer dans la zone de notification)"
      title="Fermer"
      onclick={() => run("close")}
    >
      <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M3 3l4 4M7 3l-4 4" /></svg>
    </button>
    <button
      type="button"
      class="dot minimize"
      aria-label="Réduire"
      title="Réduire"
      onclick={() => run("minimize")}
    >
      <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M2.5 5h5" /></svg>
    </button>
    <button
      type="button"
      class="dot maximize"
      aria-label="Agrandir"
      title="Agrandir"
      onclick={() => run("toggleMaximize")}
    >
      <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M5 2.5v5M2.5 5h5" /></svg>
    </button>
  </div>
  <span class="title" data-tauri-drag-region>{title}</span>
</header>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-shrink: 0;
    height: 2.25rem;
    padding: 0 0.75rem;
    background: var(--color-bg);
    border-bottom: 1px solid var(--color-border);
    user-select: none;
    -webkit-user-select: none;
  }

  .dots {
    display: flex;
    gap: 0.5rem;
  }

  .dot {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 0.8rem;
    height: 0.8rem;
    padding: 0;
    border: none;
    border-radius: 50%;
    cursor: pointer;
  }

  .close {
    background: #ff5f57;
  }

  .minimize {
    background: #febc2e;
  }

  .maximize {
    background: #28c840;
  }

  /* Comme sur macOS : les symboles apparaissent au survol des ronds. */
  svg {
    width: 0.5rem;
    height: 0.5rem;
    fill: none;
    stroke: rgb(0 0 0 / 60%);
    stroke-width: 1.5;
    stroke-linecap: round;
    opacity: 0;
  }

  .dots:hover svg,
  .dot:focus-visible svg {
    opacity: 1;
  }

  .dot:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }

  .title {
    flex: 1;
    color: var(--color-text-muted);
    font-size: 0.8rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
