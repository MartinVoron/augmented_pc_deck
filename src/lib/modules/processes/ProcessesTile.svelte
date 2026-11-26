<!--
  Tuile « Processeur et mémoire » : usage du processeur, de la mémoire et des disques,
  rafraîchi toutes les 2 secondes (en pause quand la fenêtre est masquée).
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { formatBytes, getUsage, percent, type Usage } from ".";
  import { poll } from "../../polling";
  import UsageBar from "./UsageBar.svelte";

  /** Intervalle de rafraîchissement, en millisecondes. */
  const REFRESH_MS = 2000;

  let usage = $state<Usage | null>(null);
  let error = $state("");

  onMount(() =>
    poll(async () => {
      try {
        usage = await getUsage();
        error = "";
      } catch (e) {
        error = (e as Error).message;
      }
    }, REFRESH_MS),
  );
</script>

{#if usage}
  <div class="usage">
    <UsageBar label="Processeur" value={usage.cpu} detail="{Math.round(usage.cpu)} %" />
    <UsageBar
      label="Mémoire"
      value={percent(usage.memoryUsed, usage.memoryTotal)}
      detail="{formatBytes(usage.memoryUsed)} / {formatBytes(usage.memoryTotal)}"
    />
    {#each usage.disks as disk (disk.name)}
      <UsageBar
        label="Disque {disk.name}"
        value={percent(disk.used, disk.total)}
        detail="{formatBytes(disk.used)} / {formatBytes(disk.total)}"
      />
    {/each}
  </div>
{:else if error}
  <p class="error" role="alert">{error}</p>
{:else}
  <p class="loading">Mesure en cours…</p>
{/if}

<style>
  .usage {
    display: grid;
    grid-template-columns: auto minmax(2rem, 1fr) auto;
    align-items: center;
    gap: 0.3rem 0.5rem;
    font-size: 0.75rem;
  }

  p {
    margin: 0;
  }

  .error {
    color: #f1c4c4;
  }
</style>
