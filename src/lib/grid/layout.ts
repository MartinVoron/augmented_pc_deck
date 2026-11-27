import { DEFAULT_PAGES, type PageConfig, type TileConfig, type TileKind } from "./defaultLayout";
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

type RawTile = { id: string; kind: string; title: string; size: string };

/**
 * Garde seulement les tuiles valides d'une liste lue dans la configuration
 * (un module inconnu ou une taille inconnue sont ignorés au lieu de tout casser).
 */
export function sanitizeTiles(tiles: RawTile[]): TileConfig[] {
  return tiles.flatMap((tile) =>
    isTileKind(tile.kind) && isTileSize(tile.size)
      ? [{ ...tile, kind: tile.kind, size: tile.size }]
      : [],
  );
}

/** Pages lues dans la configuration ; la disposition par défaut tant qu'elle n'est pas chargée. */
export function sanitizePages(
  pages: { id: string; name: string; tiles?: RawTile[] }[] | undefined,
): PageConfig[] {
  if (!pages || pages.length === 0) return DEFAULT_PAGES;
  return pages.map((page) => ({ ...page, tiles: sanitizeTiles(page.tiles ?? []) }));
}

/** Toutes les tuiles de toutes les pages. */
export function allTiles(pages: { tiles: { id: string }[] }[]): { id: string }[] {
  return pages.flatMap((page) => page.tiles);
}

/** Identifiant libre pour une nouvelle tuile (unique dans toutes les pages). */
export function newTileId(taken: { id: string }[], kind: TileKind): string {
  let n = 1;
  while (taken.some((tile) => tile.id === `${kind}-${n}`)) n++;
  return `${kind}-${n}`;
}

/**
 * Ajoute une tuile 1×1 d'un module à la fin de la grille. `taken` : toutes les tuiles
 * existantes, pour que l'identifiant reste unique d'une page à l'autre.
 */
export function addTile(
  tiles: TileConfig[],
  kind: TileKind,
  taken: { id: string }[] = tiles,
): TileConfig[] {
  return [...tiles, { id: newTileId(taken, kind), kind, title: KIND_TITLES[kind], size: "1x1" }];
}

export function removeTile(tiles: TileConfig[], id: string): TileConfig[] {
  return tiles.filter((tile) => tile.id !== id);
}

export function resizeTile(tiles: TileConfig[], id: string, size: TileSize): TileConfig[] {
  return tiles.map((tile) => (tile.id === id ? { ...tile, size } : tile));
}

/** Déplace un élément à une nouvelle position (bornée aux limites de la liste). */
function move<T extends { id: string }>(list: T[], id: string, toIndex: number): T[] {
  const from = list.findIndex((item) => item.id === id);
  if (from === -1) return list;
  const target = Math.max(0, Math.min(toIndex, list.length - 1));
  const result = [...list];
  const [item] = result.splice(from, 1);
  result.splice(target, 0, item);
  return result;
}

/** Déplace une tuile à une nouvelle position (bornée aux limites de la liste). */
export function moveTile(tiles: TileConfig[], id: string, toIndex: number): TileConfig[] {
  return move(tiles, id, toIndex);
}
