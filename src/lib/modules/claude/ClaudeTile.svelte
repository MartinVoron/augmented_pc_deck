<!--
  Tuile « Claude » : notifications récentes de Claude Code (projet, message, heure).
  Elle se met à jour dès qu'un événement arrive. Les sessions qui attendent encore
  une réponse sont mises en avant. En 1×1 : le nombre de sessions en attente.
-->
<script lang="ts">
  import { CircleCheck, MessageCircleQuestion, Trash2 } from "@lucide/svelte";
  import { onMount } from "svelte";
  import {
    CHANGED_EVENT,
    clearClaudeEvents,
    getClaudeEvents,
    waitingSessions,
    type ClaudeEvent,
  } from ".";
  import { onEvent } from "../../events";
  import type { TileConfig } from "../../grid/defaultLayout";
  import { timeAgo } from "../github";

  let { tile }: { tile: TileConfig } = $props();

  let events = $state<ClaudeEvent[]>([]);
  let loaded = $state(false);
  let error = $state("");
  let now = $state(Date.now());

  let compact = $derived(tile.size === "1x1");
  let waiting = $derived(waitingSessions(events));
  let waitingIds = $derived(new Set(waiting.map((e) => e.id)));

  onMount(() => {
    getClaudeEvents()
      .then((list) => (events = list))
      .catch((e: Error) => (error = e.message))
      .finally(() => (loaded = true));
    return onEvent<ClaudeEvent[]>(CHANGED_EVENT, (list) => {
      events = list;
      now = Date.now();
    });
  });

  $effect(() => {
    const interval = setInterval(() => (now = Date.now()), 30_000);
    return () => clearInterval(interval);
  });

  async function clear() {
    try {
      await clearClaudeEvents();
      events = [];
      error = "";
    } catch (e) {
      error = (e as Error).message;
    }
  }

  function received(ms: number): string {
    return timeAgo(new Date(ms).toISOString(), now);
  }
</script>

<div class="claude" data-size={tile.size}>
  {#if compact}
    <div class="summary" class:attention={waiting.length > 0}>
      {#if waiting.length > 0}
        <strong>{waiting.length}</strong>
        <span>{waiting.length > 1 ? "en attente" : "attend"}</span>
        <span class="project">{waiting[0].project}</span>
      {:else if events.length > 0}
        <CircleCheck size={26} />
        <span>Terminé</span>
        <span class="project">{events[0].project} · {received(events[0].receivedAt)}</span>
      {:else}
        <span class="hint">Rien pour l'instant</span>
      {/if}
    </div>
  {:else}
    <header>
      <span class="status">
        {#if waiting.length > 0}
          <strong class="attention">{waiting.length}</strong>
          {waiting.length > 1 ? "sessions en attente" : "session en attente"}
        {:else}
          Rien en attente
        {/if}
      </span>
      {#if events.length > 0}
        <button
          type="button"
          class="icon-button"
          aria-label="Effacer les notifications"
          onclick={clear}
        >
          <Trash2 size={13} />
        </button>
      {/if}
    </header>

    {#if error}<p class="error" role="alert">{error}</p>{/if}

    {#if loaded && events.length === 0}
      <p class="hint">Aucune notification. Mise en place du hook : Paramètres › Claude.</p>
    {:else}
      <ul>
        {#each events as event (event.id)}
          <li class:active={waitingIds.has(event.id)} data-kind={event.kind}>
            <span class="icon">
              {#if event.kind === "waiting"}
                <MessageCircleQuestion size={14} />
              {:else}
                <CircleCheck size={14} />
              {/if}
            </span>
            <span class="text">
              <span class="line">
                <span class="project">{event.project}</span>
                <span class="time">{received(event.receivedAt)}</span>
              </span>
              <span class="message">{event.message}</span>
            </span>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</div>

<style>
  .claude {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    height: 100%;
    min-height: 0;
    font-size: 0.8rem;
  }

  .summary {
    display: flex;
    flex: 1;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.1rem;
    min-width: 0;
    color: var(--color-success);
    text-align: center;
  }

  .summary.attention {
    color: var(--tile-color);
  }

  .summary strong {
    font-size: 2rem;
    line-height: 1;
  }

  .summary .project {
    max-width: 100%;
    overflow: hidden;
    color: var(--color-text-muted);
    font-size: 0.7rem;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    color: var(--color-text-muted);
  }

  .status strong {
    font-size: 1.1rem;
  }

  .attention {
    color: var(--tile-color);
  }

  ul {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 0.1rem;
    min-height: 0;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
  }

  li {
    display: flex;
    gap: 0.4rem;
    padding: 0.25rem 0.35rem;
    border-radius: 0.375rem;
    opacity: 0.7;
  }

  li.active {
    background: color-mix(in srgb, var(--tile-color) 12%, transparent);
    opacity: 1;
  }

  li[data-kind="done"] {
    opacity: 1;
  }

  .icon {
    display: inline-flex;
    flex: none;
    padding-top: 0.1rem;
    color: var(--color-success);
  }

  [data-kind="waiting"] .icon {
    color: var(--tile-color);
  }

  .text {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
  }

  .line {
    display: flex;
    gap: 0.5rem;
  }

  .project {
    flex: 1;
    font-weight: 600;
  }

  .time {
    flex: none;
    color: var(--color-text-muted);
    font-size: 0.7rem;
  }

  .project,
  .message {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .message {
    color: var(--color-text-muted);
  }

  .icon-button {
    display: inline-flex;
    padding: 0.2rem;
    border: none;
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
  }

  .icon-button:hover {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .hint {
    margin: 0;
    color: var(--color-text-muted);
  }

  .error {
    margin: 0;
    color: var(--color-danger-text);
  }
</style>
