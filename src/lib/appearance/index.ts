// Apparence de l'interface : thème, mode clair ou sombre et effet de fenêtre
// (enregistrés dans la configuration, voir src-tauri/src/modules/settings/config.rs).

/** Thème de l'interface. */
export type Theme = "classic" | "glass";

/** Mode clair ou sombre ; `system` suit le réglage de Windows. */
export type ColorMode = "system" | "dark" | "light";

/** Palette effectivement affichée. */
export type ColorScheme = "dark" | "light";

export interface AppearanceSettings {
  theme: Theme;
  mode: ColorMode;
  /** Flou natif de Windows 11 (Mica) sous la fenêtre, pour le thème Verre. */
  windowEffect: boolean;
}

export const DEFAULT_APPEARANCE: AppearanceSettings = {
  theme: "classic",
  mode: "system",
  windowEffect: false,
};

/** Nom de chaque thème, pour les paramètres. */
export const THEME_LABELS: Record<Theme, string> = {
  classic: "Classique",
  glass: "Verre",
};

/** Nom de chaque mode, pour les paramètres. */
export const MODE_LABELS: Record<ColorMode, string> = {
  system: "Comme Windows",
  dark: "Sombre",
  light: "Clair",
};

/** Complète une apparence absente ou incomplète (ancienne configuration). */
export function sanitizeAppearance(value?: Partial<AppearanceSettings> | null): AppearanceSettings {
  const theme = value?.theme && value.theme in THEME_LABELS ? value.theme : "classic";
  const mode = value?.mode && value.mode in MODE_LABELS ? value.mode : "system";
  return { theme, mode, windowEffect: value?.windowEffect === true };
}

/**
 * Palette à afficher. Le thème Classique n'existe qu'en sombre ;
 * le mode `system` suit Windows.
 */
export function resolveScheme(appearance: AppearanceSettings, systemDark: boolean): ColorScheme {
  if (appearance.theme === "classic") return "dark";
  if (appearance.mode === "system") return systemDark ? "dark" : "light";
  return appearance.mode;
}

/** Ce qu'il faut appliquer à la page et à la fenêtre. */
export interface AppliedAppearance {
  theme: Theme;
  scheme: ColorScheme;
  /** Effet Mica sous la fenêtre. */
  windowEffect: boolean;
}

/**
 * Apparence à appliquer. Ancrée en encoche, la fenêtre garde le thème Classique
 * et aucun effet : l'encoche a son propre dessin, transparent autour.
 */
export function appliedAppearance(
  value: Partial<AppearanceSettings> | null | undefined,
  systemDark: boolean,
  docked: boolean,
): AppliedAppearance {
  const appearance = sanitizeAppearance(value);
  if (docked) return { theme: "classic", scheme: "dark", windowEffect: false };
  return {
    theme: appearance.theme,
    scheme: resolveScheme(appearance, systemDark),
    windowEffect: appearance.theme === "glass" && appearance.windowEffect,
  };
}

/** Pose les attributs lus par theme.css sur la racine du document. */
export function applyToDocument(root: HTMLElement, applied: AppliedAppearance): void {
  root.dataset.theme = applied.theme;
  root.dataset.scheme = applied.scheme;
  root.toggleAttribute("data-window-effect", applied.windowEffect);
}
