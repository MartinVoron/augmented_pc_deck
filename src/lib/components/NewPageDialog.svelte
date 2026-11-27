<!--
  Nouvelle page : choisir un modèle prédéfini (ou une page vide) et un nom.
  Les tuiles du modèle sont déjà placées ; on les range ensuite comme on veut.
-->
<script lang="ts">
  import { X } from "@lucide/svelte";
  import { KIND_ICONS } from "../grid/icons";
  import { sanitizePages } from "../grid/layout";
  import { addPage, MAX_PAGE_NAME, PAGE_TEMPLATES, type PageTemplate } from "../grid/pages";
  import { settings, updateSettings } from "../modules/settings/state.svelte";

  let { onclose, oncreate }: { onclose: () => void; oncreate: (pageId: string) => void } = $props();

  let template = $state<PageTemplate>(PAGE_TEMPLATES[0]);
  let name = $state(PAGE_TEMPLATES[0].name);
  /** Le nom suit le modèle choisi tant qu'il n'a pas été personnalisé. */
  let nameEdited = $state(false);

  function choose(next: PageTemplate) {
    if (!nameEdited) name = next.name === "Page vide" ? "Nouvelle page" : next.name;
    template = next;
  }

  async function create(event: SubmitEvent) {
    event.preventDefault();
    const { pages, page } = addPage(sanitizePages(settings.config?.layout.pages), template, name);
    await updateSettings((config) => (config.layout.pages = pages));
    oncreate(page.id);
  }
</script>

<svelte:window onkeydown={(event) => event.key === "Escape" && onclose()} />

<div class="backdrop">
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="new-page-title">
    <form onsubmit={create}>
      <header>
        <h2 id="new-page-title">Nouvelle page</h2>
        <button type="button" class="icon-button" aria-label="Fermer" onclick={onclose}>
          <X size={18} />
        </button>
      </header>

      <fieldset>
        <legend>Modèle</legend>
        {#each PAGE_TEMPLATES as option (option.id)}
          <label class="template" class:selected={option.id === template.id}>
            <input
              type="radio"
              name="template"
              value={option.id}
              checked={option.id === template.id}
              onchange={() => choose(option)}
            />
            <span class="text">
              <span class="name">{option.name}</span>
              <span class="description">{option.description}</span>
            </span>
            <span class="icons" aria-hidden="true">
              {#each option.tiles as tile, i (i)}
                {@const Icon = KIND_ICONS[tile.kind]}
                <Icon size={14} />
              {/each}
            </span>
          </label>
        {/each}
      </fieldset>

      <label class="field">
        <span>Nom de la page</span>
        <input
          bind:value={name}
          maxlength={MAX_PAGE_NAME}
          required
          oninput={() => (nameEdited = true)}
        />
      </label>

      <div class="buttons">
        <button type="button" onclick={onclose}>Annuler</button>
        <button type="submit" class="primary">Créer la page</button>
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

  .dialog {
    width: min(30rem, 100%);
    max-height: 100%;
    padding: 1rem;
    overflow: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--tile-radius);
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 0.875rem;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
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

  fieldset {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin: 0;
    padding: 0;
    border: none;
  }

  legend {
    margin-bottom: 0.35rem;
    color: var(--color-text-muted);
  }

  .template {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.5rem 0.6rem;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
    cursor: pointer;
  }

  .template.selected {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
  }

  .text {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
  }

  .name {
    font-weight: 600;
  }

  .description {
    color: var(--color-text-muted);
    font-size: 0.8rem;
  }

  .icons {
    display: flex;
    gap: 0.2rem;
    color: var(--color-text-muted);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    color: var(--color-text-muted);
  }

  .field input {
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: var(--color-bg);
    color: var(--color-text);
    font: inherit;
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
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
</style>
