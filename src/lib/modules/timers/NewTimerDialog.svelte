<!--
  Création d'un compte à rebours : un nom et une durée (avec des durées toutes prêtes).
-->
<script lang="ts">
  import { X } from "@lucide/svelte";

  let {
    onsave,
    onclose,
  }: { onsave: (name: string, durationSecs: number) => void; onclose: () => void } = $props();

  const PRESETS = [1, 5, 10, 15, 25, 45];

  let name = $state("");
  let minutes = $state(5);
  let seconds = $state(0);

  let durationSecs = $derived(
    Math.max(0, Math.floor(minutes || 0)) * 60 + Math.max(0, Math.floor(seconds || 0)),
  );
  let valid = $derived(durationSecs > 0 && durationSecs <= 24 * 3600);

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!valid) return;
    onsave(name.trim() || `Minuteur ${minutes} min`, durationSecs);
  }
</script>

<svelte:window onkeydown={(event) => event.key === "Escape" && onclose()} />

<div class="backdrop">
  <div class="dialog" role="dialog" aria-modal="true" aria-label="Nouveau minuteur">
    <form onsubmit={submit}>
      <header>
        <h2>Nouveau minuteur</h2>
        <button type="button" class="icon-button" aria-label="Fermer" onclick={onclose}>
          <X size={18} />
        </button>
      </header>

      <label>
        <span>Nom</span>
        <!-- svelte-ignore a11y_autofocus -->
        <input bind:value={name} placeholder="Thé, lessive, pause…" autofocus />
      </label>

      <div class="duration">
        <label>
          <span>Minutes</span>
          <input type="number" min="0" max="1440" bind:value={minutes} />
        </label>
        <label>
          <span>Secondes</span>
          <input type="number" min="0" max="59" bind:value={seconds} />
        </label>
      </div>

      <div class="presets" role="group" aria-label="Durées toutes prêtes">
        {#each PRESETS as preset (preset)}
          <button
            type="button"
            aria-pressed={minutes === preset && seconds === 0}
            onclick={() => {
              minutes = preset;
              seconds = 0;
            }}
          >
            {preset} min
          </button>
        {/each}
      </div>

      {#if !valid}
        <p class="hint">La durée doit être comprise entre 1 seconde et 24 heures.</p>
      {/if}

      <div class="actions">
        <button type="button" onclick={onclose}>Annuler</button>
        <button type="submit" class="primary" disabled={!valid}>Créer</button>
      </div>
    </form>
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

  form {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .dialog {
    width: min(22rem, 100%);
    padding: 1rem;
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

  label {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 0.25rem;
    color: var(--color-text-muted);
  }

  input {
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: var(--color-bg);
    color: var(--color-text);
    font: inherit;
  }

  .duration,
  .presets,
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .actions {
    justify-content: flex-end;
  }

  button {
    padding: 0.3rem 0.6rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-text);
    font: inherit;
    cursor: pointer;
  }

  .presets button {
    padding: 0.2rem 0.5rem;
    font-size: 0.8rem;
  }

  button[aria-pressed="true"] {
    border-color: var(--color-warning);
    color: var(--color-warning);
  }

  button:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .primary {
    border-color: var(--color-accent);
    background: var(--color-accent);
    color: #fff;
  }

  .icon-button {
    display: inline-flex;
    padding: 0.25rem;
    color: var(--color-text-muted);
  }

  .hint {
    margin: 0;
    color: var(--color-warning);
    font-size: 0.8rem;
  }
</style>
