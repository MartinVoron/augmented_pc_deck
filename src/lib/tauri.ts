import { invoke } from "@tauri-apps/api/core";

/**
 * Erreur renvoyée par une commande Tauri (voir `src-tauri/src/error.rs`).
 * `code` est stable et sert au code ; `message` est en français, prêt à afficher.
 */
export class AppError extends Error {
  readonly code: string;

  constructor(code: string, message: string) {
    super(message);
    this.name = "AppError";
    this.code = code;
  }
}

function toAppError(raw: unknown): AppError {
  if (typeof raw === "object" && raw !== null && "code" in raw && "message" in raw) {
    return new AppError(String(raw.code), String(raw.message));
  }
  return new AppError("other", String(raw));
}

/**
 * Appelle une commande Tauri. Toute l'interface passe par cette fonction
 * plutôt que par `invoke` directement, pour recevoir des erreurs homogènes.
 */
export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (raw) {
    throw toAppError(raw);
  }
}
