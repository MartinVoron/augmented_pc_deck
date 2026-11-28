// Apparence de l'interface : thème, mode clair ou sombre et niveau de flou
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
  /** Flou des tuiles du thème Verre, en pourcentage : 0 transparent, 100 flou complet. */
  blur: number;
}

export const DEFAULT_APPEARANCE: AppearanceSettings = {
  theme: "classic",
  mode: "system",
  blur: 100,
};

/** Ramène un niveau de flou entre 0 et 100 (100 s'il est absent ou invalide). */
export function clampBlur(value: unknown): number {
  if (typeof value !== "number" || !Number.isFinite(value)) return DEFAULT_APPEARANCE.blur;
  return Math.round(Math.min(100, Math.max(0, value)));
}

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
  return { theme, mode, blur: clampBlur(value?.blur) };
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
  /** Niveau de flou des tuiles, de 0 à 100. */
  blur: number;
}

/**
 * Apparence à appliquer. Ancrée en encoche, la fenêtre garde le thème Classique :
 * l'encoche a son propre dessin, transparent autour.
 */
export function appliedAppearance(
  value: Partial<AppearanceSettings> | null | undefined,
  systemDark: boolean,
  docked: boolean,
): AppliedAppearance {
  const appearance = sanitizeAppearance(value);
  if (docked) return { theme: "classic", scheme: "dark", blur: DEFAULT_APPEARANCE.blur };
  return {
    theme: appearance.theme,
    scheme: resolveScheme(appearance, systemDark),
    blur: appearance.blur,
  };
}

/** Pose les attributs lus par theme.css sur la racine du document. */
export function applyToDocument(root: HTMLElement, applied: AppliedAppearance): void {
  root.dataset.theme = applied.theme;
  root.dataset.scheme = applied.scheme;
  // De 0 à 1 : theme.css en déduit le flou et l'opacité des tuiles du thème Verre.
  root.style.setProperty("--glass-level", String(applied.blur / 100));
}
