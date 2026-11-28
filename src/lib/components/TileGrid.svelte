<!--
  Grille de tuiles. Le nombre de colonnes s'adapte à la largeur de la fenêtre,
  sauf si `columns` le fixe (panneau de la fenêtre ancrée) ; les tuiles se placent
  dans le premier trou libre (grid-auto-flow: dense).
-->
<script lang="ts">
  import type { Snippet } from "svelte";

  let { children, columns }: { children: Snippet; columns?: number } = $props();
</script>

<div
  class="grid"
  style:grid-template-columns={columns ? `repeat(${columns}, var(--tile-unit))` : undefined}
>
  {@render children()}
</div>

<style>
  .grid {
    display: grid;
    /* Au moins 2 colonnes pour que les tuiles 2×1 et 2×2 tiennent toujours. */
    grid-template-columns: repeat(
      auto-fill,
      minmax(min(var(--tile-unit), calc((100% - var(--tile-gap)) / 2)), 1fr)
    );
    grid-auto-rows: var(--tile-unit);
    grid-auto-flow: dense;
    gap: var(--tile-gap);
    padding: var(--tile-gap);
  }

  .grid > :global(*) {
    min-width: 0;
  }
</style>
