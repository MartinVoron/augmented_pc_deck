<!--
  Tuile « Horloge » : l'heure et la date du jour. En 1×1, l'heure et une date courte ;
  en 2×1 et 2×2, l'heure en grand et la date complète. Les secondes sont en option
  (mode édition).
-->
<script lang="ts">
  import { formatDate, formatShortDate, formatTime, nextTickDelay } from ".";
  import type { TileConfig } from "../../grid/defaultLayout";

  let { tile }: { tile: TileConfig } = $props();

  let now = $state(new Date());
  let seconds = $derived(tile.showSeconds ?? false);
  let compact = $derived(tile.size === "1x1");

  // Se recale sur la seconde (ou la minute) suivante pour ne jamais être en retard.
  $effect(() => {
    const withSeconds = seconds;
    let timer: ReturnType<typeof setTimeout>;
    const schedule = () => {
      timer = setTimeout(
        () => {
          now = new Date();
          schedule();
        },
        nextTickDelay(new Date(), withSeconds),
      );
    };
    now = new Date();
    schedule();
    return () => clearTimeout(timer);
  });
</script>

<div class="clock" data-size={tile.size}>
  <time datetime={now.toISOString()} class="time">{formatTime(now, seconds)}</time>
  <span class="date">{compact ? formatShortDate(now) : formatDate(now)}</span>
</div>

<style>
  .clock {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    height: 100%;
    text-align: center;
  }

  .time {
    color: var(--color-text);
    font-size: 2rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    line-height: 1;
    letter-spacing: -0.02em;
  }

  [data-size="2x1"] .time {
    font-size: 3rem;
  }

  [data-size="2x2"] .time {
    font-size: 4.5rem;
  }

  .date {
    color: var(--color-text-muted);
    font-size: 0.8rem;
  }

  [data-size="2x2"] .date {
    font-size: 1.1rem;
  }
</style>
