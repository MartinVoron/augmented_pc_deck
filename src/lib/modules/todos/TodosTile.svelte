<!--
  Tuile de la liste de tâches : ajouter, modifier (double-clic ou crayon), cocher,
  supprimer et réordonner. Les tâches terminées peuvent être masquées.
  Chaque changement est enregistré tout de suite dans la configuration.
-->
<script lang="ts">
  import { ArrowDown, ArrowUp, Eye, EyeOff, Pencil, Plus, Trash2 } from "@lucide/svelte";
  import { tick } from "svelte";
  import { addTodo, clearDone, editTodo, moveTodo, removeTodo, toggleTodo, type Todo } from ".";
  import type { TileConfig } from "../../grid/defaultLayout";
  import { updateTodos, updateTodosTile } from "./state";

  let { tile }: { tile: TileConfig } = $props();

  let todos = $derived(tile.todos ?? []);
  let hideDone = $derived(tile.hideDone ?? false);
  let visible = $derived(hideDone ? todos.filter((t) => !t.done) : todos);
  let doneCount = $derived(todos.filter((t) => t.done).length);

  let newText = $state("");
  let editingId = $state<string | null>(null);
  let editText = $state("");

  function save(change: (list: Todo[]) => Todo[]) {
    void updateTodos(tile.id, change);
  }

  function add(event: SubmitEvent) {
    event.preventDefault();
    if (!newText.trim()) return;
    save((list) => addTodo(list, newText));
    newText = "";
  }

  async function startEdit(todo: Todo) {
    editingId = todo.id;
    editText = todo.text;
    await tick();
    document.getElementById(`${tile.id}-edit`)?.focus();
  }

  function finishEdit() {
    if (editingId === null) return;
    const id = editingId;
    const text = editText;
    editingId = null;
    save((list) => editTodo(list, id, text));
  }
</script>

<div class="todos">
  <div class="toolbar">
    <span class="count">
      {todos.length - doneCount} à faire{#if doneCount > 0}, {doneCount} terminée{doneCount > 1
          ? "s"
          : ""}{/if}
    </span>
    {#if doneCount > 0}
      <button
        type="button"
        class="icon"
        aria-pressed={hideDone}
        title={hideDone ? "Afficher les tâches terminées" : "Masquer les tâches terminées"}
        aria-label="Masquer les tâches terminées"
        onclick={() => void updateTodosTile(tile.id, (t) => (t.hideDone = !hideDone))}
      >
        {#if hideDone}<EyeOff size={14} />{:else}<Eye size={14} />{/if}
      </button>
      <button
        type="button"
        class="icon danger"
        title="Supprimer les tâches terminées"
        aria-label="Supprimer les tâches terminées"
        onclick={() => save(clearDone)}
      >
        <Trash2 size={14} />
      </button>
    {/if}
  </div>

  <ul>
    {#each visible as todo, index (todo.id)}
      <li data-done={todo.done || undefined}>
        <input
          type="checkbox"
          checked={todo.done}
          aria-label="Terminée : {todo.text}"
          onchange={() => save((list) => toggleTodo(list, todo.id))}
        />
        {#if editingId === todo.id}
          <input
            id="{tile.id}-edit"
            class="edit"
            aria-label="Texte de la tâche"
            bind:value={editText}
            onblur={finishEdit}
            onkeydown={(event) => {
              if (event.key === "Enter") finishEdit();
              if (event.key === "Escape") editingId = null;
            }}
          />
        {:else}
          <span class="text" role="presentation" ondblclick={() => startEdit(todo)}>
            {todo.text}
          </span>
          <span class="actions">
            <button
              type="button"
              class="icon"
              aria-label="Monter {todo.text}"
              disabled={index === 0}
              onclick={() => save((list) => moveTodo(list, todo.id, -1, visible))}
            >
              <ArrowUp size={12} />
            </button>
            <button
              type="button"
              class="icon"
              aria-label="Descendre {todo.text}"
              disabled={index === visible.length - 1}
              onclick={() => save((list) => moveTodo(list, todo.id, 1, visible))}
            >
              <ArrowDown size={12} />
            </button>
            <button
              type="button"
              class="icon"
              aria-label="Modifier {todo.text}"
              onclick={() => startEdit(todo)}
            >
              <Pencil size={12} />
            </button>
            <button
              type="button"
              class="icon danger"
              aria-label="Supprimer {todo.text}"
              onclick={() => save((list) => removeTodo(list, todo.id))}
            >
              <Trash2 size={12} />
            </button>
          </span>
        {/if}
      </li>
    {/each}
  </ul>

  {#if todos.length === 0}
    <p class="empty">Rien à faire pour l'instant.</p>
  {:else if visible.length === 0}
    <p class="empty">Tout est fait !</p>
  {/if}

  <form onsubmit={add}>
    <input bind:value={newText} placeholder="Nouvelle tâche…" aria-label="Nouvelle tâche" />
    <button type="submit" class="icon" aria-label="Ajouter la tâche" disabled={!newText.trim()}>
      <Plus size={14} />
    </button>
  </form>
</div>

<style>
  .todos {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    min-height: 100%;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    font-size: 0.72rem;
  }

  .count {
    flex: 1;
  }

  ul {
    display: flex;
    flex-direction: column;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    min-height: 1.6rem;
    padding: 0 0.25rem;
    border-radius: 0.375rem;
    color: var(--color-text);
  }

  li:hover,
  li:focus-within {
    background: var(--color-bg);
  }

  li[data-done] .text {
    color: var(--color-text-muted);
    text-decoration: line-through;
  }

  input[type="checkbox"] {
    flex: none;
    margin: 0;
    accent-color: var(--tile-color);
  }

  .text {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .actions {
    display: none;
    flex: none;
    gap: 0.1rem;
  }

  li:hover .actions,
  li:focus-within .actions {
    display: flex;
  }

  .icon {
    display: inline-flex;
    padding: 0.2rem;
    border: 0;
    border-radius: 0.3rem;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .icon:hover:not(:disabled),
  .icon[aria-pressed="true"] {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .icon.danger:hover:not(:disabled) {
    color: var(--color-danger);
  }

  .icon:disabled {
    opacity: 0.3;
    cursor: default;
  }

  form {
    display: flex;
    gap: 0.25rem;
    margin-top: auto;
  }

  form input,
  .edit {
    flex: 1;
    min-width: 0;
    padding: 0.2rem 0.4rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: var(--color-bg);
    color: var(--color-text);
    font: inherit;
    font-size: 0.8rem;
  }

  .empty {
    margin: 0;
    font-size: 0.8rem;
  }
</style>
