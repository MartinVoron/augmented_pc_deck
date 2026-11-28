<!--
  Liste des processus : triable par nom, processeur ou mémoire, filtrable par nom,
  rafraîchie toutes les 2 secondes. Un processus s'arrête après confirmation.
-->
<script lang="ts">
  import { portal } from "../../actions/portal";
  import { ArrowDown, ArrowUp, Search, Square, X } from "@lucide/svelte";
  import { onMount } from "svelte";
  import {
    filterProcesses,
    formatBytes,
    killProcess,
    listProcesses,
    sortProcesses,
    type ProcessInfo,
    type SortDirection,
    type SortKey,
  } from ".";
  import { poll } from "../../polling";

  let { onclose }: { onclose: () => void } = $props();

  const COLUMNS: { key: SortKey; label: string }[] = [
    { key: "name", label: "Nom" },
    { key: "cpu", label: "Processeur" },
    { key: "memory", label: "Mémoire" },
  ];
  const cpuFormat = new Intl.NumberFormat("fr-FR", {
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  });

  let processes = $state<ProcessInfo[]>([]);
  let loaded = $state(false);
  let query = $state("");
  let sortKey = $state<SortKey>("memory");
  let direction = $state<SortDirection>("desc");
  /** Processus dont l'arrêt attend une confirmation. */
  let confirming = $state<ProcessInfo | null>(null);
  let message = $state<{ kind: "ok" | "error"; text: string } | null>(null);

  let shown = $derived(sortProcesses(filterProcesses(processes, query), sortKey, direction));

  async function refresh() {
    try {
      processes = await listProcesses();
    } catch (e) {
      message = { kind: "error", text: (e as Error).message };
    } finally {
      loaded = true;
    }
  }

  onMount(() => poll(refresh, 2000));

  function sortBy(key: SortKey) {
    if (sortKey === key) {
      direction = direction === "asc" ? "desc" : "asc";
    } else {
      sortKey = key;
      direction = key === "name" ? "asc" : "desc";
    }
  }

  async function kill(process: ProcessInfo) {
    confirming = null;
    try {
      await killProcess(process);
      message = { kind: "ok", text: `« ${process.name} » a été arrêté.` };
      processes = processes.filter((p) => p.pid !== process.pid);
    } catch (e) {
      message = { kind: "error", text: (e as Error).message };
    }
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key !== "Escape") return;
    if (confirming) confirming = null;
    else onclose();
  }}
/>

<div class="backdrop" use:portal>
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="processes-title">
    <header>
      <h2 id="processes-title">Processus</h2>
      <label class="search">
        <Search size={14} />
        <input bind:value={query} placeholder="Filtrer par nom" aria-label="Filtrer par nom" />
      </label>
      <button type="button" class="icon-button" aria-label="Fermer" onclick={onclose}>
        <X size={18} />
      </button>
    </header>

    {#if message}
      <p
        class="message"
        data-kind={message.kind}
        role={message.kind === "error" ? "alert" : "status"}
      >
        {message.text}
      </p>
    {/if}

    <div class="table">
      <table>
        <thead>
          <tr>
            {#each COLUMNS as column (column.key)}
              <th
                class={column.key}
                aria-sort={sortKey === column.key
                  ? direction === "asc"
                    ? "ascending"
                    : "descending"
                  : "none"}
              >
                <button type="button" onclick={() => sortBy(column.key)}>
                  {column.label}
                  {#if sortKey === column.key}
                    {#if direction === "asc"}<ArrowUp size={12} />{:else}<ArrowDown
                        size={12}
                      />{/if}
                  {/if}
                </button>
              </th>
            {/each}
            <th class="action"><span class="sr-only">Action</span></th>
          </tr>
        </thead>
        <tbody>
          {#each shown as process (process.pid)}
            <tr>
              <td class="name" title="PID {process.pid}">{process.name}</td>
              <td class="cpu">{cpuFormat.format(process.cpu)} %</td>
              <td class="memory">{formatBytes(process.memory)}</td>
              <td class="action">
                {#if confirming?.pid === process.pid}
                  <span class="confirm">
                    <button type="button" class="danger" onclick={() => kill(process)}>
                      Oui, arrêter
                    </button>
                    <button type="button" onclick={() => (confirming = null)}>Annuler</button>
                  </span>
                {:else}
                  <button
                    type="button"
                    aria-label="Arrêter {process.name}"
                    onclick={() => {
                      message = null;
                      confirming = process;
                    }}
                  >
                    <Square size={12} /> Arrêter
                  </button>
                {/if}
              </td>
            </tr>
          {:else}
            <tr>
              <td colspan="4" class="empty">
                {loaded ? "Aucun processus ne correspond." : "Chargement…"}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    <p class="count">{shown.length} processus sur {processes.length}</p>
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
    gap: 0.6rem;
    width: min(44rem, 100%);
    height: min(36rem, 100%);
    padding: 1rem;
    border: 1px solid var(--color-border);
    border-radius: var(--tile-radius);
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 0.85rem;
  }

  header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  h2 {
    margin: 0;
    font-size: 1rem;
  }

  .search {
    display: flex;
    flex: 1;
    align-items: center;
    gap: 0.4rem;
    padding: 0 0.5rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: var(--color-bg);
    color: var(--color-text-muted);
  }

  .search input {
    flex: 1;
    min-width: 0;
    padding: 0.35rem 0;
    border: 0;
    outline: none;
    background: transparent;
    color: var(--color-text);
    font: inherit;
  }

  .table {
    flex: 1;
    min-height: 0;
    overflow: auto;
    border: 1px solid var(--color-border);
    border-radius: 0.5rem;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  thead th {
    position: sticky;
    top: 0;
    background: var(--color-surface-hover);
    text-align: left;
  }

  th button {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.4rem 0.5rem;
    border: 0;
    background: none;
    color: var(--color-text-muted);
    font: inherit;
    font-weight: 600;
    cursor: pointer;
  }

  td {
    padding: 0.25rem 0.5rem;
    border-top: 1px solid var(--color-border);
  }

  tbody tr:hover {
    background: var(--color-surface-hover);
  }

  .cpu,
  .memory {
    width: 7rem;
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  th.cpu button,
  th.memory button {
    justify-content: flex-end;
    width: 100%;
  }

  td.name {
    max-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .action {
    width: 11rem;
    text-align: right;
  }

  td button,
  .icon-button {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.15rem 0.45rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-text-muted);
    font: inherit;
    font-size: 0.75rem;
    cursor: pointer;
  }

  td button:hover {
    color: var(--color-text);
  }

  .confirm {
    display: inline-flex;
    gap: 0.25rem;
  }

  button.danger {
    border-color: var(--color-danger);
    background: var(--color-danger);
    color: #fff;
  }

  .icon-button {
    padding: 0.25rem;
  }

  .empty {
    color: var(--color-text-muted);
    text-align: center;
  }

  .message {
    margin: 0;
    padding: 0.35rem 0.6rem;
    border-radius: 0.375rem;
    background: var(--color-success-bg);
    color: var(--color-success-text);
  }

  .message[data-kind="error"] {
    background: var(--color-danger-bg);
    color: var(--color-danger-text);
  }

  .count {
    margin: 0;
    color: var(--color-text-muted);
    font-size: 0.75rem;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }
</style>
