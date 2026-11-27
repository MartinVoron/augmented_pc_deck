// Enregistrement des minuteurs d'une tuile dans la configuration partagée.
import type { TimerSettings } from ".";
import { updateTile } from "../../grid/state";

/** Modifie la liste des minuteurs d'une tuile et l'enregistre. */
export function updateTimers(
  tileId: string,
  change: (timers: TimerSettings[]) => TimerSettings[],
): Promise<void> {
  return updateTile(tileId, (tile) => (tile.timers = change(tile.timers ?? [])));
}

/** Identifiant libre pour un nouveau minuteur. */
export function newTimerId(timers: TimerSettings[]): string {
  let n = timers.length + 1;
  while (timers.some((t) => t.id === `minuteur-${n}`)) n++;
  return `minuteur-${n}`;
}
