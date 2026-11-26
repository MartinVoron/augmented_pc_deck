// Enregistrement des raccourcis d'une tuile dans la configuration partagée.
import type { Shortcut } from ".";
import { updateSettings } from "../settings/state.svelte";

/** Modifie la liste des raccourcis d'une tuile et l'enregistre. */
export function updateShortcuts(
  tileId: string,
  change: (shortcuts: Shortcut[]) => Shortcut[],
): Promise<void> {
  return updateSettings((config) => {
    const tile = config.layout.tiles.find((t) => t.id === tileId);
    if (tile) tile.shortcuts = change(tile.shortcuts ?? []);
  });
}
