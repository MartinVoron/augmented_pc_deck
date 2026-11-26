<script lang="ts">
  import Tile from "./lib/components/Tile.svelte";
  import TileGrid from "./lib/components/TileGrid.svelte";
  import { DEFAULT_LAYOUT, KIND_ACCENTS } from "./lib/grid/defaultLayout";
  import { KIND_ICONS } from "./lib/grid/icons";
  import { getAppVersion } from "./lib/modules/app";

  let version = $state("");
  let error = $state("");

  // Vérifie que l'interface et le Rust communiquent bien.
  getAppVersion()
    .then((v) => (version = v))
    .catch((e: Error) => (error = e.message));
</script>

<main>
  <TileGrid>
    {#each DEFAULT_LAYOUT as tile (tile.id)}
      <Tile
        title={tile.title}
        size={tile.size}
        accent={KIND_ACCENTS[tile.kind]}
        icon={KIND_ICONS[tile.kind]}
      >
        <p class="placeholder">Bientôt disponible</p>
      </Tile>
    {/each}
  </TileGrid>

  <footer>
    {#if version}
      Augmented PC Deck {version}
    {:else if error}
      Le moteur Tauri n'est pas joignable : {error}
    {/if}
  </footer>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  footer {
    margin-top: auto;
    padding: 0.5rem var(--tile-gap);
    color: var(--color-text-muted);
    font-size: 0.75rem;
  }

  .placeholder {
    margin: 0;
  }
</style>
