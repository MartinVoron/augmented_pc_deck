// Tâches : liste de choses à faire, enregistrée dans la configuration de la tuile.
// Fonctions pures de modification de la liste.

export interface Todo {
  id: string;
  text: string;
  done: boolean;
}

/** Identifiant libre pour une nouvelle tâche. */
export function newTodoId(todos: Todo[]): string {
  let n = todos.length + 1;
  while (todos.some((t) => t.id === `tache-${n}`)) n++;
  return `tache-${n}`;
}

/** Ajoute une tâche à la fin (un texte vide est ignoré). */
export function addTodo(todos: Todo[], text: string): Todo[] {
  const clean = text.trim();
  return clean ? [...todos, { id: newTodoId(todos), text: clean, done: false }] : todos;
}

/** Change le texte d'une tâche ; un texte vide la supprime. */
export function editTodo(todos: Todo[], id: string, text: string): Todo[] {
  const clean = text.trim();
  if (!clean) return removeTodo(todos, id);
  return todos.map((t) => (t.id === id ? { ...t, text: clean } : t));
}

export function toggleTodo(todos: Todo[], id: string): Todo[] {
  return todos.map((t) => (t.id === id ? { ...t, done: !t.done } : t));
}

export function removeTodo(todos: Todo[], id: string): Todo[] {
  return todos.filter((t) => t.id !== id);
}

/** Supprime toutes les tâches terminées. */
export function clearDone(todos: Todo[]): Todo[] {
  return todos.filter((t) => !t.done);
}

/**
 * Déplace une tâche d'une place vers le haut (-1) ou le bas (+1) parmi les tâches
 * `visible` (les tâches masquées gardent leur place).
 */
export function moveTodo(todos: Todo[], id: string, offset: -1 | 1, visible = todos): Todo[] {
  const shown = visible.findIndex((t) => t.id === id);
  const neighbour = visible[shown + offset];
  if (shown === -1 || !neighbour) return todos;
  const from = todos.findIndex((t) => t.id === id);
  const to = todos.findIndex((t) => t.id === neighbour.id);
  const result = [...todos];
  [result[from], result[to]] = [result[to], result[from]];
  return result;
}
