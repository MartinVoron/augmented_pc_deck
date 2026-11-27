<script lang="ts">
  import { Check, LayoutGrid, Settings } from "@lucide/svelte";
  import Dashboard from "./lib/components/Dashboard.svelte";
  import PageTabs from "./lib/components/PageTabs.svelte";
  import { pageToShow, sanitizePages } from "./lib/grid/layout";
  import TitleBar from "./lib/components/TitleBar.svelte";
  import { getAppVersion } from "./lib/modules/app";
  import SettingsScreen from "./lib/modules/settings/SettingsScreen.svelte";
  import { loadSettings, settings } from "./lib/modules/settings/state.svelte";

  let version = $state("");
  let error = $state("");
  let showSettings = $state(false);
  let editing = $state(false);
  /** Page affichée ; l'application s'ouvre sur l'écran d'accueil. */
  let pageId = $state("");

  let pages = $derived(sanitizePages(settings.config?.layout.pages));
  let currentPageId = $derived(pageToShow(pages, pageId).id);

  // Vérifie que l'interface et le Rust communiquent bien.
  getAppVersion()
    .then((v) => (version = v))
    .catch((e: Error) => (error = e.message));

  void loadSettings();
</script>

<div class="window">
  <TitleBar />
  <div class="content">
    {#if showSettings}
      <SettingsScreen onclose={() => (showSettings = false)} />
    {:else}
      <main>
        {#if editing || pages.length > 1}
          <PageTabs currentId={currentPageId} {editing} onselect={(id) => (pageId = id)} />
        {/if}
        <Dashboard {editing} pageId={currentPageId} />

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
  </div>
</div>

<style>
  .window {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .content {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  main {
    display: flex;
    flex-direction: column;
    min-height: 100%;
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
