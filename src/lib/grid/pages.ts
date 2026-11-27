// Pages de tuiles : modèles prédéfinis, ajout, renommage, rangement et suppression.
import type { PageConfig, TileConfig, TileKind } from "./defaultLayout";
import { allTiles, KIND_TITLES, newTileId } from "./layout";
import type { TileSize } from "./types";

/** Modèle de page : un nom et des tuiles déjà placées. */
export interface PageTemplate {
  id: string;
  name: string;
  description: string;
  tiles: { kind: TileKind; size: TileSize }[];
}

/** Modèles proposés pour une nouvelle page. */
export const PAGE_TEMPLATES: PageTemplate[] = [
  {
    id: "travail",
    name: "Travail",
    description: "GitHub, Gmail, Claude, tâches et raccourcis",
    tiles: [
      { kind: "github", size: "2x1" },
      { kind: "gmail", size: "2x1" },
      { kind: "claude", size: "2x1" },
      { kind: "todos", size: "2x2" },
      { kind: "launcher", size: "2x1" },
    ],
  },
  {
    id: "systeme",
    name: "Système",
    description: "Processeur et mémoire, minuteurs et raccourcis",
    tiles: [
      { kind: "processes", size: "2x2" },
      { kind: "timers", size: "2x1" },
      { kind: "launcher", size: "2x1" },
    ],
  },
  {
    id: "detente",
    name: "Détente",
    description: "Musique, météo et horloge",
    tiles: [
      { kind: "music", size: "2x2" },
      { kind: "clock", size: "2x1" },
      { kind: "weather", size: "2x1" },
    ],
  },
  {
    id: "focus",
    name: "Concentration",
    description: "Minuteurs (Pomodoro), tâches et horloge",
    tiles: [
      { kind: "timers", size: "2x2" },
      { kind: "todos", size: "2x2" },
      { kind: "clock", size: "2x1" },
    ],
  },
  { id: "vide", name: "Page vide", description: "Ajoutez vos tuiles une à une", tiles: [] },
];

/** Longueur maximale d'un nom de page (il doit tenir dans un onglet). */
export const MAX_PAGE_NAME = 30;

/** Nom propre : espaces retirés, longueur bornée, « Page » s'il est vide. */
export function cleanPageName(name: string): string {
  return name.trim().slice(0, MAX_PAGE_NAME).trim() || "Page";
}

/** Ajoute une page à la fin, à partir d'un modèle. Renvoie les pages et la nouvelle page. */
export function addPage(
  pages: PageConfig[],
  template: PageTemplate,
  name = template.name,
): { pages: PageConfig[]; page: PageConfig } {
  let n = pages.length + 1;
  while (pages.some((p) => p.id === `page-${n}`)) n++;
  const taken: { id: string }[] = [...allTiles(pages)];
  const tiles: TileConfig[] = template.tiles.map(({ kind, size }) => {
    const tile: TileConfig = { id: newTileId(taken, kind), kind, title: KIND_TITLES[kind], size };
    taken.push(tile);
    return tile;
  });
  const page = { id: `page-${n}`, name: cleanPageName(name), tiles };
  return { pages: [...pages, page], page };
}

export function renamePage(pages: PageConfig[], id: string, name: string): PageConfig[] {
  return pages.map((page) => (page.id === id ? { ...page, name: cleanPageName(name) } : page));
}

/** Supprime une page ; l'écran d'accueil et la dernière page restante ne peuvent pas l'être. */
export function removePage(pages: PageConfig[], id: string): PageConfig[] {
  if (pages.length <= 1 || pages.some((page) => page.id === id && page.home)) return pages;
  return pages.filter((page) => page.id !== id);
}

/** Déplace une page d'un cran vers la gauche (-1) ou la droite (+1). */
export function movePage(pages: PageConfig[], id: string, offset: -1 | 1): PageConfig[] {
  const from = pages.findIndex((page) => page.id === id);
  const to = from + offset;
  if (from === -1 || to < 0 || to >= pages.length) return pages;
  const result = [...pages];
  [result[from], result[to]] = [result[to], result[from]];
  return result;
}
