// Enregistrement des tâches d'une tuile dans la configuration partagée.
import type { Todo } from ".";
import { updateTile } from "../../grid/state";

/** Modifie une tuile de la liste de tâches et l'enregistre. */
export const updateTodosTile = updateTile;

/** Modifie la liste des tâches d'une tuile et l'enregistre. */
export function updateTodos(tileId: string, change: (todos: Todo[]) => Todo[]): Promise<void> {
  return updateTile(tileId, (tile) => (tile.todos = change(tile.todos ?? [])));
}
