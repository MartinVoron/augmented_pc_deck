<!--
  Tuile « Météo » : température, temps et ville. En 2×1 : minimum, maximum et
  ressenti du jour ; en 2×2 : les prochains jours en plus. Rafraîchie toutes les
  30 minutes (en pause quand la fenêtre est masquée). La ville se choisit en mode
  édition (Caen par défaut).
-->
<script lang="ts">
  import { RefreshCw } from "@lucide/svelte";
  import { untrack } from "svelte";
  import {
    dayLabel,
    DEFAULT_PLACE,
    describe,
    formatTemperature,
    getForecast,
    REFRESH_MS,
    type Forecast,
  } from ".";
  import type { TileConfig } from "../../grid/defaultLayout";
  import { poll } from "../../polling";
  import WeatherIcon from "./WeatherIcon.svelte";

  let { tile }: { tile: TileConfig } = $props();

  let forecast = $state<Forecast | null>(null);
  let error = $state("");
  let loading = $state(false);

  let place = $derived(tile.location ?? DEFAULT_PLACE);
  let today = $derived(forecast?.days[0]);
  let nextDays = $derived(forecast?.days.slice(1) ?? []);

  async function refresh() {
    loading = true;
    try {
      forecast = await getForecast(place);
      error = "";
    } catch (e) {
      error = (e as Error).message;
    } finally {
      loading = false;
    }
  }

  // Recharge tout de suite quand la ville change.
  $effect(() => {
    void place.latitude;
    void place.longitude;
    return untrack(() => poll(refresh, REFRESH_MS));
  });
</script>

<div class="weather" data-size={tile.size}>
  {#if !forecast && error}
    <p class="error" role="alert">{error}</p>
  {:else if !forecast}
    <p class="hint">Chargement…</p>
  {:else}
    <div class="now">
      <WeatherIcon
        code={forecast.current.code}
        isDay={forecast.current.isDay}
        size={tile.size === "1x1" ? 28 : 44}
      />
      <span class="temperature">{formatTemperature(forecast.current.temperature)}</span>
      {#if tile.size !== "1x1"}
        <span class="summary">
          <span class="label">{describe(forecast.current.code).label}</span>
          {#if today}
            <span class="muted">
              {formatTemperature(today.min)} / {formatTemperature(today.max)} · ressenti
              {formatTemperature(forecast.current.feelsLike)}
            </span>
          {/if}
        </span>
      {/if}
    </div>

    {#if tile.size === "2x2" && nextDays.length > 0}
      <ul class="days" aria-label="Prochains jours">
        {#each nextDays as day, index (day.date)}
          <li>
            <span class="day">{dayLabel(day.date, index + 1)}</span>
            <WeatherIcon code={day.code} size={18} />
            <span class="muted">{describe(day.code).label}</span>
            <span class="range">
              {formatTemperature(day.min)} / {formatTemperature(day.max)}
            </span>
          </li>
        {/each}
      </ul>
    {/if}

    <footer>
      <span class="place" title={place.detail}>{place.name}</span>
      {#if tile.size !== "1x1"}
        <button
          type="button"
          class="icon-button"
          class:spinning={loading}
          aria-label="Actualiser la météo"
          disabled={loading}
          onclick={() => void refresh()}
        >
          <RefreshCw size={13} />
        </button>
      {/if}
    </footer>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
  {/if}
</div>

<style>
  .weather {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    height: 100%;
    min-height: 0;
    font-size: 0.8rem;
  }

  .now {
    display: flex;
    flex: 1;
    align-items: center;
    gap: 0.6rem;
    min-height: 0;
  }

  [data-size="1x1"] .now {
    flex-direction: column;
    justify-content: center;
    gap: 0.2rem;
  }

  [data-size="2x2"] .now {
    flex: none;
  }

  .temperature {
    font-size: 2rem;
    font-weight: 600;
    line-height: 1;
  }

  [data-size="2x1"] .temperature,
  [data-size="2x2"] .temperature {
    font-size: 2.6rem;
  }

  .summary {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }

  .label {
    font-weight: 600;
  }

  .muted {
    color: var(--color-text-muted);
  }

  .days {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 0.2rem;
    min-height: 0;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
  }

  .days li {
    display: grid;
    grid-template-columns: 2.6rem auto 1fr auto;
    align-items: center;
    gap: 0.5rem;
  }

  .day {
    font-weight: 600;
    text-transform: capitalize;
  }

  .range {
    font-variant-numeric: tabular-nums;
  }

  .days .muted {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    color: var(--color-text-muted);
  }

  [data-size="1x1"] footer {
    justify-content: center;
  }

  .place {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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

  .icon-button:hover:not(:disabled) {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  .spinning :global(svg) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .hint {
    margin: 0;
    color: var(--color-text-muted);
  }

  .error {
    margin: 0;
    color: #f1c4c4;
  }
</style>
