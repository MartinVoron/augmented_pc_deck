// Avancement du Pomodoro de chaque tuile, gardé tant que l'application tourne.
import type { PomodoroProgress } from "./pomodoro";

const progress = $state<Record<string, PomodoroProgress>>({});

export function pomodoroProgress(tileId: string): PomodoroProgress {
  return progress[tileId] ?? { phase: "work", completedWork: 0 };
}

export function setPomodoroProgress(tileId: string, value: PomodoroProgress): void {
  progress[tileId] = value;
}
