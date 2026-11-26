<script lang="ts">
  import { Check, LayoutGrid, Settings } from "@lucide/svelte";
  import Dashboard from "./lib/components/Dashboard.svelte";
  import { getAppVersion } from "./lib/modules/app";
  import SettingsScreen from "./lib/modules/settings/SettingsScreen.svelte";
  import { loadSettings } from "./lib/modules/settings/state.svelte";

  let version = $state("");
  let error = $state("");
  let showSettings = $state(false);
  let editing = $state(false);

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
    <Dashboard {editing} />

    <footer>
      <span>
        {#if version}
          Augmented PC Deck {version}
        {:else if error}
          Le moteur Tauri n'est pas joignable : {error}
        {/if}
      </span>
      <span class="buttons">
        <button type="button" aria-pressed={editing} onclick={() => (editing = !editing)}>
          {#if editing}
            <Check size={16} /> Terminé
          {:else}
            <LayoutGrid size={16} /> Modifier la disposition
          {/if}
        </button>
        <button
          type="button"
          aria-label="Ouvrir les paramètres"
          onclick={() => (showSettings = true)}
        >
          <Settings size={16} />
          Paramètres
        </button>
      </span>
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

  footer button[aria-pressed="true"] {
    border-color: var(--color-accent);
    color: var(--color-text);
  }

  footer button:hover {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .buttons {
    display: flex;
    gap: 0.5rem;
  }
</style>
