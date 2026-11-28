// Applique l'apparence choisie dans les paramètres à la page et à la fenêtre.
import { getCurrentWindow, Effect } from "@tauri-apps/api/window";
import { logError } from "../logging";
import { settings } from "../modules/settings/state.svelte";
import { appliedAppearance, applyToDocument } from ".";

/** Réglage clair ou sombre de Windows, suivi en direct. */
const system = $state({ dark: true });

/** Suit le réglage clair ou sombre du système. Renvoie la fonction qui arrête le suivi. */
export function watchSystemScheme(): () => void {
  const query = globalThis.matchMedia?.("(prefers-color-scheme: dark)");
  if (!query) return () => {};
  system.dark = query.matches;
  const onchange = (event: MediaQueryListEvent) => (system.dark = event.matches);
  query.addEventListener("change", onchange);
  return () => query.removeEventListener("change", onchange);
}

/** L'effet de fenêtre a échoué : on garde le fond peint jusqu'au prochain lancement. */
const effect = $state({ failed: false });

/** Dernier effet demandé à la fenêtre, pour ne pas le redemander à chaque changement. */
let effectOn = false;

async function setWindowEffect(on: boolean): Promise<void> {
  if (on === effectOn) return;
  effectOn = on;
  try {
    const window = getCurrentWindow();
    await (on ? window.setEffects({ effects: [Effect.Mica] }) : window.clearEffects());
  } catch (e) {
    // Hors de Tauri, ou système sans Mica.
    if (on) effect.failed = true;
    void logError("apparence", e);
  }
}

/**
 * Applique l'apparence de la configuration. À appeler dans un `$effect` :
 * il se relance quand la configuration, le mode de Windows ou l'ancrage changent.
 */
export function syncAppearance(docked: boolean): void {
  const applied = appliedAppearance(settings.config?.appearance, system.dark, docked);
  if (effect.failed) applied.windowEffect = false;
  applyToDocument(document.documentElement, applied);
  void setWindowEffect(applied.windowEffect);
}
