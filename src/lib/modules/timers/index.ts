// Minuteurs : comptes à rebours nommés (voir src-tauri/src/modules/timers.rs).
// Les comptes à rebours tournent côté Rust, pour continuer quand la fenêtre est masquée.
import { onEvent } from "../../events";
import { call } from "../../tauri";

/** Un compte à rebours enregistré dans la configuration de la tuile. */
export interface TimerSettings {
  id: string;
  name: string;
  /** Durée, en secondes. */
  durationSecs: number;
}

export type TimerStatus = "running" | "paused" | "finished";

/** Un compte à rebours lancé (absent tant qu'il n'a pas démarré ou après remise à zéro). */
export interface TimerRun {
  name: string;
  status: TimerStatus;
  durationMs: number;
  /** Temps restant au moment de la pause. */
  remainingMs: number;
  /** Heure de fin prévue (millisecondes depuis 1970), quand il tourne. */
  endsAt: number | null;
}

export type TimerRuns = Record<string, TimerRun>;

/** Clé d'un minuteur côté Rust : unique dans toute l'application. */
export function timerKey(tileId: string, timerId: string): string {
  return `${tileId}:${timerId}`;
}

export function listRuns(): Promise<TimerRuns> {
  return call<TimerRuns>("timers_list");
}

/** Notification de fin personnalisée (sinon « Minuteur terminé »). */
export interface TimerAlert {
  title: string;
  body: string;
}

/** Démarre un minuteur, ou le reprend s'il est en pause. */
export function startTimer(
  key: string,
  name: string,
  durationMs: number,
  alert?: TimerAlert,
): Promise<TimerRun> {
  return call<TimerRun>("timers_start", { key, name, durationMs, alert: alert ?? null });
}

export function pauseTimer(key: string): Promise<void> {
  return call<void>("timers_pause", { key });
}

/** Remet un minuteur à sa durée complète, à l'arrêt. */
export function resetTimer(key: string): Promise<void> {
  return call<void>("timers_reset", { key });
}

/** Appelle `handler` avec tous les minuteurs à chaque changement (fin, pause…). */
export function onTimersChanged(handler: (runs: TimerRuns) => void): () => void {
  return onEvent<TimerRuns>("timers-changed", handler);
}

/** Temps restant d'un minuteur, en millisecondes. */
export function remainingMs(run: TimerRun | undefined, durationMs: number, now: number): number {
  if (!run) return durationMs;
  if (run.status === "running" && run.endsAt !== null) return Math.max(0, run.endsAt - now);
  return run.remainingMs;
}

/** Durée lisible : « 4:05 », « 1:02:03 ». Arrondie à la seconde supérieure. */
export function formatDuration(ms: number): string {
  const total = Math.ceil(Math.max(0, ms) / 1000);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = String(total % 60).padStart(2, "0");
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, "0")}:${seconds}`
    : `${minutes}:${seconds}`;
}
