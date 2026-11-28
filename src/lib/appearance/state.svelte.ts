// Applique l'apparence choisie dans les paramètres à la page.
import { settings } from "../modules/settings/state.svelte";
import { appliedAppearance, applyToDocument } from ".";

/** Réglage clair ou sombre de Windows, suivi en direct. */
const system = $state({ dark: true });

/**
 * Niveau de flou en cours de réglage (curseur pas encore lâché) : affiché tout de suite,
 * enregistré seulement au lâcher du curseur.
 */
export const blurPreview = $state<{ value: number | null }>({ value: null });

/** Suit le réglage clair ou sombre du système. Renvoie la fonction qui arrête le suivi. */
export function watchSystemScheme(): () => void {
  const query = globalThis.matchMedia?.("(prefers-color-scheme: dark)");
  if (!query) return () => {};
  system.dark = query.matches;
  const onchange = (event: MediaQueryListEvent) => (system.dark = event.matches);
  query.addEventListener("change", onchange);
  return () => query.removeEventListener("change", onchange);
}

/**
 * Applique l'apparence de la configuration. À appeler dans un `$effect` :
 * il se relance quand la configuration, le mode de Windows, l'ancrage ou le curseur changent.
 */
export function syncAppearance(docked: boolean): void {
  const appearance = settings.config?.appearance;
  const value =
    blurPreview.value === null ? appearance : { ...appearance, blur: blurPreview.value };
  applyToDocument(document.documentElement, appliedAppearance(value, system.dark, docked));
}
