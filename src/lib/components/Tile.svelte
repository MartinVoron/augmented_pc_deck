<!--
  Tuile générique de la grille. Chaque module place son contenu à l'intérieur.
  Le contenu est protégé par TileBoundary : s'il plante, la tuile affiche un message clair.
-->
<script lang="ts">
  import type { Component, Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import { TILE_SPANS, type TileAccent, type TileSize } from "../grid/types";
  import TileBoundary from "./TileBoundary.svelte";

  interface Props extends Omit<HTMLAttributes<HTMLElement>, "title" | "children"> {
    title: string;
    size?: TileSize;
    accent?: TileAccent;
    /** Icône affichée à côté du titre (un composant de @lucide/svelte). */
    icon?: Component<{ size?: number | string }>;
    children?: Snippet;
  }

  let { title, size = "1x1", accent = "accent", icon: Icon, children, ...rest }: Props = $props();

  let span = $derived(TILE_SPANS[size]);
</script>

<section
  {...rest}
  class="tile"
  data-size={size}
  style:grid-column="span {span.columns}"
  style:grid-row="span {span.rows}"
  style:--tile-color="var(--color-{accent})"
  aria-label={title}
>
  <header>
    {#if Icon}
      <span class="icon" aria-hidden="true"><Icon size={18} /></span>
    {/if}
    <h2>{title}</h2>
  </header>
  <div class="content">
    <TileBoundary name={title}>
      {@render children?.()}
    </TileBoundary>
  </div>
</section>

<style>
  .tile {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    padding: 0.75rem;
    border: 1px solid var(--tile-border);
    border-top: var(--tile-accent-width) solid var(--tile-accent-color, var(--tile-color));
    border-radius: var(--tile-radius);
    background: var(--tile-bg);
    box-shadow: var(--tile-shadow);
    backdrop-filter: var(--tile-backdrop);
    overflow: hidden;
    transition: background-color 0.15s;
  }

  .tile:hover {
    background: var(--tile-bg-hover);
  }

  header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .icon {
    display: inline-flex;
    color: var(--tile-color);
  }

  h2 {
    margin: 0;
    color: var(--tile-title-color);
    font-size: var(--tile-title-size);
    font-weight: 600;
    text-transform: var(--tile-title-transform);
    letter-spacing: var(--tile-title-spacing);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .content {
    flex: 1;
    min-height: 0;
    overflow: auto;
    color: var(--color-text-muted);
    font-size: 0.875rem;
  }
</style>
