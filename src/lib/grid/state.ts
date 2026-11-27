// Modification d'une tuile dans la configuration partagée.
import type { Config, TileSettings } from "../modules/settings";
import { updateSettings } from "../modules/settings/state.svelte";

/** Retrouve une tuile, quelle que soit sa page. */
export function findTile(
  config: Config | null | undefined,
  tileId: string,
): TileSettings | undefined {
  for (const page of config?.layout.pages ?? []) {
    const tile = page.tiles.find((t) => t.id === tileId);
    if (tile) return tile;
  }
  return undefined;
}

/** Modifie une tuile (retrouvée par son identifiant) et l'enregistre. */
export function updateTile(tileId: string, change: (tile: TileSettings) => void): Promise<void> {
  return updateSettings((config) => {
    const tile = findTile(config, tileId);
    if (tile) change(tile);
  });
}
