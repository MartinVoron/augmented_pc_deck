<!--
  Tuile des minuteurs : plusieurs comptes à rebours nommés, à démarrer, mettre en pause
  et remettre à zéro. Le temps s'écoule côté Rust ; la tuile ne fait que l'afficher.
-->
<script lang="ts">
  import { Pause, Play, Plus, RotateCcw, Trash2 } from "@lucide/svelte";
  import { onMount } from "svelte";
  import {
    formatDuration,
    listRuns,
    onTimersChanged,
    pauseTimer,
    remainingMs,
    resetTimer,
    startTimer,
    timerKey,
    type TimerRuns,
    type TimerSettings,
    type TimerStatus,
  } from ".";
  import type { TileConfig } from "../../grid/defaultLayout";
  import NewTimerDialog from "./NewTimerDialog.svelte";
  import { newTimerId, updateTimers } from "./state";

  let { tile }: { tile: TileConfig } = $props();

  let timers = $derived(tile.timers ?? []);
  let runs = $state<TimerRuns>({});
  let now = $state(Date.now());
  let error = $state("");
  let adding = $state(false);

  let anyRunning = $derived(Object.values(runs).some((run) => run.status === "running"));

  onMount(() => {
    void listRuns()
      .then((list) => (runs = list))
      .catch((e: Error) => (error = e.message));
    return onTimersChanged((list) => (runs = list));
  });

  // L'affichage n'avance que lorsqu'un minuteur tourne.
  $effect(() => {
    if (!anyRunning) return;
    now = Date.now();
    const interval = setInterval(() => (now = Date.now()), 250);
    return () => clearInterval(interval);
  });

  function statusOf(key: string): TimerStatus | "idle" {
    return runs[key]?.status ?? "idle";
  }

  async function run(action: () => Promise<unknown>) {
    try {
      await action();
      error = "";
    } catch (e) {
      error = (e as Error).message;
    }
  }

  function toggle(timer: TimerSettings) {
    const key = timerKey(tile.id, timer.id);
    return run(() =>
      runs[key]?.status === "running"
        ? pauseTimer(key)
        : runs[key]?.status === "finished"
          ? resetTimer(key).then(() => startTimer(key, timer.name, timer.durationSecs * 1000))
          : startTimer(key, timer.name, timer.durationSecs * 1000),
    );
  }

  function remove(timer: TimerSettings) {
    return run(async () => {
      await resetTimer(timerKey(tile.id, timer.id));
      await updateTimers(tile.id, (list) => list.filter((t) => t.id !== timer.id));
    });
  }

  function add(name: string, durationSecs: number) {
    adding = false;
    void updateTimers(tile.id, (list) => [...list, { id: newTimerId(list), name, durationSecs }]);
  }
</script>

<ul>
  {#each timers as timer (timer.id)}
    {@const key = timerKey(tile.id, timer.id)}
    {@const status = statusOf(key)}
    <li data-status={status}>
      <span class="text">
        <span class="name">{timer.name}</span>
        <span class="time" role="timer" aria-label="Temps restant de {timer.name}">
          {status === "finished"
            ? "Terminé"
            : formatDuration(remainingMs(runs[key], timer.durationSecs * 1000, now))}
        </span>
      </span>
      <span class="buttons">
        <button
          type="button"
          aria-label="{status === 'running' ? 'Mettre en pause' : 'Démarrer'} {timer.name}"
          onclick={() => toggle(timer)}
        >
          {#if status === "running"}<Pause size={14} />{:else}<Play size={14} />{/if}
        </button>
        {#if status !== "idle"}
          <button
            type="button"
            aria-label="Remettre à zéro {timer.name}"
            onclick={() => run(() => resetTimer(key))}
          >
            <RotateCcw size={14} />
          </button>
        {:else}
          <button
            type="button"
            class="delete"
            aria-label="Supprimer {timer.name}"
            onclick={() => remove(timer)}
          >
            <Trash2 size={14} />
          </button>
        {/if}
      </span>
    </li>
  {/each}
</ul>

{#if timers.length === 0}
  <p class="empty">Aucun minuteur.</p>
{/if}

<button type="button" class="add" onclick={() => (adding = true)}>
  <Plus size={14} /> Minuteur
</button>

{#if error}
  <p class="error" role="alert">{error}</p>
{/if}

{#if adding}
  <NewTimerDialog onsave={add} onclose={() => (adding = false)} />
{/if}

<style>
  ul {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.4rem;
    padding: 0.2rem 0.4rem;
    border-radius: 0.5rem;
    background: var(--color-bg);
  }

  li[data-status="finished"] {
    outline: 1px solid var(--color-warning);
    animation: blink 1s ease-in-out infinite alternate;
  }

  @keyframes blink {
    to {
      background: color-mix(in srgb, var(--color-warning) 18%, var(--color-bg));
    }
  }

  .text {
    display: flex;
    flex-direction: column;
    min-width: 0;
    line-height: 1.2;
  }

  .name {
    overflow: hidden;
    font-size: 0.72rem;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .time {
    color: var(--color-text);
    font-size: 1.05rem;
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }

  li[data-status="running"] .time {
    color: var(--color-warning);
  }

  li[data-status="paused"] .time {
    opacity: 0.6;
  }

  .buttons {
    display: flex;
    flex: none;
    gap: 0.15rem;
  }

  button {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-text-muted);
    font: inherit;
    cursor: pointer;
  }

  button:hover {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .delete:hover {
    color: var(--color-danger);
  }

  .add {
    margin-top: 0.35rem;
    padding: 0.15rem 0.45rem;
    font-size: 0.75rem;
  }

  p {
    margin: 0;
  }

  .error {
    margin-top: 0.3rem;
    color: var(--color-danger-text);
    font-size: 0.75rem;
  }
</style>
