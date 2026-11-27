import { DEFAULT_LAYOUT, type TileConfig, type TileKind } from "./defaultLayout";
import { TILE_SPANS, type TileSize } from "./types";

/** Modules qu'on peut placer sur la grille, avec leur nom par défaut. */
export const KIND_TITLES: Record<TileKind, string> = {
  launcher: "Raccourcis",
  processes: "Processeur et mémoire",
  timers: "Minuteurs",
  todos: "Tâches",
  music: "Musique",
  github: "GitHub",
  gmail: "Gmail",
  claude: "Claude",
  clock: "Horloge",
  weather: "Météo",
};

export const TILE_SIZES = Object.keys(TILE_SPANS) as TileSize[];

function isTileKind(kind: string): kind is TileKind {
  return kind in KIND_TITLES;
}

function isTileSize(size: string): size is TileSize {
  return size in TILE_SPANS;
}

/**
 * Garde seulement les tuiles valides d'une disposition lue dans la configuration
 * (un module inconnu ou une taille inconnue sont ignorés au lieu de tout casser).
 */
export function sanitizeLayout(
  tiles: { id: string; kind: string; title: string; size: string }[] | undefined,
): TileConfig[] {
  if (!tiles) return DEFAULT_LAYOUT;
  return tiles.flatMap((tile) =>
    isTileKind(tile.kind) && isTileSize(tile.size)
      ? [{ ...tile, kind: tile.kind, size: tile.size }]
      : [],
  );
}

/** Ajoute une tuile 1×1 d'un module à la fin de la grille. */
export function addTile(tiles: TileConfig[], kind: TileKind): TileConfig[] {
  let n = 1;
  while (tiles.some((tile) => tile.id === `${kind}-${n}`)) n++;
  return [...tiles, { id: `${kind}-${n}`, kind, title: KIND_TITLES[kind], size: "1x1" }];
}

export function removeTile(tiles: TileConfig[], id: string): TileConfig[] {
  return tiles.filter((tile) => tile.id !== id);
}

export function resizeTile(tiles: TileConfig[], id: string, size: TileSize): TileConfig[] {
  return tiles.map((tile) => (tile.id === id ? { ...tile, size } : tile));
}

/** Déplace une tuile à une nouvelle position (bornée aux limites de la liste). */
export function moveTile(tiles: TileConfig[], id: string, toIndex: number): TileConfig[] {
  const from = tiles.findIndex((tile) => tile.id === id);
  if (from === -1) return tiles;
  const target = Math.max(0, Math.min(toIndex, tiles.length - 1));
  const result = [...tiles];
  const [tile] = result.splice(from, 1);
  result.splice(target, 0, tile);
  return result;
}
