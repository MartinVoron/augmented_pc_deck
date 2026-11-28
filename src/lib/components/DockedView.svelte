<!--
  Fenêtre ancrée sur un bord de l'écran, dans le style de l'encoche d'un MacBook
  (boring.notch) ou de la Dynamic Island d'un iPhone.
  - Repliée : une petite encoche noire collée au bord, raccordée à l'écran par deux
    coins creusés (« oreilles »), avec le logo ou entièrement noire.
  - La fenêtre (transparente) garde la taille du panneau ouvert : seule l'encoche
    dessinée change de taille. Sous Linux, elle a la taille de l'encoche quand
    celle-ci est repliée, et l'interface dit au Rust où est la souris.
  - Souris dessus : l'encoche s'étire jusqu'à la taille du panneau avec un ressort
    qui rebondit un peu, et les tuiles apparaissent.
  - Souris sortie : l'encoche se replie, avec un ressort sans rebond.
  - Le bouton Détacher remet la fenêtre normale.
  Le suivi de la souris se fait dans le Rust (src-tauri/src/dock.rs).
-->
<script lang="ts">
  import { PictureInPicture2 } from "@lucide/svelte";
  import { untrack } from "svelte";
  import { prefersReducedMotion, Spring } from "svelte/motion";
  import { innerHeight, innerWidth } from "svelte/reactivity/window";
  import {
    NOTCH_ACTIVE_LENGTH,
    NOTCH_LENGTH,
    setDockNotch,
    setDockPanel,
    setDockPointer,
  } from "../dock";
  import { live, pickActivity, watchActivity } from "../dock/activity.svelte";
  import { CLOSE_SPRING, foldedSize, OPEN_SPRING, openSize, windowForPanel } from "../dock/island";
  import { panelLayout } from "../dock/panel";
  import { dock } from "../dock/state.svelte";
  import { pageToShow, sanitizePages } from "../grid/layout";
  import { logError } from "../logging";
  import { settings, updateSettings } from "../modules/settings/state.svelte";
  import Dashboard from "./Dashboard.svelte";
  import NotchActivity from "./NotchActivity.svelte";
  import PageTabs from "./PageTabs.svelte";

  let { pageId, onselect }: { pageId: string; onselect: (id: string) => void } = $props();

  let pages = $derived(sanitizePages(settings.config?.layout.pages));
  let page = $derived(pageToShow(pages, pageId));
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
    const size = windowForPanel(dock.edge, layout);
    setDockPanel(size.width, size.height).catch((e: unknown) => logError("ancrage", e));
  });

  // Activité en direct (morceau, minuteur) : l'encoche repliée s'allonge pour la montrer.
  $effect(() => watchActivity());
  let activity = $derived(pickActivity(live.track, live.runs));
  let vertical = $derived(dock.edge === "left" || dock.edge === "right");
  let notchLength = $derived(activity && !vertical ? NOTCH_ACTIVE_LENGTH : NOTCH_LENGTH);
  // Le Rust ouvre le panneau quand la souris s'arrête sur l'encoche : il doit
  // connaître sa longueur.
  $effect(() => {
    setDockNotch(notchLength).catch((e: unknown) => logError("ancrage", e));
  });

  // Souris dans la fenêtre ou non : sous Linux, c'est ce qui ouvre le panneau.
  $effect(() => {
    const root = document.documentElement;
    let inside = false;
    const send = (value: boolean) => {
      if (value === inside) return;
      inside = value;
      setDockPointer(value).catch((e: unknown) => logError("ancrage", e));
    };
    const enter = () => send(true);
    const leave = () => send(false);
    root.addEventListener("mouseenter", enter);
    root.addEventListener("mousemove", enter);
    root.addEventListener("mouseleave", leave);
    return () => {
      root.removeEventListener("mouseenter", enter);
      root.removeEventListener("mousemove", enter);
      root.removeEventListener("mouseleave", leave);
      send(false);
    };
  });

  let folded = $derived(foldedSize(dock.edge, notchLength));
  let target = $derived(
    dock.expanded
      ? openSize(dock.edge, {
          width: innerWidth.current ?? layout.width,
          height: innerHeight.current ?? layout.height,
        })
      : folded,
  );

  /** Taille de l'encoche, animée par un ressort. */
  const size = new Spring(foldedSize("top"), OPEN_SPRING);

  $effect(() => {
    const spring = dock.expanded ? OPEN_SPRING : CLOSE_SPRING;
    size.stiffness = spring.stiffness;
    size.damping = spring.damping;
    void size.set(target, { instant: prefersReducedMotion.current });
  });

  // Changer de bord ne s'anime pas : l'encoche apparaît directement à sa place.
  $effect(() => {
    const edge = dock.edge;
    void size.set(
      foldedSize(
        edge,
        untrack(() => notchLength),
      ),
      { instant: true },
    );
  });

  function popOut() {
    void updateSettings((config) => (config.window.dock = "none"));
  }
