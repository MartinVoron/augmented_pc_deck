<!--
  Contenu d'une tuile hors du mode édition : le composant du module affiché.
-->
<script lang="ts">
  import type { TileConfig } from "../grid/defaultLayout";
  import LauncherTile from "../modules/launcher/LauncherTile.svelte";
  import MusicTile from "../modules/music/MusicTile.svelte";
  import ProcessesTile from "../modules/processes/ProcessesTile.svelte";
  import TimersTile from "../modules/timers/TimersTile.svelte";
  import TodosTile from "../modules/todos/TodosTile.svelte";

  let { tile }: { tile: TileConfig } = $props();
</script>

{#if tile.kind === "launcher"}
  <LauncherTile {tile} />
{:else if tile.kind === "processes"}
  <ProcessesTile />
{:else if tile.kind === "timers"}
  <TimersTile {tile} />
{:else if tile.kind === "todos"}
  <TodosTile {tile} />
{:else if tile.kind === "music"}
  <MusicTile {tile} />
{:else}
  <p class="soon">Bientôt disponible</p>
{/if}

<style>
  .soon {
    margin: 0;
  }
</style>
