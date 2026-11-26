<!--
  Tuile des minuteurs, en trois onglets : comptes à rebours, chronomètre et Pomodoro.
  Les trois vues restent montées pour continuer à suivre leurs minuteurs.
-->
<script lang="ts">
  import type { TileConfig } from "../../grid/defaultLayout";
  import CountdownList from "./CountdownList.svelte";
  import PomodoroView from "./PomodoroView.svelte";
  import StopwatchView from "./StopwatchView.svelte";
  import { selectedTab, selectTab, type TimersTab } from "./tabs.svelte";

  let { tile }: { tile: TileConfig } = $props();

  const TABS: { id: TimersTab; label: string }[] = [
    { id: "countdowns", label: "Minuteurs" },
    { id: "stopwatch", label: "Chrono" },
    { id: "pomodoro", label: "Pomodoro" },
  ];

  let tab = $derived(selectedTab(tile.id));
</script>

<div class="tabs" role="tablist" aria-label="Mode des minuteurs">
  {#each TABS as item (item.id)}
    <button
      type="button"
      role="tab"
      id="{tile.id}-tab-{item.id}"
      aria-selected={tab === item.id}
      aria-controls="{tile.id}-panel-{item.id}"
      onclick={() => selectTab(tile.id, item.id)}
    >
      {item.label}
    </button>
  {/each}
</div>

<div
  role="tabpanel"
  id="{tile.id}-panel-countdowns"
  aria-labelledby="{tile.id}-tab-countdowns"
  hidden={tab !== "countdowns"}
>
  <CountdownList {tile} />
</div>
<div
  role="tabpanel"
  id="{tile.id}-panel-stopwatch"
  aria-labelledby="{tile.id}-tab-stopwatch"
  hidden={tab !== "stopwatch"}
>
  <StopwatchView tileId={tile.id} />
</div>
<div
  role="tabpanel"
  id="{tile.id}-panel-pomodoro"
  aria-labelledby="{tile.id}-tab-pomodoro"
  hidden={tab !== "pomodoro"}
>
  <PomodoroView {tile} />
</div>

<style>
  .tabs {
    display: flex;
    gap: 0.15rem;
    margin-bottom: 0.4rem;
    padding: 0.15rem;
    border-radius: 0.5rem;
    background: var(--color-bg);
  }

  .tabs button {
    flex: 1;
    min-width: 0;
    padding: 0.1rem 0.2rem;
    overflow: hidden;
    border: 0;
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-text-muted);
    font: inherit;
    font-size: 0.7rem;
    text-overflow: ellipsis;
    white-space: nowrap;
    cursor: pointer;
  }

  .tabs button[aria-selected="true"] {
    background: var(--color-surface-hover);
    color: var(--color-warning);
  }
</style>
