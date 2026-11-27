<!--
  Onglets des pages de tuiles. Un clic change de page.
  En mode édition : renommer, déplacer et supprimer la page affichée, et ajouter
  une page (vide ou à partir d'un modèle). Chaque changement est enregistré tout de suite.
-->
<script lang="ts">
  import { ArrowLeft, ArrowRight, Check, Pencil, Plus, Trash2, X } from "@lucide/svelte";
  import { tick } from "svelte";
  import type { PageConfig } from "../grid/defaultLayout";
  import { sanitizePages } from "../grid/layout";
  import { MAX_PAGE_NAME, movePage, removePage, renamePage } from "../grid/pages";
  import { settings, updateSettings } from "../modules/settings/state.svelte";
  import NewPageDialog from "./NewPageDialog.svelte";

  let {
    currentId,
    editing = false,
    onselect,
  }: { currentId: string; editing?: boolean; onselect: (id: string) => void } = $props();

  let pages = $derived(sanitizePages(settings.config?.layout.pages));
  let current = $derived(pages.find((p) => p.id === currentId) ?? pages[0]);
  let index = $derived(pages.indexOf(current));
  let deleteDetail = $derived(
    current.tiles.length === 0
      ? ""
      : current.tiles.length === 1
        ? " et sa tuile"
        : ` et ses ${current.tiles.length} tuiles`,
  );

  let renaming = $state(false);
  let draftName = $state("");
  let confirmingDelete = $state(false);
  let creating = $state(false);

  // Quitter le mode édition ou changer de page annule les actions en cours.
  $effect(() => {
    void currentId;
    if (!editing) creating = false;
    renaming = false;
    confirmingDelete = false;
  });

  function save(next: PageConfig[]) {
    void updateSettings((config) => (config.layout.pages = next));
  }

  async function startRename() {
    draftName = current.name;
    renaming = true;
    await tick();
    document.getElementById("page-name")?.focus();
  }

  function finishRename(event?: SubmitEvent) {
    event?.preventDefault();
    if (renaming) save(renamePage(pages, current.id, draftName));
    renaming = false;
  }

  function remove() {
    const next = removePage(pages, current.id);
    confirmingDelete = false;
    save(next);
    onselect(next[Math.max(0, index - 1)].id);
  }

  /** Flèches gauche et droite pour passer d'un onglet à l'autre. */
  function onkeydown(event: KeyboardEvent) {
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    const next = pages[index + (event.key === "ArrowLeft" ? -1 : 1)];
    if (!next) return;
    event.preventDefault();
    onselect(next.id);
    void tick().then(() => document.getElementById(`tab-${next.id}`)?.focus());
  }
</script>

<nav class="pages" aria-label="Pages">
  <div class="tabs" role="tablist" aria-label="Pages de tuiles" tabindex="-1" {onkeydown}>
    {#each pages as page (page.id)}
      <button
        type="button"
        role="tab"
        id="tab-{page.id}"
        aria-selected={page.id === current.id}
        tabindex={page.id === current.id ? 0 : -1}
        onclick={() => onselect(page.id)}
      >
        {page.name}
        {#if editing}<span class="count">{page.tiles.length}</span>{/if}
      </button>
    {/each}
    {#if editing}
      <button type="button" class="add" onclick={() => (creating = true)}>
        <Plus size={14} /> Nouvelle page
      </button>
    {/if}
  </div>

  {#if editing}
    <div class="actions">
      {#if renaming}
        <form onsubmit={finishRename}>
          <input
            id="page-name"
            aria-label="Nom de la page"
            maxlength={MAX_PAGE_NAME}
            bind:value={draftName}
            onkeydown={(event) => event.key === "Escape" && (renaming = false)}
          />
          <button type="submit" class="icon-button" aria-label="Valider le nom">
            <Check size={14} />
          </button>
          <button
            type="button"
            class="icon-button"
            aria-label="Annuler"
            onclick={() => (renaming = false)}
          >
            <X size={14} />
          </button>
        </form>
      {:else if confirmingDelete}
        <span class="warning">
          Supprimer « {current.name} »{deleteDetail} ?
        </span>
        <button type="button" onclick={() => (confirmingDelete = false)}>Annuler</button>
        <button type="button" class="danger" onclick={remove}>Supprimer</button>
      {:else}
        <button
          type="button"
          class="icon-button"
          aria-label="Renommer la page {current.name}"
          onclick={startRename}
        >
          <Pencil size={14} />
        </button>
        <button
          type="button"
          class="icon-button"
          aria-label="Déplacer la page {current.name} à gauche"
          disabled={index === 0}
          onclick={() => save(movePage(pages, current.id, -1))}
        >
          <ArrowLeft size={14} />
        </button>
        <button
          type="button"
          class="icon-button"
          aria-label="Déplacer la page {current.name} à droite"
          disabled={index === pages.length - 1}
          onclick={() => save(movePage(pages, current.id, 1))}
        >
          <ArrowRight size={14} />
        </button>
        <button
          type="button"
          class="icon-button danger"
          aria-label="Supprimer la page {current.name}"
          disabled={pages.length <= 1}
          title={pages.length <= 1 ? "Il faut garder au moins une page" : undefined}
          onclick={() => (confirmingDelete = true)}
        >
          <Trash2 size={14} />
        </button>
      {/if}
    </div>
  {/if}
</nav>

{#if creating}
  <NewPageDialog
    onclose={() => (creating = false)}
    oncreate={(id) => {
      creating = false;
      onselect(id);
    }}
  />
{/if}

<style>
  .pages {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: var(--tile-gap) var(--tile-gap) 0;
  }

  .tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    min-width: 0;
  }

  [role="tab"],
  .add {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.35rem 0.8rem;
    border: 1px solid transparent;
    border-radius: 999px;
    background: transparent;
    color: var(--color-text-muted);
    font: inherit;
    font-size: 0.875rem;
    cursor: pointer;
  }

  [role="tab"]:hover {
    color: var(--color-text);
    background: var(--color-surface);
  }

  [role="tab"][aria-selected="true"] {
    border-color: var(--color-border);
    background: var(--color-surface);
    color: var(--color-text);
    font-weight: 600;
  }

  .count {
    min-width: 1.2rem;
    padding: 0 0.3rem;
    border-radius: 999px;
    background: var(--color-bg);
    color: var(--color-text-muted);
    font-size: 0.7rem;
    font-weight: 400;
  }

  .add {
    border: 1px dashed var(--color-border);
  }

  .actions,
  form {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.875rem;
  }

  input {
    width: 12rem;
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: var(--color-bg);
    color: var(--color-text);
    font: inherit;
  }

  .actions > button:not(.icon-button) {
    padding: 0.25rem 0.6rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-text);
    font: inherit;
    cursor: pointer;
  }

  .actions > button.danger:not(.icon-button) {
    border-color: var(--color-danger);
    color: var(--color-danger);
  }

  .icon-button {
    display: inline-flex;
    padding: 0.25rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .icon-button:hover:not(:disabled) {
    color: var(--color-text);
    background: var(--color-surface);
  }

  .icon-button.danger:hover:not(:disabled) {
    color: var(--color-danger);
  }

  .icon-button:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .warning {
    color: var(--color-text-muted);
  }
</style>
