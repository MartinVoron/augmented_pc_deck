/** Tailles possibles d'une tuile : largeur × hauteur, en cases de la grille. */
export type TileSize = "1x1" | "2x1" | "2x2";

/** Couleur d'accent d'une tuile (voir `src/lib/theme.css`). */
export type TileAccent = "accent" | "success" | "warning" | "danger" | "purple";

/** Nombre de colonnes et de lignes occupées par chaque taille. */
export const TILE_SPANS: Record<TileSize, { columns: number; rows: number }> = {
  "1x1": { columns: 1, rows: 1 },
  "2x1": { columns: 2, rows: 1 },
  "2x2": { columns: 2, rows: 2 },
};
