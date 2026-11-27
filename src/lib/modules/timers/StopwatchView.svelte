<!--
  Chronomètre avec tours. Le dernier tour est affiché en premier.
-->
<script lang="ts">
  import { Flag, Pause, Play, RotateCcw } from "@lucide/svelte";
  import { formatStopwatch, lapDuration, stopwatchFor } from "./stopwatch.svelte";

  let { tileId }: { tileId: string } = $props();

  let stopwatch = $derived(stopwatchFor(tileId));
  let now = $state(Date.now());

  $effect(() => {
    if (!stopwatch.running) return;
    now = Date.now();
    const interval = setInterval(() => (now = Date.now()), 100);
    return () => clearInterval(interval);
  });

  let elapsed = $derived(stopwatch.elapsed(now));
</script>

<div class="stopwatch">
  <span class="time" role="timer" aria-label="Temps du chronomètre">
    {formatStopwatch(elapsed)}
  </span>
  <span class="buttons">
    {#if stopwatch.running}
      <button type="button" onclick={() => stopwatch.pause(Date.now())}>
        <Pause size={14} /> Pause
      </button>
      <button type="button" onclick={() => stopwatch.lap(Date.now())}>
        <Flag size={14} /> Tour
      </button>
    {:else}
      <button type="button" class="primary" onclick={() => stopwatch.start(Date.now())}>
        <Play size={14} />
        {elapsed > 0 ? "Reprendre" : "Démarrer"}
      </button>
      {#if elapsed > 0}
        <button type="button" onclick={() => stopwatch.reset()}>
          <RotateCcw size={14} /> Remettre à zéro
        </button>
      {/if}
    {/if}
  </span>

  {#if stopwatch.laps.length > 0}
    <ol aria-label="Tours">
      {#each stopwatch.laps.map((total, i) => ({ total, i })).reverse() as { total, i } (i)}
        <li>
          <span>Tour {i + 1}</span>
          <span class="lap">{formatStopwatch(lapDuration(stopwatch.laps, i))}</span>
          <span class="total">{formatStopwatch(total)}</span>
        </li>
      {/each}
    </ol>
  {/if}
</div>

<style>
  .stopwatch {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .time {
    color: var(--color-text);
    font-size: 1.5rem;
    font-variant-numeric: tabular-nums;
    font-weight: 600;
    line-height: 1.1;
  }

  .buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  button {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.2rem 0.45rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-text-muted);
    font: inherit;
    font-size: 0.75rem;
    cursor: pointer;
  }

  button:hover {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .primary {
    border-color: var(--color-warning);
    color: var(--color-warning);
  }

  ol {
    display: flex;
    flex-direction: column;
    margin: 0;
    padding: 0;
    list-style: none;
    font-size: 0.75rem;
    font-variant-numeric: tabular-nums;
  }

  li {
    display: grid;
    grid-template-columns: 1fr auto auto;
    gap: 0.6rem;
    padding: 0.1rem 0;
    border-top: 1px solid var(--color-border);
  }

  .lap {
    color: var(--color-text);
  }
</style>
