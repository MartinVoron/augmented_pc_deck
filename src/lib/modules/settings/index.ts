// Paramètres : lecture et enregistrement de la configuration (voir src-tauri/src/modules/settings).
import { call } from "../../tauri";

/** Une tuile de la grille, telle qu'enregistrée dans la configuration. */
export interface TileSettings {
  id: string;
  kind: string;
  title: string;
  size: "1x1" | "2x1" | "2x2";
}

/** Configuration complète de l'application (fichier `config.json`). */
export interface Config {
  version: number;
  general: {
    notificationsEnabled: boolean;
  };
  layout: {
    tiles: TileSettings[];
  };
}

/** Lit la configuration actuelle. */
export function getSettings(): Promise<Config> {
  return call<Config>("settings_get");
}

/** Enregistre la configuration ; elle s'applique tout de suite. */
export function saveSettings(config: Config): Promise<Config> {
  return call<Config>("settings_save", { config });
}
