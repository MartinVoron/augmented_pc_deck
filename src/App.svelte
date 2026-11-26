<script lang="ts">
  import { Settings } from "@lucide/svelte";
  import Tile from "./lib/components/Tile.svelte";
  import TileGrid from "./lib/components/TileGrid.svelte";
  import { DEFAULT_LAYOUT, KIND_ACCENTS } from "./lib/grid/defaultLayout";
  import { KIND_ICONS } from "./lib/grid/icons";
  import { getAppVersion } from "./lib/modules/app";
  import SettingsScreen from "./lib/modules/settings/SettingsScreen.svelte";
  import { loadSettings } from "./lib/modules/settings/state.svelte";

  let version = $state("");
  let error = $state("");
  let showSettings = $state(false);

  // Vérifie que l'interface et le Rust communiquent bien.
  getAppVersion()
    .then((v) => (version = v))
    .catch((e: Error) => (error = e.message));

  void loadSettings();
</script>

{#if showSettings}
  <SettingsScreen onclose={() => (showSettings = false)} />
{:else}
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
      <span>
        {#if version}
          Augmented PC Deck {version}
        {:else if error}
          Le moteur Tauri n'est pas joignable : {error}
        {/if}
      </span>
      <button
        type="button"
        aria-label="Ouvrir les paramètres"
        onclick={() => (showSettings = true)}
      >
        <Settings size={16} />
        Paramètres
      </button>
    </footer>
  </main>
{/if}

<style>
  main {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-top: auto;
    padding: 0.5rem var(--tile-gap);
    color: var(--color-text-muted);
    font-size: 0.75rem;
  }

  footer button {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.3rem 0.6rem;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    background: transparent;
    color: var(--color-text-muted);
    font: inherit;
    cursor: pointer;
  }

  footer button:hover {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .placeholder {
    margin: 0;
  }
</style>
