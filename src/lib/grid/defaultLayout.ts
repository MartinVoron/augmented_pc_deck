import type { Shortcut } from "../modules/launcher";
import type { TileAccent, TileSize } from "./types";

/** Module affiché par une tuile. */
export type TileKind =
  "launcher" | "processes" | "timers" | "todos" | "music" | "github" | "gmail" | "claude";

export interface TileConfig {
  id: string;
  kind: TileKind;
  title: string;
  size: TileSize;
  /** Raccourcis d'une tuile du lanceur. */
  shortcuts?: Shortcut[];
}

/** Couleur d'accent de chaque module. */
export const KIND_ACCENTS: Record<TileKind, TileAccent> = {
  launcher: "accent",
  processes: "success",
  timers: "warning",
  todos: "accent",
  music: "purple",
  github: "purple",
  gmail: "danger",
  claude: "warning",
};

/** Disposition proposée au premier lancement. */
export const DEFAULT_LAYOUT: TileConfig[] = [
  { id: "launcher-1", kind: "launcher", title: "Raccourcis", size: "2x1" },
  { id: "processes-1", kind: "processes", title: "Processeur et mémoire", size: "2x1" },
  { id: "music-1", kind: "music", title: "Musique", size: "2x2" },
  { id: "timers-1", kind: "timers", title: "Minuteurs", size: "1x1" },
  { id: "todos-1", kind: "todos", title: "Tâches", size: "2x2" },
  { id: "github-1", kind: "github", title: "GitHub", size: "1x1" },
  { id: "gmail-1", kind: "gmail", title: "Gmail", size: "1x1" },
  { id: "claude-1", kind: "claude", title: "Claude", size: "1x1" },
];
