// Lanceur : ouvrir une application, un fichier, un dossier ou une URL depuis une tuile.
// Voir src-tauri/src/modules/launcher.
import { call } from "../../tauri";

/** Un raccourci de la tuile du lanceur, tel qu'enregistré dans la configuration. */
export interface Shortcut {
  id: string;
  /** Nom affiché sous l'icône. */
  name: string;
  /** Programme, fichier, dossier ou URL à ouvrir. */
  target: string;
  /** Arguments de lancement d'un programme, comme sur une ligne de commande. */
  args?: string;
  /** Image choisie à la place de l'icône extraite de la cible. */
  icon?: string;
}

/** Une cible avec un schéma (`https:`, `mailto:`…) est une URL ; `C:` est un lecteur. */
export function isUrl(target: string): boolean {
  return /^[a-z][a-z0-9+.-]+:/i.test(target.trim());
}

/** Lance la cible d'un raccourci. Rejette une `AppError` claire si elle n'existe plus. */
export function launch(shortcut: Shortcut): Promise<void> {
  return call<void>("launcher_launch", { target: shortcut.target, args: shortcut.args ?? "" });
}

const iconCache = new Map<string, Promise<string | null>>();

/**
 * Icône d'un programme, d'un fichier ou d'un dossier (`data:image/png…`), ou `null`.
 * Le résultat est gardé en mémoire : chaque cible n'est lue qu'une fois.
 */
export function getIcon(target: string): Promise<string | null> {
  let icon = iconCache.get(target);
  if (!icon) {
    icon = call<string | null>("launcher_icon", { target }).catch(() => null);
    iconCache.set(target, icon);
  }
  return icon;
}

/** Oublie les icônes gardées en mémoire (après un changement de cible). */
export function clearIconCache(): void {
  iconCache.clear();
}

/** Lit une image choisie comme icône personnalisée (`data:` URL, 512 Ko au plus). */
export function readImage(path: string): Promise<string> {
  return call<string>("launcher_read_image", { path });
}
