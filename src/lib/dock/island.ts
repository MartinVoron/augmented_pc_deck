// Géométrie de l'encoche dessinée dans la fenêtre ancrée (voir DockedView.svelte).
import { BOUNCE_ROOM, NOTCH_DEPTH, NOTCH_EAR, NOTCH_LENGTH, type DockEdge } from ".";

export interface Size {
  width: number;
  height: number;
}

/** Ressorts de l'animation : l'ouverture rebondit un peu, la fermeture non. */
export const OPEN_SPRING = { stiffness: 0.12, damping: 0.45 };
export const CLOSE_SPRING = { stiffness: 0.35, damping: 0.95 };

function isVertical(edge: DockEdge): boolean {
  return edge === "left" || edge === "right";
}

/**
 * Taille de l'encoche repliée, sans ses oreilles. La fenêtre fait `length` pixels
 * le long du bord : les oreilles prennent `NOTCH_EAR` de chaque côté.
 */
export function foldedSize(edge: DockEdge, length = NOTCH_LENGTH): Size {
  const along = length - 2 * NOTCH_EAR;
  return isVertical(edge)
    ? { width: NOTCH_DEPTH, height: along }
    : { width: along, height: NOTCH_DEPTH };
}

/**
 * Taille de l'encoche ouverte dans une fenêtre de `viewport` : toute la fenêtre,
 * moins les oreilles et la marge du rebond. Jamais plus petite que repliée.
 */
export function openSize(edge: DockEdge, viewport: Size): Size {
  const folded = foldedSize(edge);
  const ears = 2 * NOTCH_EAR;
  const size = isVertical(edge)
    ? { width: viewport.width - BOUNCE_ROOM, height: viewport.height - ears - BOUNCE_ROOM }
    : { width: viewport.width - ears - BOUNCE_ROOM, height: viewport.height - BOUNCE_ROOM };
  return {
    width: Math.max(folded.width, size.width),
    height: Math.max(folded.height, size.height),
  };
}

/** Taille de fenêtre à demander au Rust pour afficher un panneau de `panel`. */
export function windowForPanel(edge: DockEdge, panel: Size): Size {
  const ears = 2 * NOTCH_EAR;
  return isVertical(edge)
    ? { width: panel.width + BOUNCE_ROOM, height: panel.height + ears + BOUNCE_ROOM }
    : { width: panel.width + ears + BOUNCE_ROOM, height: panel.height + BOUNCE_ROOM };
}
