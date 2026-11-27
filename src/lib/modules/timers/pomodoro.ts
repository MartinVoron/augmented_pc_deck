// Mode Pomodoro : alternance de séances de travail et de pauses (fonctions pures).

/** Durées du mode Pomodoro, en minutes (enregistrées dans la tuile). */
export interface PomodoroSettings {
  workMins: number;
  shortBreakMins: number;
  longBreakMins: number;
  /** Nombre de séances de travail avant une longue pause. */
  sessionsBeforeLongBreak: number;
}

export const DEFAULT_POMODORO: PomodoroSettings = {
  workMins: 25,
  shortBreakMins: 5,
  longBreakMins: 15,
  sessionsBeforeLongBreak: 4,
};

export type Phase = "work" | "shortBreak" | "longBreak";

export const PHASE_LABELS: Record<Phase, string> = {
  work: "Travail",
  shortBreak: "Pause",
  longBreak: "Longue pause",
};

/** Où en est le cycle : phase actuelle et nombre de séances de travail terminées. */
export interface PomodoroProgress {
  phase: Phase;
  completedWork: number;
}

export function phaseDurationMs(phase: Phase, settings: PomodoroSettings): number {
  const minutes =
    phase === "work"
      ? settings.workMins
      : phase === "shortBreak"
        ? settings.shortBreakMins
        : settings.longBreakMins;
  return minutes * 60_000;
}

/** Phase suivante : une pause après le travail (longue toutes les N séances), sinon le travail. */
export function nextPhase(
  progress: PomodoroProgress,
  settings: PomodoroSettings,
): PomodoroProgress {
  if (progress.phase !== "work") return { phase: "work", completedWork: progress.completedWork };
  const completedWork = progress.completedWork + 1;
  const every = Math.max(1, settings.sessionsBeforeLongBreak);
  return { phase: completedWork % every === 0 ? "longBreak" : "shortBreak", completedWork };
}

/** Texte de la notification envoyée à la fin d'une phase. */
export function phaseEndAlert(
  progress: PomodoroProgress,
  settings: PomodoroSettings,
): { title: string; body: string } {
  const next = nextPhase(progress, settings);
  const minutes = phaseDurationMs(next.phase, settings) / 60_000;
  return progress.phase === "work"
    ? {
        title: "Séance de travail terminée",
        body: `Place à la ${next.phase === "longBreak" ? "longue pause" : "pause"} (${minutes} min).`,
      }
    : { title: "Pause terminée", body: `On reprend le travail (${minutes} min).` };
}

/** Vérifie et corrige des durées saisies (entre 1 et 180 minutes, 1 à 12 séances). */
export function sanitizePomodoro(
  settings: Partial<PomodoroSettings> | undefined,
): PomodoroSettings {
  const clamp = (value: unknown, fallback: number, max: number) => {
    const n = Math.round(Number(value));
    return Number.isFinite(n) && n >= 1 ? Math.min(n, max) : fallback;
  };
  return {
    workMins: clamp(settings?.workMins, DEFAULT_POMODORO.workMins, 180),
    shortBreakMins: clamp(settings?.shortBreakMins, DEFAULT_POMODORO.shortBreakMins, 180),
    longBreakMins: clamp(settings?.longBreakMins, DEFAULT_POMODORO.longBreakMins, 180),
    sessionsBeforeLongBreak: clamp(
      settings?.sessionsBeforeLongBreak,
      DEFAULT_POMODORO.sessionsBeforeLongBreak,
      12,
    ),
  };
}
