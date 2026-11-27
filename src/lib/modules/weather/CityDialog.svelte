<!--
  Choix de la ville d'une tuile Météo : on tape un nom, puis on choisit parmi les
  villes trouvées (avec leur région, pour distinguer les homonymes).
-->
<script lang="ts">
  import { MapPin, Search, X } from "@lucide/svelte";
  import { DEFAULT_PLACE, searchPlaces, type Place } from ".";
  import { findTile, updateTile } from "../../grid/state";
  import { settings } from "../settings/state.svelte";

  let { tileId, onclose }: { tileId: string; onclose: () => void } = $props();

  let tile = $derived(findTile(settings.config, tileId));
  let current = $derived(tile?.location ?? DEFAULT_PLACE);

  let query = $state("");
  let results = $state<Place[]>([]);
  let searched = $state(false);
  let searching = $state(false);
  let error = $state("");

  async function search(event: SubmitEvent) {
    event.preventDefault();
    if (query.trim().length < 2) return;
    searching = true;
    try {
      results = await searchPlaces(query);
      searched = true;
      error = "";
    } catch (e) {
      error = (e as Error).message;
    } finally {
      searching = false;
    }
  }

  async function choose(place: Place) {
    await updateTile(tileId, (t) => (t.location = place));
    onclose();
  }
</script>

<svelte:window onkeydown={(event) => event.key === "Escape" && onclose()} />

<div class="backdrop">
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="city-title">
    <header>
      <h2 id="city-title">Ville de la météo</h2>
      <button type="button" class="icon-button" aria-label="Fermer" onclick={onclose}>
        <X size={18} />
      </button>
    </header>

    <p class="hint">
      Actuellement : <strong>{current.name}</strong>
      {#if current.detail}({current.detail}){/if}
    </p>

    <form onsubmit={search}>
      <input
        aria-label="Nom de la ville"
        placeholder="Caen, Paris, Bayeux…"
        bind:value={query}
        autocomplete="off"
      />
      <button type="submit" class="primary" disabled={searching || query.trim().length < 2}>
        <Search size={14} />
        {searching ? "Recherche…" : "Rechercher"}
      </button>
    </form>

    {#if error}<p class="error" role="alert">{error}</p>{/if}

    {#if results.length > 0}
      <ul>
        {#each results as place (`${place.latitude},${place.longitude}`)}
          <li>
            <button type="button" class="place" onclick={() => void choose(place)}>
              <MapPin size={14} />
              <span class="name">{place.name}</span>
              <span class="detail">{place.detail}</span>
            </button>
          </li>
        {/each}
      </ul>
    {:else if searched}
      <p class="hint">Aucune ville trouvée. Vérifiez l'orthographe.</p>
    {/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 10;
    display: grid;
    place-items: center;
    padding: 1rem;
    background: rgb(0 0 0 / 55%);
  }

  .dialog {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    width: min(26rem, 100%);
    max-height: 100%;
    padding: 1rem;
    overflow: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--tile-radius);
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 0.875rem;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  h2 {
    margin: 0;
    font-size: 1rem;
  }

  form {
    display: flex;
    gap: 0.5rem;
  }

  input {
    flex: 1;
    min-width: 0;
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: var(--color-bg);
    color: var(--color-text);
    font: inherit;
  }

  button {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.35rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    background: transparent;
    color: var(--color-text);
    font: inherit;
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .primary {
    border-color: var(--color-accent);
    background: var(--color-accent);
    color: #fff;
  }

  .icon-button {
    padding: 0.2rem;
    border: none;
    color: var(--color-text-muted);
  }

  ul {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .place {
    width: 100%;
    border-color: transparent;
    text-align: left;
  }

  .place:hover {
    border-color: var(--color-border);
    background: var(--color-surface-hover);
  }

  .name {
    font-weight: 600;
  }

  .detail {
    color: var(--color-text-muted);
  }

  .hint {
    margin: 0;
    color: var(--color-text-muted);
  }

  .error {
    margin: 0;
    color: #f1c4c4;
  }
</style>
