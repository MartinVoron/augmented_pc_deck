import { error as writeError } from "@tauri-apps/plugin-log";

/** Transforme n'importe quelle erreur en texte lisible pour le journal. */
export function describeError(raw: unknown): string {
  if (raw instanceof Error) {
    return raw.stack ?? `${raw.name}: ${raw.message}`;
  }
  return String(raw);
}

/**
 * Écrit une erreur de l'interface dans le journal de l'application.
 * Ne lance jamais d'exception : si le journal est indisponible, l'erreur va dans la console.
 */
export async function logError(context: string, raw: unknown): Promise<void> {
  const message = `[${context}] ${describeError(raw)}`;
  console.error(message);
  try {
    await writeError(message);
  } catch {
    // Journal indisponible (par exemple dans les tests) : la console suffit.
  }
}

/** Envoie au journal les erreurs que personne n'a attrapées. */
export function logUncaughtErrors(): void {
  window.addEventListener("error", (event) => {
    void logError("interface", event.error ?? event.message);
  });
  window.addEventListener("unhandledrejection", (event) => {
    void logError("interface", event.reason);
  });
}
