// Modification d'une tuile dans la configuration partagée.
import type { TileSettings } from "../modules/settings";
import { updateSettings } from "../modules/settings/state.svelte";

/** Modifie une tuile (retrouvée par son identifiant) et l'enregistre. */
export function updateTile(tileId: string, change: (tile: TileSettings) => void): Promise<void> {
  return updateSettings((config) => {
    const tile = config.layout.tiles.find((t) => t.id === tileId);
    if (tile) change(tile);
  });
}
