<!--
  Tuile « Musique » : titre, artiste, pochette et progression du morceau en cours,
  quelle que soit l'application qui joue. Interroge Windows toutes les secondes
  (en pause quand la fenêtre est masquée) ; la barre avance entre deux mesures.
-->
<script lang="ts">
  import { Music } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { currentPosition, formatTime, getNowPlaying, type NowPlaying } from ".";
  import type { TileConfig } from "../../grid/defaultLayout";
  import { poll } from "../../polling";

  let { tile }: { tile: TileConfig } = $props();

  let track = $state<NowPlaying | null>(null);
  let loaded = $state(false);
  let error = $state("");
  let now = $state(Date.now());

  onMount(() =>
    poll(async () => {
      try {
        track = await getNowPlaying();
        error = "";
      } catch (e) {
        error = (e as Error).message;
      } finally {
        loaded = true;
        now = Date.now();
      }
    }, 1000),
  );

  // Fait avancer la barre en douceur pendant la lecture.
  $effect(() => {
    if (track?.status !== "playing") return;
    const interval = setInterval(() => (now = Date.now()), 250);
    return () => clearInterval(interval);
  });

  let position = $derived(track ? currentPosition(track, now) : null);
  let progress = $derived(
    track?.durationMs && position !== null ? (position / track.durationMs) * 100 : 0,
  );
</script>

{#if track}
  <div class="music" data-size={tile.size} data-status={track.status}>
    <div class="cover">
      {#if track.thumbnail}
        <img src={track.thumbnail} alt="Pochette de {track.album || track.title}" />
      {:else}
        <Music size={28} />
      {/if}
    </div>
    <div class="info">
      <p class="title" title={track.title}>{track.title}</p>
      <p class="artist" title={track.artist}>{track.artist || "Artiste inconnu"}</p>
      <p class="source">
        {track.source}{#if track.status !== "playing"}
          · {track.status === "paused" ? "en pause" : "arrêté"}{/if}
      </p>
    </div>
    {#if track.durationMs !== null && position !== null}
      <div class="progress">
        <span
          class="bar"
          role="progressbar"
          aria-label="Progression du morceau"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={Math.round(progress)}
          aria-valuetext="{formatTime(position)} sur {formatTime(track.durationMs)}"
        >
          <span class="fill" style:width="{progress}%"></span>
        </span>
        <span class="times">
          <span>{formatTime(position)}</span>
          <span>{formatTime(track.durationMs)}</span>
        </span>
      </div>
    {/if}
  </div>
{:else if error}
  <p class="error" role="alert">{error}</p>
{:else if loaded}
  <div class="empty">
    <Music size={24} />
    <p>Rien ne joue en ce moment.</p>
  </div>
{/if}

<style>
  .music {
    display: grid;
    grid-template-areas:
      "cover info"
      "progress progress";
    grid-template-columns: auto minmax(0, 1fr);
    align-items: center;
    gap: 0.35rem 0.6rem;
  }

  .music[data-size="2x2"] {
    grid-template-areas:
      "cover"
      "info"
      "progress";
    grid-template-columns: minmax(0, 1fr);
    justify-items: center;
    text-align: center;
  }

  .cover {
    grid-area: cover;
    display: grid;
    place-items: center;
    width: 3.25rem;
    aspect-ratio: 1;
    overflow: hidden;
    border-radius: 0.5rem;
    background: var(--color-bg);
    color: var(--tile-color);
  }

  .music[data-size="2x2"] .cover {
    width: min(9rem, 100%);
    box-shadow: 0 6px 20px rgb(0 0 0 / 35%);
  }

  .music[data-size="1x1"] .cover {
    display: none;
  }

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .info {
    grid-area: info;
    min-width: 0;
    max-width: 100%;
  }

  p {
    margin: 0;
  }

  .title,
  .artist {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .title {
    color: var(--color-text);
    font-weight: 600;
  }

  .artist {
    font-size: 0.8rem;
  }

  .source {
    font-size: 0.7rem;
    opacity: 0.8;
  }

  .progress {
    grid-area: progress;
    width: 100%;
  }

  .bar {
    display: block;
    height: 0.3rem;
    overflow: hidden;
    border-radius: 999px;
    background: var(--color-bg);
  }

  .fill {
    display: block;
    height: 100%;
    background: var(--tile-color);
    transition: width 0.25s linear;
  }

  .times {
    display: flex;
    justify-content: space-between;
    font-size: 0.68rem;
    font-variant-numeric: tabular-nums;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.4rem;
    height: 100%;
    opacity: 0.8;
    text-align: center;
  }

  .error {
    color: #f1c4c4;
  }
</style>
