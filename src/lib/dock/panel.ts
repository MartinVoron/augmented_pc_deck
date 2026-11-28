// Taille du panneau ouvert d'une fenêtre ancrée, d'après les tuiles de la page.
import { TILE_SPANS, type TileSize } from "../grid/types";
import type { DockEdge } from ".";

/** Côté d'une case de la grille et écart entre les cases (voir `theme.css`). */
export const TILE_UNIT = 140;
export const TILE_GAP = 12;
/** Hauteur de la barre du panneau (pages, bouton Détacher). */
export const PANEL_BAR = 44;

/** Disposition du panneau : grille de `columns` × `rows` cases, taille en pixels. */
export interface PanelLayout {
  columns: number;
  rows: number;
  width: number;
  height: number;
}

/**
 * Lignes occupées par les tuiles dans une grille de `columns` colonnes, rangées
 * comme le fait le navigateur (`grid-auto-flow: dense`) : chaque tuile prend le
 * premier trou où elle tient, en partant du haut.
 */
export function packedRows(sizes: TileSize[], columns: number): number {
  const used: boolean[][] = [];
  const free = (row: number, column: number) => !used[row]?.[column];
  let rows = 0;
  for (const size of sizes) {
    const span = TILE_SPANS[size];
    const width = Math.min(span.columns, columns);
    for (let row = 0; ; row++) {
      let placed = false;
      for (let column = 0; column + width <= columns; column++) {
        let fits = true;
        for (let r = row; r < row + span.rows && fits; r++) {
          for (let c = column; c < column + width && fits; c++) fits = free(r, c);
        }
        if (!fits) continue;
        for (let r = row; r < row + span.rows; r++) {
          used[r] ??= [];
          for (let c = column; c < column + width; c++) used[r][c] = true;
        }
        rows = Math.max(rows, row + span.rows);
        placed = true;
        break;
      }
      if (placed) break;
    }
  }
  return rows;
}

function cells(count: number): number {
  return count * TILE_UNIT + Math.max(0, count - 1) * TILE_GAP + 2 * TILE_GAP;
}

/**
 * Disposition du panneau d'une fenêtre ancrée au bord `edge` :
 * - en haut ou en bas, une bande de deux lignes, aussi large que nécessaire
 *   (au plus `maxWidth` pixels : au-delà, des lignes s'ajoutent) ;
 * - à gauche ou à droite, une colonne de deux cases de large.
 * Une page vide garde la place d'un message (2 × 1 cases).
 */
export function panelLayout(
  sizes: TileSize[],
  edge: DockEdge,
  maxWidth = Number.POSITIVE_INFINITY,
): PanelLayout {
  let columns = 2;
  if (edge === "top" || edge === "bottom") {
    const area = sizes.reduce(
      (sum, size) => sum + TILE_SPANS[size].columns * TILE_SPANS[size].rows,
      0,
    );
    const fitting = Math.floor((maxWidth - TILE_GAP) / (TILE_UNIT + TILE_GAP));
    const widest = Math.max(2, fitting);
    columns = Math.max(2, Math.min(Math.ceil(area / 2), widest));
    // Des trous peuvent pousser une tuile sur une troisième ligne : on élargit.
    while (columns < widest && packedRows(sizes, columns) > 2) columns++;
  }
  const rows = Math.max(1, packedRows(sizes, columns));
  return {
    columns,
    rows,
    width: cells(columns),
    height: PANEL_BAR + cells(rows),
  };
}
