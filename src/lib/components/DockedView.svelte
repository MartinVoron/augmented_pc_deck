<!--
  Fenêtre ancrée sur un bord de l'écran, façon encoche de MacBook ou Dynamic Island.
  - Repliée : une petite encoche noire collée au bord, avec le logo ou entièrement noire.
  - Souris dessus : le Rust agrandit la fenêtre (transparente) à la taille du panneau,
    puis l'encoche s'étire jusqu'à lui avec une animation et les tuiles apparaissent.
  - Souris sortie : l'encoche se replie (même durée que CLOSE_ANIMATION_MS du Rust),
    puis la fenêtre redevient petite.
  - Le bouton Détacher remet la fenêtre normale.
  Le suivi de la souris se fait dans le Rust (src-tauri/src/dock.rs).
-->
<script lang="ts">
  import { PictureInPicture2 } from "@lucide/svelte";
  import { NOTCH_DEPTH, NOTCH_LENGTH, setDockPanel } from "../dock";
  import { panelLayout } from "../dock/panel";
  import { dock } from "../dock/state.svelte";
  import { pageToShow, sanitizePages } from "../grid/layout";
  import { logError } from "../logging";
  import { settings, updateSettings } from "../modules/settings/state.svelte";
  import Dashboard from "./Dashboard.svelte";
  import PageTabs from "./PageTabs.svelte";

  let { pageId, onselect }: { pageId: string; onselect: (id: string) => void } = $props();

  let pages = $derived(sanitizePages(settings.config?.layout.pages));
  let page = $derived(pageToShow(pages, pageId));
  let vertical = $derived(dock.edge === "left" || dock.edge === "right");
  let logo = $derived((settings.config?.window.notchStyle ?? "logo") === "logo");
  let layout = $derived(
    // Largeur de l'écran inconnue (0 hors de Tauri) : pas de limite.
    panelLayout(
      page.tiles.map((tile) => tile.size),
      dock.edge,
      globalThis.screen?.availWidth || undefined,
    ),
  );

  // Le Rust donne cette taille à la fenêtre quand le panneau s'ouvre.
  $effect(() => {
    setDockPanel(layout.width, layout.height).catch((e: unknown) => logError("ancrage", e));
  });

  function popOut() {
    void updateSettings((config) => (config.window.dock = "none"));
  }
</script>

<div
  class="dock"
  data-edge={dock.edge}
  data-expanded={dock.expanded || undefined}
  style:--notch-width="{vertical ? NOTCH_DEPTH : NOTCH_LENGTH}px"
  style:--notch-height="{vertical ? NOTCH_LENGTH : NOTCH_DEPTH}px"
  style:--panel-width="{layout.width}px"
  style:--panel-height="{layout.height}px"
>
  <div class="island">
    <div class="notch" aria-hidden="true">
      {#if logo}<img src="/favicon.png" alt="" width="18" height="18" />{/if}
    </div>
    <div class="panel" inert={!dock.expanded} aria-hidden={!dock.expanded}>
      <div class="bar">
        {#if pages.length > 1}
          <PageTabs currentId={page.id} {onselect} />
        {:else}
          <span class="name">{page.name}</span>
        {/if}
        <button
          type="button"
          class="pop-out"
          aria-label="Détacher la fenêtre du bord de l'écran"
          title="Détacher"
          onclick={popOut}
        >
          <PictureInPicture2 size={16} />
        </button>
      </div>
      <div class="tiles">
        <Dashboard pageId={page.id} columns={layout.columns} />
      </div>
    </div>
  </div>
</div>

<style>
  .dock {
    --radius-folded: 16px;
    --radius-open: 24px;
    --ease: cubic-bezier(0.32, 0.72, 0, 1);
    --duration: 260ms;

    position: fixed;
    inset: 0;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .dock[data-edge="top"] {
    align-items: flex-start;
  }

  .dock[data-edge="bottom"] {
    align-items: flex-end;
  }

  .dock[data-edge="left"] {
    justify-content: flex-start;
  }

  .dock[data-edge="right"] {
    justify-content: flex-end;
  }

  .island {
    position: relative;
    flex-shrink: 0;
    width: var(--notch-width);
    height: var(--notch-height);
    overflow: hidden;
    background: #000;
    box-shadow: 0 0 0 1px rgb(255 255 255 / 6%);
    transition:
      width var(--duration) var(--ease),
      height var(--duration) var(--ease),
      border-radius var(--duration) var(--ease);
  }

  [data-expanded] .island {
    width: 100%;
    height: 100%;
    --radius-folded: var(--radius-open);
  }

  /* Seuls les coins tournés vers l'écran sont arrondis, comme une encoche. */
  [data-edge="top"] .island {
    border-radius: 0 0 var(--radius-folded) var(--radius-folded);
  }

  [data-edge="bottom"] .island {
    border-radius: var(--radius-folded) var(--radius-folded) 0 0;
  }

  [data-edge="left"] .island {
    border-radius: 0 var(--radius-folded) var(--radius-folded) 0;
  }

  [data-edge="right"] .island {
    border-radius: var(--radius-folded) 0 0 var(--radius-folded);
  }

  .notch {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: opacity 120ms ease 100ms;
  }

  .notch img {
    opacity: 0.85;
  }

  [data-expanded] .notch {
    opacity: 0;
    transition-delay: 0s;
  }

  /* Le panneau garde sa taille pendant l'animation : l'encoche le découvre. */
  .panel {
    position: absolute;
    display: flex;
    flex-direction: column;
    width: min(var(--panel-width), 100vw);
    height: min(var(--panel-height), 100vh);
    opacity: 0;
    transition: opacity 100ms ease;
  }

  [data-expanded] .panel {
    opacity: 1;
    transition: opacity 180ms ease 90ms;
  }

  [data-edge="top"] .panel,
  [data-edge="bottom"] .panel {
    left: 50%;
    transform: translateX(-50%);
  }

  [data-edge="top"] .panel {
    top: 0;
  }

  [data-edge="bottom"] .panel {
    bottom: 0;
  }

  [data-edge="left"] .panel,
  [data-edge="right"] .panel {
    top: 50%;
    transform: translateY(-50%);
  }

  [data-edge="left"] .panel {
    left: 0;
  }

  [data-edge="right"] .panel {
    right: 0;
  }

  .bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    flex-shrink: 0;
    min-width: 0;
    height: 44px;
    padding: 0 var(--tile-gap);
  }

  .bar > :global(nav) {
    flex: 1;
    min-width: 0;
  }

  .name {
    color: var(--color-text-muted);
    font-size: 0.8rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .pop-out {
    display: inline-flex;
    flex-shrink: 0;
    padding: 0.35rem;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .pop-out:hover,
  .pop-out:focus-visible {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .tiles {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  @media (prefers-reduced-motion: reduce) {
    .island,
    .notch,
    .panel {
      transition: none;
    }
  }
</style>
