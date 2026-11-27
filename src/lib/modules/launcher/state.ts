// Enregistrement des raccourcis d'une tuile dans la configuration partagée.
import type { Shortcut } from ".";
import { updateTile } from "../../grid/state";

/** Modifie la liste des raccourcis d'une tuile et l'enregistre. */
export function updateShortcuts(
  tileId: string,
  change: (shortcuts: Shortcut[]) => Shortcut[],
): Promise<void> {
  return updateTile(tileId, (tile) => (tile.shortcuts = change(tile.shortcuts ?? [])));
}
