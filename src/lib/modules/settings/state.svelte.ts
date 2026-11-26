import { logError } from "../../logging";
import { getSettings, saveSettings, type Config } from ".";

/**
 * Configuration partagée par toute l'interface. Chaque changement est enregistré
 * tout de suite par le Rust, qui l'applique sans redémarrer l'application.
 */
export const settings = $state<{ config: Config | null; error: string }>({
  config: null,
  error: "",
});

/** Charge la configuration au démarrage de l'interface. */
export async function loadSettings(): Promise<void> {
  try {
    settings.config = await getSettings();
    settings.error = "";
  } catch (e) {
    settings.error = (e as Error).message;
    void logError("paramètres", e);
  }
}

/** Modifie la configuration et l'enregistre. En cas d'échec, l'ancienne valeur est gardée. */
export async function updateSettings(change: (config: Config) => void): Promise<void> {
  if (!settings.config) return;
  const next = structuredClone($state.snapshot(settings.config));
  change(next);
  try {
    settings.config = await saveSettings(next);
    settings.error = "";
  } catch (e) {
    settings.error = (e as Error).message;
    void logError("paramètres", e);
  }
}
