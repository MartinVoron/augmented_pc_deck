<!--
  Mode Pomodoro : séances de travail et pauses, avec une longue pause toutes les N séances.
  Le compte à rebours de chaque phase tourne côté Rust (notification à la fin, même
  fenêtre masquée) ; à la fin d'une phase, la suivante est prête à démarrer.
-->
<script lang="ts">
  import { Pause, Play, RotateCcw, Settings2, SkipForward } from "@lucide/svelte";
  import { onMount, untrack } from "svelte";
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
  } from ".";
  import type { TileConfig } from "../../grid/defaultLayout";
  import { updateTile } from "../../grid/state";
  import {
    nextPhase,
    PHASE_LABELS,
    phaseDurationMs,
    phaseEndAlert,
    sanitizePomodoro,
    type PomodoroSettings,
  } from "./pomodoro";
  import { pomodoroProgress, setPomodoroProgress } from "./pomodoroState.svelte";

  let { tile }: { tile: TileConfig } = $props();

  let key = $derived(timerKey(tile.id, "pomodoro"));
  let settings = $derived(sanitizePomodoro(tile.pomodoro));
  let progress = $derived(pomodoroProgress(tile.id));
  let runs = $state<TimerRuns>({});
  let run = $derived(runs[key]);
  let now = $state(Date.now());
  let error = $state("");
  let editing = $state(false);
  let draft = $state<PomodoroSettings>({ ...sanitizePomodoro(undefined) });

  let duration = $derived(phaseDurationMs(progress.phase, settings));
  let sessionInCycle = $derived(progress.completedWork % settings.sessionsBeforeLongBreak);

  onMount(() => {
    void listRuns()
      .then((list) => (runs = list))
      .catch((e: Error) => (error = e.message));
    return onTimersChanged((list) => (runs = list));
  });

  $effect(() => {
    if (run?.status !== "running") return;
    now = Date.now();
    const interval = setInterval(() => (now = Date.now()), 250);
    return () => clearInterval(interval);
  });

  // Une phase terminée (notification déjà envoyée par le Rust) : on prépare la suivante.
  $effect(() => {
    if (run?.status === "finished") untrack(advance);
  });

  async function act(action: () => Promise<unknown>) {
    try {
      await action();
      error = "";
    } catch (e) {
      error = (e as Error).message;
    }
  }

  function advance() {
    setPomodoroProgress(tile.id, nextPhase(progress, settings));
    void act(() => resetTimer(key));
  }

  function toggle() {
    return act(() =>
      run?.status === "running"
        ? pauseTimer(key)
        : startTimer(
            key,
            PHASE_LABELS[progress.phase],
            duration,
            phaseEndAlert(progress, settings),
          ),
    );
  }

  function openSettings() {
    draft = { ...settings };
    editing = true;
  }

  function saveSettings(event: SubmitEvent) {
    event.preventDefault();
    const next = sanitizePomodoro($state.snapshot(draft));
    editing = false;
    void updateTile(tile.id, (target) => (target.pomodoro = next));
  }
</script>

{#if editing}
  <form onsubmit={saveSettings} aria-label="Durées du Pomodoro">
    <label>
      <span>Travail (min)</span><input
        type="number"
        min="1"
        max="180"
        bind:value={draft.workMins}
      />
    </label>
    <label>
      <span>Pause (min)</span><input
        type="number"
        min="1"
        max="180"
        bind:value={draft.shortBreakMins}
      />
    </label>
    <label>
      <span>Longue pause (min)</span><input
        type="number"
        min="1"
        max="180"
        bind:value={draft.longBreakMins}
      />
    </label>
    <label>
      <span>Séances avant la longue pause</span><input
        type="number"
        min="1"
        max="12"
        bind:value={draft.sessionsBeforeLongBreak}
      />
    </label>
    <span class="buttons">
      <button type="button" onclick={() => (editing = false)}>Annuler</button>
      <button type="submit" class="primary">Enregistrer</button>
    </span>
  </form>
{:else}
  <div class="pomodoro" data-phase={progress.phase}>
    <span class="phase">
      {PHASE_LABELS[progress.phase]}
      <span
        class="dots"
        aria-label="Séance {sessionInCycle + 1} sur {settings.sessionsBeforeLongBreak}"
      >
        {#each { length: settings.sessionsBeforeLongBreak }, i (i)}
          <span class="dot" data-done={i < sessionInCycle || undefined}></span>
        {/each}
      </span>
    </span>
    <span class="time" role="timer" aria-label="Temps restant du Pomodoro">
      {formatDuration(remainingMs(run, duration, now))}
    </span>
    <span class="buttons">
      <button
        type="button"
        class="primary"
        aria-label={run?.status === "running" ? "Mettre en pause" : "Démarrer"}
        onclick={toggle}
      >
        {#if run?.status === "running"}<Pause size={14} />{:else}<Play size={14} />{/if}
      </button>
      <button
        type="button"
        aria-label="Recommencer la phase"
        disabled={!run}
        onclick={() => act(() => resetTimer(key))}
      >
        <RotateCcw size={14} />
      </button>
      <button type="button" aria-label="Passer à la phase suivante" onclick={advance}>
        <SkipForward size={14} />
      </button>
      <button type="button" aria-label="Régler les durées" onclick={openSettings}>
        <Settings2 size={14} />
      </button>
    </span>
  </div>
{/if}

{#if error}
  <p class="error" role="alert">{error}</p>
{/if}

<style>
  .pomodoro {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    --phase-color: var(--color-danger);
  }

  .pomodoro[data-phase="shortBreak"],
  .pomodoro[data-phase="longBreak"] {
    --phase-color: var(--color-success);
  }

  .phase {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.4rem;
    color: var(--phase-color);
    font-size: 0.75rem;
    font-weight: 600;
  }

  .dots {
    display: inline-flex;
    gap: 0.2rem;
  }

  .dot {
    width: 0.4rem;
    height: 0.4rem;
    border: 1px solid var(--color-text-muted);
    border-radius: 50%;
  }

  .dot[data-done] {
    border-color: var(--color-danger);
    background: var(--color-danger);
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
    padding: 0.25rem 0.4rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: transparent;
    color: var(--color-text-muted);
    font: inherit;
    font-size: 0.75rem;
    cursor: pointer;
  }

  button:hover:not(:disabled) {
    background: var(--color-surface-hover);
    color: var(--color-text);
  }

  button:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .primary {
    border-color: var(--phase-color, var(--color-warning));
    color: var(--phase-color, var(--color-warning));
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.75rem;
  }

  label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  input {
    width: 3.5rem;
    padding: 0.1rem 0.3rem;
    border: 1px solid var(--color-border);
    border-radius: 0.375rem;
    background: var(--color-bg);
    color: var(--color-text);
    font: inherit;
  }

  .error {
    margin: 0.3rem 0 0;
    color: #f1c4c4;
    font-size: 0.75rem;
  }
</style>
