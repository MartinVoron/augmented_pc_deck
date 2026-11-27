// Paramètres : lecture et enregistrement de la configuration (voir src-tauri/src/modules/settings).
import { call } from "../../tauri";
import type { Shortcut } from "../launcher";
import type { TimerSettings } from "../timers";
import type { PomodoroSettings } from "../timers/pomodoro";
import type { Todo } from "../todos";
import type { Place } from "../weather";

/** Une tuile de la grille, telle qu'enregistrée dans la configuration. */
export interface TileSettings {
  id: string;
  kind: string;
  title: string;
  size: "1x1" | "2x1" | "2x2";
  /** Raccourcis d'une tuile du lanceur. */
  shortcuts?: Shortcut[];
  /** Comptes à rebours d'une tuile des minuteurs. */
  timers?: TimerSettings[];
  /** Durées du mode Pomodoro d'une tuile des minuteurs. */
  pomodoro?: PomodoroSettings;
  /** Tâches d'une tuile de la liste de tâches. */
  todos?: Todo[];
  /** Masque les tâches terminées. */
  hideDone?: boolean;
  /** Affiche les secondes d'une tuile Horloge. */
  showSeconds?: boolean;
  /** Ville d'une tuile Météo (Caen si absente). */
  location?: Place;
}

/** Une page de tuiles (un onglet). */
export interface PageSettings {
  id: string;
  name: string;
  /** Écran d'accueil : affiché à l'ouverture, il ne peut pas être supprimé. */
  home?: boolean;
  tiles: TileSettings[];
}

/** Configuration complète de l'application (fichier `config.json`). */
export interface Config {
  version: number;
  general: {
    /** Langue de l'interface (`fr` uniquement pour l'instant). */
    language: string;
    notificationsEnabled: boolean;
  };
  window: {
    /** Fenêtre toujours au-dessus des autres. */
    alwaysOnTop: boolean;
    fullscreen: boolean;
  };
  layout: {
    /** Pages de tuiles, dans l'ordre des onglets. */
    pages: PageSettings[];
  };
  /** Réglages du module GitHub (absents d'une ancienne configuration). */
  github?: {
    /** Intervalle de rafraîchissement de l'activité, en minutes. */
    refreshMinutes: number;
  };
  /** Réglages du module Gmail (absents d'une ancienne configuration). */
  gmail?: {
    /** Intervalle de rafraîchissement des non lus, en minutes. */
    refreshMinutes: number;
    /** Notification Windows pour chaque nouveau message. */
    notifyNew: boolean;
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

/** Indique si l'application se lance au démarrage de Windows. */
export function getAutostart(): Promise<boolean> {
  return call<boolean>("window_autostart_get");
}

/** Active ou désactive le lancement au démarrage de Windows. */
export function setAutostart(enabled: boolean): Promise<void> {
  return call<void>("window_autostart_set", { enabled });
}
