// Enregistrement des tâches d'une tuile dans la configuration partagée.
import type { Todo } from ".";
import type { TileSettings } from "../settings";
import { updateSettings } from "../settings/state.svelte";

/** Modifie une tuile de la liste de tâches et l'enregistre. */
export function updateTodosTile(
  tileId: string,
  change: (tile: TileSettings) => void,
): Promise<void> {
  return updateSettings((config) => {
    const tile = config.layout.tiles.find((t) => t.id === tileId);
    if (tile) change(tile);
  });
}

/** Modifie la liste des tâches d'une tuile et l'enregistre. */
export function updateTodos(tileId: string, change: (todos: Todo[]) => Todo[]): Promise<void> {
  return updateTodosTile(tileId, (tile) => (tile.todos = change(tile.todos ?? [])));
}
