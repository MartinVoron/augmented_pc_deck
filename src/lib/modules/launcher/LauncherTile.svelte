<!--
  Tuile du lanceur : un bouton par raccourci. Un clic ouvre la cible ;
  si elle n'existe plus, un message clair s'affiche dans la tuile.
-->
<script lang="ts">
  import type { TileConfig } from "../../grid/defaultLayout";
  import type { AppError } from "../../tauri";
  import { Plus } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { launch, type Shortcut } from ".";
  import { onFileDrop } from "./files";
  import ShortcutIcon from "./ShortcutIcon.svelte";
  import ShortcutsDialog from "./ShortcutsDialog.svelte";
  import { shortcutFromTarget } from "./shortcuts";
  import { updateShortcuts } from "./state";

  let { tile }: { tile: TileConfig } = $props();

  let shortcuts = $derived(tile.shortcuts ?? []);
  let error = $state("");
  let errorTimer: ReturnType<typeof setTimeout> | undefined;
  let showDialog = $state(false);
  let root: HTMLElement | undefined = $state();

  // Un fichier déposé sur la tuile depuis l'Explorateur devient un raccourci.
  onMount(() =>
    onFileDrop((paths, element) => {
      if (!root || !element || !root.closest(".tile")?.contains(element)) return;
      void updateShortcuts(tile.id, (list) =>
        paths.reduce((acc, path) => [...acc, shortcutFromTarget(acc, path)], list),
      );
    }),
  );

  async function open(shortcut: Shortcut) {
    clearTimeout(errorTimer);
    error = "";
    try {
      await launch(shortcut);
    } catch (e) {
      const { code, message } = e as AppError;
      error = code === "not_found" ? `${message}. Modifiez ou supprimez ce raccourci.` : message;
      errorTimer = setTimeout(() => (error = ""), 8000);
    }
  }
</script>

<div class="launcher" bind:this={root}>
  {#if shortcuts.length === 0}
    <div class="empty">
      <p>Aucun raccourci pour l'instant. Glissez un fichier ici ou :</p>
      <button type="button" class="add" onclick={() => (showDialog = true)}>
        <Plus size={14} /> Ajouter un raccourci
      </button>
    </div>
  {:else}
    <ul>
      {#each shortcuts as shortcut (shortcut.id)}
        <li>
          <button type="button" title={shortcut.target} onclick={() => open(shortcut)}>
            <span class="icon">
              <ShortcutIcon target={shortcut.target} icon={shortcut.icon} size={28} />
            </span>
            <span class="name">{shortcut.name}</span>
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  {#if error}
    <p class="error" role="alert">{error}</p>
  {/if}
</div>

{#if showDialog}
  <ShortcutsDialog tileId={tile.id} onclose={() => (showDialog = false)} />
{/if}

<style>
  .launcher {
    position: relative;
    min-height: 100%;
  }

  ul {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(3.5rem, 1fr));
    gap: 0.25rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  button {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.15rem;
    width: 100%;
    padding: 0.3rem 0.15rem;
    border: 1px solid transparent;
    border-radius: 0.5rem;
    background: transparent;
    color: var(--color-text);
    font: inherit;
    cursor: pointer;
  }

  button:hover {
    border-color: var(--color-border);
    background: var(--color-bg);
  }

  button:active {
    transform: scale(0.96);
  }

  .icon {
    display: inline-flex;
    color: var(--tile-color);
  }

  .name {
    max-width: 100%;
    overflow: hidden;
    font-size: 0.7rem;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.35rem;
  }

  .empty p {
    margin: 0;
  }

  button.add {
    flex-direction: row;
    width: auto;
    padding: 0.25rem 0.5rem;
    border-color: var(--color-border);
    font-size: 0.8rem;
  }

  .error {
    position: absolute;
    right: 0;
    bottom: 0;
    left: 0;
    margin: 0;
    padding: 0.3rem 0.5rem;
    border-radius: 0.375rem;
    background: #2a1a1a;
    color: #f1c4c4;
    font-size: 0.75rem;
    line-height: 1.3;
  }
</style>
