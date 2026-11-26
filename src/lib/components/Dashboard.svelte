<!--
  Tableau de bord : la grille de tuiles enregistrée dans la configuration.
  En mode édition, on peut ajouter, supprimer, déplacer (glisser-déposer ou flèches)
  et redimensionner les tuiles. Chaque changement est enregistré tout de suite.
-->
<script lang="ts">
  import { ArrowLeft, ArrowRight, Pencil, Plus, Trash2 } from "@lucide/svelte";
  import { KIND_ACCENTS, type TileConfig, type TileKind } from "../grid/defaultLayout";
  import { KIND_ICONS } from "../grid/icons";
  import {
    addTile,
    KIND_TITLES,
    moveTile,
    removeTile,
    resizeTile,
    sanitizeLayout,
    TILE_SIZES,
  } from "../grid/layout";
  import type { TileSize } from "../grid/types";
  import { settings, updateSettings } from "../modules/settings/state.svelte";
  import ShortcutsDialog from "../modules/launcher/ShortcutsDialog.svelte";
  import Tile from "./Tile.svelte";
  import TileContent from "./TileContent.svelte";
  import TileGrid from "./TileGrid.svelte";

  let { editing = false }: { editing?: boolean } = $props();

  let tiles = $derived(sanitizeLayout(settings.config?.layout.tiles));
  let draggedId = $state<string | null>(null);
  let newKind = $state<TileKind>("timers");
  /** Tuile du lanceur dont on modifie les raccourcis. */
  let shortcutsTileId = $state<string | null>(null);

  const SIZE_LABELS: Record<TileSize, string> = {
    "1x1": "Petite (1×1)",
    "2x1": "Large (2×1)",
    "2x2": "Grande (2×2)",
  };

  function save(next: TileConfig[]) {
    void updateSettings((config) => (config.layout.tiles = next));
  }

  function drop(targetIndex: number) {
    if (draggedId) save(moveTile(tiles, draggedId, targetIndex));
    draggedId = null;
  }
</script>

<TileGrid>
  {#each tiles as tile, index (tile.id)}
    <Tile
      title={tile.title}
      size={tile.size}
      accent={KIND_ACCENTS[tile.kind]}
      icon={KIND_ICONS[tile.kind]}
      draggable={editing}
      data-editing={editing || undefined}
      data-dragging={draggedId === tile.id || undefined}
      ondragstart={() => (draggedId = tile.id)}
      ondragend={() => (draggedId = null)}
      ondragover={(event) => editing && event.preventDefault()}
      ondrop={(event) => {
        event.preventDefault();
        drop(index);
      }}
    >
      {#if editing}
        <div class="toolbar">
          <button
            type="button"
            class="icon-button"
            aria-label="Déplacer {tile.title} avant"
            disabled={index === 0}
            onclick={() => save(moveTile(tiles, tile.id, index - 1))}
          >
            <ArrowLeft size={14} />
          </button>
          <button
            type="button"
            class="icon-button"
            aria-label="Déplacer {tile.title} après"
            disabled={index === tiles.length - 1}
            onclick={() => save(moveTile(tiles, tile.id, index + 1))}
          >
            <ArrowRight size={14} />
          </button>
          <button
            type="button"
            class="icon-button danger"
            aria-label="Supprimer {tile.title}"
            onclick={() => save(removeTile(tiles, tile.id))}
          >
            <Trash2 size={14} />
          </button>
        </div>
        <label class="size">
          <span>Taille</span>
          <select
            aria-label="Taille de {tile.title}"
            value={tile.size}
            onchange={(event) =>
              save(resizeTile(tiles, tile.id, event.currentTarget.value as TileSize))}
          >
            {#each TILE_SIZES as size (size)}
              <option value={size}>{SIZE_LABELS[size]}</option>
            {/each}
          </select>
        </label>
        {#if tile.kind === "launcher"}
          <button type="button" class="shortcuts" onclick={() => (shortcutsTileId = tile.id)}>
            <Pencil size={14} /> Raccourcis…
          </button>
        {/if}
        <p class="hint">Glisser pour déplacer.</p>
      {:else}
        <TileContent {tile} />
      {/if}
    </Tile>
  {/each}

  {#if editing}
    <div class="add">
      <label>
        <span>Ajouter une tuile</span>
        <select aria-label="Module de la nouvelle tuile" bind:value={newKind}>
          {#each Object.entries(KIND_TITLES) as [kind, title] (kind)}
            <option value={kind}>{title}</option>
          {/each}
        </select>
      </label>
      <button type="button" onclick={() => save(addTile(tiles, newKind))}>
        <Plus size={14} /> Ajouter
      </button>
    </div>
  {/if}
</TileGrid>

{#if shortcutsTileId}
  <ShortcutsDialog tileId={shortcutsTileId} onclose={() => (shortcutsTileId = null)} />
{/if}

{#if editing && tiles.length === 0}
  <p class="empty">La grille est vide : ajoutez une tuile.</p>
{/if}

<style>
  .hint {
    margin: 0;
  }

  .toolbar {
    display: flex;
    gap: 0.25rem;
    margin-bottom: 0.5rem;
  }

  .icon-button {
    display: inline-flex;
    padding: 0.2rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .icon-button:hover:not(:disabled) {
    color: var(--color-text);
    background: var(--color-bg);
  }

  .icon-button:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .icon-button.danger:hover {
    color: var(--color-danger);
  }

  .size {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    margin-bottom: 0.5rem;
  }

  select,
  .add button,
  .shortcuts {
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: var(--color-bg);
    color: var(--color-text);
    font: inherit;
  }

  .shortcuts {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    margin-bottom: 0.5rem;
    cursor: pointer;
  }

  .add {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.75rem;
    border: 2px dashed var(--color-border);
    border-radius: var(--tile-radius);
    color: var(--color-text-muted);
    font-size: 0.875rem;
  }

  .add label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .add button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    cursor: pointer;
  }

  .empty {
    padding: 0 var(--tile-gap);
    color: var(--color-text-muted);
  }

  :global(.tile[data-editing]) {
    cursor: grab;
    border-style: dashed;
  }

  :global(.tile[data-dragging]) {
    opacity: 0.4;
  }
</style>
