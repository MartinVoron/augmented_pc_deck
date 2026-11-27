<!--
  Fenêtre de gestion des raccourcis d'une tuile du lanceur : ajouter, modifier,
  réordonner et supprimer. La cible se choisit avec le sélecteur de Windows,
  en la tapant (URL) ou en déposant un fichier depuis l'Explorateur.
-->
<script lang="ts">
  import {
    ArrowDown,
    ArrowUp,
    File,
    Folder,
    ImagePlus,
    Pencil,
    Plus,
    Trash2,
    Undo2,
    X,
  } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { clearIconCache, readImage, type Shortcut } from ".";
  import { findTile } from "../../grid/state";
  import { settings } from "../settings/state.svelte";
  import { onFileDrop, pickPath } from "./files";
  import ShortcutIcon from "./ShortcutIcon.svelte";
  import {
    defaultName,
    moveShortcut,
    removeShortcut,
    saveShortcut,
    shortcutFromTarget,
  } from "./shortcuts";
  import { updateShortcuts } from "./state";

  let { tileId, onclose }: { tileId: string; onclose: () => void } = $props();

  let tile = $derived(findTile(settings.config, tileId));
  let shortcuts = $derived(tile?.shortcuts ?? []);

  /** Raccourci en cours de modification (nouveau ou existant). */
  let draft = $state<Shortcut | null>(null);
  let error = $state("");

  let isNew = $derived(draft !== null && !shortcuts.some((s) => s.id === draft?.id));
  let isProgram = $derived(draft !== null && /\.(exe|bat|cmd|com)"?$/i.test(draft.target.trim()));
  let canSave = $derived(draft !== null && draft.name.trim() !== "" && draft.target.trim() !== "");

  function save(change: (list: Shortcut[]) => Shortcut[]) {
    void updateShortcuts(tileId, change);
  }

  function startNew(target = "") {
    error = "";
    draft = { ...shortcutFromTarget(shortcuts, target), name: target ? defaultName(target) : "" };
  }

  function edit(shortcut: Shortcut) {
    error = "";
    draft = { ...shortcut, args: shortcut.args ?? "" };
  }

  /** Change la cible ; le nom suit tant qu'il n'a pas été personnalisé. */
  function setTarget(target: string) {
    if (!draft) return;
    if (!draft.name.trim() || draft.name === defaultName(draft.target)) {
      draft.name = defaultName(target);
    }
    draft.target = target;
  }

  async function chooseTarget(kind: "file" | "folder") {
    try {
      const path = await pickPath(kind);
      if (path) setTarget(path);
    } catch (e) {
      error = (e as Error).message;
    }
  }

  async function chooseIcon() {
    if (!draft) return;
    try {
      const path = await pickPath("image");
      if (path) draft.icon = await readImage(path);
      error = "";
    } catch (e) {
      error = (e as Error).message;
    }
  }

  function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!draft || !canSave) return;
    const shortcut = $state.snapshot(draft);
    save((list) => saveShortcut(list, shortcut));
    clearIconCache();
    draft = null;
  }

  onMount(() =>
    onFileDrop((paths) => {
      if (draft) {
        setTarget(paths[0]);
      } else {
        save((list) => paths.reduce((acc, p) => [...acc, shortcutFromTarget(acc, p)], list));
      }
    }),
  );
</script>

<svelte:window onkeydown={(event) => event.key === "Escape" && onclose()} />

