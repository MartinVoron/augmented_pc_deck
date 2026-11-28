<!--
  Contenu de l'encoche repliée, comme boring.notch : la pochette du morceau en cours
  et un petit égaliseur animé, ou l'icône et le temps restant du minuteur qui tourne.
  Sur les bords gauche et droit (encoche debout), seuls les symboles s'affichent.
-->
<script lang="ts">
  import { Music, Timer } from "@lucide/svelte";
  import type { Activity } from "../dock/activity.svelte";
  import { formatDuration, remainingMs } from "../modules/timers";

  let { activity, vertical = false }: { activity: NonNullable<Activity>; vertical?: boolean } =
    $props();

  let now = $state(Date.now());
  $effect(() => {
    if (activity.kind !== "timer") return;
    const id = setInterval(() => (now = Date.now()), 500);
    return () => clearInterval(id);
  });
</script>

<div class="activity" class:vertical data-kind={activity.kind}>
  {#if activity.kind === "music"}
    {#if activity.track.thumbnail}
      <img class="art" src={activity.track.thumbnail} alt="" width="20" height="20" />
    {:else}
      <span class="art placeholder"><Music size={12} /></span>
    {/if}
    {#if !vertical}
      <span class="label" title="{activity.track.title} · {activity.track.artist}">
        {activity.track.title}
      </span>
    {/if}
    <span class="bars" role="img" aria-label="Lecture en cours : {activity.track.title}">
      <i></i><i></i><i></i><i></i>
    </span>
  {:else}
    <span class="timer-icon"><Timer size={15} /></span>
    {#if !vertical}
      <span class="label">{activity.run.name}</span>
    {/if}
    <span class="time" role="timer" aria-label="{activity.run.name} : temps restant">
      {formatDuration(remainingMs(activity.run, activity.run.durationMs, now))}
    </span>
  {/if}
</div>

<style>
  .activity {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    height: 100%;
    padding: 0 12px;
    color: #f5f5f7;
    font-size: 0.75rem;
  }

  .activity.vertical {
    flex-direction: column;
    justify-content: space-between;
    padding: 12px 0;
  }

  .art {
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    border-radius: 5px;
    object-fit: cover;
  }

  .placeholder {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: #2c2c2e;
    color: #a1a1a6;
  }

  .label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    color: #a1a1a6;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  /* Égaliseur : quatre barres qui montent et descendent, décalées. */
  .bars {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
    height: 14px;
  }

  .bars i {
    width: 3px;
    height: 100%;
    border-radius: 2px;
    background: #30d158;
    transform-origin: center;
    animation: bounce 900ms ease-in-out infinite;
  }

  .bars i:nth-child(2) {
    animation-delay: -300ms;
  }

  .bars i:nth-child(3) {
    animation-delay: -600ms;
  }

  .bars i:nth-child(4) {
    animation-delay: -150ms;
  }

  @keyframes bounce {
    0%,
    100% {
      transform: scaleY(0.3);
    }

    50% {
      transform: scaleY(1);
    }
  }

  .timer-icon {
    display: inline-flex;
    color: #ff9f0a;
  }

  .time {
    flex-shrink: 0;
    color: #ff9f0a;
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }

  .vertical .time {
    font-size: 0.6rem;
  }

  @media (prefers-reduced-motion: reduce) {
    .bars i {
      animation: none;
      transform: scaleY(0.6);
    }
  }
</style>