</script>

<div
  class="dock"
  data-edge={dock.edge}
  data-expanded={dock.expanded || undefined}
  style:--panel-width="{layout.width}px"
  style:--panel-height="{layout.height}px"
>
  <div
    class="island"
    style:width="{size.current.width}px"
    style:height="{size.current.height}px"
    data-folded-width={folded.width}
    data-folded-height={folded.height}
  >
    <span class="ear start" aria-hidden="true"></span>
    <span class="ear end" aria-hidden="true"></span>
    <div class="body">
      <div class="notch" aria-hidden={dock.expanded}>
        {#if activity}
          <NotchActivity {activity} {vertical} />
        {:else if logo}
          <img src="/favicon.png" alt="" width="16" height="16" />
        {/if}
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
</div>

<style>
  .dock {
    --ear: 10px; /* NOTCH_EAR */
    --radius-folded: 12px;
    --radius-open: 28px;
    --radius: var(--radius-folded);

    position: fixed;
    inset: 0;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .dock[data-expanded] {
    --radius: var(--radius-open);
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

  /* Taille animée par le ressort (style width/height). */
  .island {
    position: relative;
    flex-shrink: 0;
  }

  .body {
    position: absolute;
    inset: 0;
    overflow: hidden;
    background: #000;
    transition:
      border-radius 300ms ease,
      box-shadow 300ms ease;
  }

  [data-expanded] .body {
    box-shadow: 0 10px 30px rgb(0 0 0 / 45%);
  }

  /* Seuls les coins tournés vers l'écran sont arrondis, comme une encoche. */
  [data-edge="top"] .body {
    border-radius: 0 0 var(--radius) var(--radius);
  }

  [data-edge="bottom"] .body {
    border-radius: var(--radius) var(--radius) 0 0;
  }

  [data-edge="left"] .body {
    border-radius: 0 var(--radius) var(--radius) 0;
  }

  [data-edge="right"] .body {
    border-radius: var(--radius) 0 0 var(--radius);
  }

  /* Oreilles : coins creusés qui raccordent l'encoche au bord de l'écran. */
  .ear {
    position: absolute;
    width: var(--ear);
    height: var(--ear);
    --cut: transparent calc(var(--ear) - 0.5px), #000 var(--ear);
  }

  [data-edge="top"] .ear,
  [data-edge="bottom"] .ear {
    right: 100%;
  }

  [data-edge="top"] .ear.end,
  [data-edge="bottom"] .ear.end {
    right: auto;
    left: 100%;
  }

  [data-edge="top"] .ear {
    top: 0;
    background: radial-gradient(circle at 0 100%, var(--cut));
  }

  [data-edge="top"] .ear.end {
    background: radial-gradient(circle at 100% 100%, var(--cut));
  }

  [data-edge="bottom"] .ear {
    bottom: 0;
    background: radial-gradient(circle at 0 0, var(--cut));
  }

  [data-edge="bottom"] .ear.end {
    background: radial-gradient(circle at 100% 0, var(--cut));
  }

  [data-edge="left"] .ear,
  [data-edge="right"] .ear {
    bottom: 100%;
  }

  [data-edge="left"] .ear.end,
  [data-edge="right"] .ear.end {
    bottom: auto;
    top: 100%;
  }

  [data-edge="left"] .ear {
    left: 0;
    background: radial-gradient(circle at 100% 0, var(--cut));
  }

  [data-edge="left"] .ear.end {
    background: radial-gradient(circle at 100% 100%, var(--cut));
  }

  [data-edge="right"] .ear {
    right: 0;
    background: radial-gradient(circle at 0 0, var(--cut));
  }

  [data-edge="right"] .ear.end {
    background: radial-gradient(circle at 0 100%, var(--cut));
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
    /* Taille du panneau ouvert, sans dépasser la fenêtre (BOUNCE_ROOM : 16 px). */
    width: min(var(--panel-width), calc(100vw - 2 * var(--ear) - 16px));
    height: min(var(--panel-height), calc(100vh - 16px));
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
    width: min(var(--panel-width), calc(100vw - 16px));
    height: min(var(--panel-height), calc(100vh - 2 * var(--ear) - 16px));
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
    .body,
    .notch,
    .panel {
      transition: none;
    }
  }
</style>