<div class="backdrop">
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="shortcuts-title">
    <header>
      <h2 id="shortcuts-title">Raccourcis de « {tile?.title ?? "Raccourcis"} »</h2>
      <button type="button" class="icon-button" aria-label="Fermer" onclick={onclose}>
        <X size={18} />
      </button>
    </header>

    {#if draft}
      <form onsubmit={submit}>
        <h3>{isNew ? "Nouveau raccourci" : "Modifier le raccourci"}</h3>

        <label>
          <span>Cible : programme, fichier, dossier ou URL</span>
          <input
            value={draft.target}
            oninput={(event) => setTarget(event.currentTarget.value)}
            placeholder="C:\Program Files\… ou https://…"
            required
          />
        </label>
        <div class="row">
          <button type="button" onclick={() => chooseTarget("file")}>
            <File size={14} /> Fichier…
          </button>
          <button type="button" onclick={() => chooseTarget("folder")}>
            <Folder size={14} /> Dossier…
          </button>
          <span class="hint">ou glissez un fichier depuis l'Explorateur</span>
        </div>

        <label>
          <span>Nom</span>
          <input bind:value={draft.name} required />
        </label>

        <label>
          <span>Arguments de lancement</span>
          <input
            bind:value={draft.args}
            disabled={!isProgram}
            placeholder={isProgram ? '--profil "Mon profil"' : "Seulement pour un programme"}
          />
        </label>

        <div class="icon-row">
          <span class="preview"><ShortcutIcon target={draft.target} icon={draft.icon} /></span>
          <button type="button" onclick={chooseIcon}>
            <ImagePlus size={14} /> Icône personnalisée…
          </button>
          {#if draft.icon}
            <button type="button" onclick={() => draft && (draft.icon = undefined)}>
              <Undo2 size={14} /> Icône d'origine
            </button>
          {/if}
        </div>

        {#if error}<p class="error" role="alert">{error}</p>{/if}

        <div class="row actions">
          <button type="button" onclick={() => (draft = null)}>Annuler</button>
          <button type="submit" class="primary" disabled={!canSave}>Enregistrer</button>
        </div>
      </form>
    {:else}
      {#if shortcuts.length === 0}
        <p class="hint">
          Aucun raccourci. Ajoutez-en un, ou glissez ici des fichiers depuis l'Explorateur.
        </p>
      {:else}
        <ul>
          {#each shortcuts as shortcut, index (shortcut.id)}
            <li>
              <span class="preview">
                <ShortcutIcon target={shortcut.target} icon={shortcut.icon} />
              </span>
              <span class="text">
                <span class="name">{shortcut.name}</span>
                <span class="target">{shortcut.target}</span>
              </span>
              <button
                type="button"
                class="icon-button"
                aria-label="Monter {shortcut.name}"
                disabled={index === 0}
                onclick={() => save((list) => moveShortcut(list, shortcut.id, -1))}
              >
                <ArrowUp size={14} />
              </button>
              <button
                type="button"
                class="icon-button"
                aria-label="Descendre {shortcut.name}"
                disabled={index === shortcuts.length - 1}
                onclick={() => save((list) => moveShortcut(list, shortcut.id, 1))}
              >
                <ArrowDown size={14} />
              </button>
              <button
                type="button"
                class="icon-button"
                aria-label="Modifier {shortcut.name}"
                onclick={() => edit(shortcut)}
              >
                <Pencil size={14} />
              </button>
              <button
                type="button"
                class="icon-button danger"
                aria-label="Supprimer {shortcut.name}"
                onclick={() => save((list) => removeShortcut(list, shortcut.id))}
              >
                <Trash2 size={14} />
              </button>
            </li>
          {/each}
        </ul>
        <p class="hint">Astuce : glissez des fichiers ici pour les ajouter d'un coup.</p>
      {/if}
      <div class="row actions">
        <button type="button" class="primary" onclick={() => startNew()}>
          <Plus size={14} /> Ajouter un raccourci
        </button>
      </div>
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
    width: min(32rem, 100%);
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
    gap: 0.5rem;
  }

  h2,
  h3 {
    margin: 0;
    font-size: 1rem;
  }

  h3 {
    font-size: 0.9rem;
    color: var(--color-text-muted);
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  label {
    display: flex;
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

  input:disabled {
    opacity: 0.5;
  }

  .row,
  .icon-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem;
  }

  .actions {
    justify-content: flex-end;
  }

  button {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.3rem 0.6rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-text);
    font: inherit;
    cursor: pointer;
  }

  button:hover:not(:disabled) {
    background: var(--color-surface-hover);
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

  .primary:hover:not(:disabled) {
    background: #3f7ae6;
  }

  .icon-button {
    padding: 0.25rem;
    color: var(--color-text-muted);
  }

  .icon-button.danger:hover {
    color: var(--color-danger);
  }

  ul {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.3rem 0.4rem;
    border-radius: 0.5rem;
    background: var(--color-bg);
  }

  .preview {
    display: inline-flex;
    flex: none;
    color: var(--color-accent);
  }

  .text {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
  }

  .name,
  .target {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .target {
    color: var(--color-text-muted);
    font-size: 0.75rem;
  }

  .hint {
    margin: 0;
    color: var(--color-text-muted);
    font-size: 0.8rem;
  }

  .error {
    margin: 0;
    padding: 0.3rem 0.5rem;
    border-radius: 0.375rem;
    background: #2a1a1a;
    color: #f1c4c4;
  }
</style>
