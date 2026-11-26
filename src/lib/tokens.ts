import { call } from "./tauri";

/** Services qui demandent un jeton de connexion. */
export type TokenKind = "github" | "gmail" | "claude";

/**
 * Indique, pour chaque service, si un jeton est enregistré.
 * La valeur des jetons ne quitte jamais le Rust : l'interface ne peut pas la relire.
 */
export function getTokenStatus(): Promise<Record<TokenKind, boolean>> {
  return call<Record<TokenKind, boolean>>("tokens_status");
}

/** Enregistre (ou remplace) un jeton dans le Gestionnaire d'identification Windows. */
export function saveToken(kind: TokenKind, value: string): Promise<void> {
  return call<void>("tokens_set", { kind, value });
}

/** Supprime un jeton. */
export function deleteToken(kind: TokenKind): Promise<void> {
  return call<void>("tokens_delete", { kind });
}
