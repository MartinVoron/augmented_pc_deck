// Modifications de la liste des raccourcis d'une tuile du lanceur (fonctions pures).
import { isUrl, type Shortcut } from ".";

/** Nom proposé pour une cible : nom du fichier sans extension, ou nom du site. */
export function defaultName(target: string): string {
  const text = target.trim().replace(/^"|"$/g, "");
  if (isUrl(text)) {
    try {
      const url = new URL(text);
      if (url.hostname) return url.hostname.replace(/^www\./, "");
    } catch {
      // Schéma sans adresse (mailto:, spotify:…) : on garde le texte.
    }
    return text;
  }
  const file =
    text
      .replace(/[\\/]+$/, "")
      .split(/[\\/]/)
      .pop() ?? text;
  const dot = file.lastIndexOf(".");
  return dot > 0 ? file.slice(0, dot) : file || text;
}

/** Identifiant libre pour un nouveau raccourci. */
export function newShortcutId(shortcuts: Shortcut[]): string {
  let n = shortcuts.length + 1;
  while (shortcuts.some((s) => s.id === `raccourci-${n}`)) n++;
  return `raccourci-${n}`;
}

/** Raccourci créé à partir d'une cible (fichier déposé, choisi ou URL). */
export function shortcutFromTarget(shortcuts: Shortcut[], target: string): Shortcut {
  return { id: newShortcutId(shortcuts), name: defaultName(target), target: target.trim() };
}

/** Ajoute un raccourci, ou remplace celui qui a le même identifiant. */
export function saveShortcut(shortcuts: Shortcut[], shortcut: Shortcut): Shortcut[] {
  const clean: Shortcut = {
    ...shortcut,
    name: shortcut.name.trim(),
    target: shortcut.target.trim(),
  };
  if (!clean.args?.trim()) delete clean.args;
  if (!clean.icon) delete clean.icon;
  return shortcuts.some((s) => s.id === clean.id)
    ? shortcuts.map((s) => (s.id === clean.id ? clean : s))
    : [...shortcuts, clean];
}

export function removeShortcut(shortcuts: Shortcut[], id: string): Shortcut[] {
  return shortcuts.filter((s) => s.id !== id);
}

/** Déplace un raccourci d'une place vers l'avant (-1) ou l'arrière (+1). */
export function moveShortcut(shortcuts: Shortcut[], id: string, offset: -1 | 1): Shortcut[] {
  const from = shortcuts.findIndex((s) => s.id === id);
  const to = from + offset;
  if (from === -1 || to < 0 || to >= shortcuts.length) return shortcuts;
  const result = [...shortcuts];
  [result[from], result[to]] = [result[to], result[from]];
  return result;
}
