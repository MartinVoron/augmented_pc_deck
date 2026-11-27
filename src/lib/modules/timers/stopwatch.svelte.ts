// Chronomètre avec tours. Il tourne dans l'interface : le temps est calculé à partir
// de l'heure de départ, donc il reste juste même si l'affichage est ralenti.

export class Stopwatch {
  running = $state(false);
  /** Temps écoulé avant le dernier départ, en millisecondes. */
  accumulatedMs = $state(0);
  /** Temps total au moment de chaque tour. */
  laps = $state<number[]>([]);
  private startedAt = 0;

  elapsed(now: number): number {
    return this.accumulatedMs + (this.running ? now - this.startedAt : 0);
  }

  start(now: number): void {
    if (this.running) return;
    this.startedAt = now;
    this.running = true;
  }

  pause(now: number): void {
    if (!this.running) return;
    this.accumulatedMs = this.elapsed(now);
    this.running = false;
  }

  lap(now: number): void {
    if (this.running) this.laps = [...this.laps, this.elapsed(now)];
  }

  reset(): void {
    this.running = false;
    this.accumulatedMs = 0;
    this.laps = [];
  }
}

/** Durée d'un tour : écart avec le tour précédent. */
export function lapDuration(laps: number[], index: number): number {
  return laps[index] - (index > 0 ? laps[index - 1] : 0);
}

/** Temps lisible au dixième : « 0:07,3 », « 12:04,0 », « 1:02:03,4 ». */
export function formatStopwatch(ms: number): string {
  const tenths = Math.floor(Math.max(0, ms) / 100);
  const total = Math.floor(tenths / 10);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = String(total % 60).padStart(2, "0");
  const base = hours > 0 ? `${hours}:${String(minutes).padStart(2, "0")}` : `${minutes}`;
  return `${base}:${seconds},${tenths % 10}`;
}

/** Un chronomètre par tuile, gardé quand on change d'onglet ou de mode. */
// Simple registre : seuls les chronomètres eux-mêmes ont besoin d'être réactifs.
// eslint-disable-next-line svelte/prefer-svelte-reactivity
const stopwatches = new Map<string, Stopwatch>();

export function stopwatchFor(tileId: string): Stopwatch {
  let stopwatch = stopwatches.get(tileId);
  if (!stopwatch) {
    stopwatch = new Stopwatch();
    stopwatches.set(tileId, stopwatch);
  }
  return stopwatch;
}
