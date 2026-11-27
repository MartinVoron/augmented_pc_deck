// GitHub : connexion avec un jeton personnel, puis activité (voir src-tauri/src/modules/github.rs).
import { call } from "../../tauri";

/** Compte GitHub connecté. */
export interface GithubAccount {
  login: string;
  name: string | null;
  avatarUrl: string;
  htmlUrl: string;
  /** Droits d'un jeton classique ; `null` pour un jeton à accès précis. */
  scopes: string[] | null;
}

/** Droits dont un jeton classique a besoin pour toute la tuile. */
export const REQUIRED_SCOPES = ["repo", "notifications"];

/** Page de GitHub qui crée un jeton classique avec les bons droits, déjà cochés. */
export const NEW_TOKEN_URL =
  "https://github.com/settings/tokens/new?scopes=repo,notifications&description=Augmented%20PC%20Deck";

/** Droits nécessaires qui manquent à un jeton classique. */
export function missingScopes(account: GithubAccount): string[] {
  if (!account.scopes) return [];
  return REQUIRED_SCOPES.filter((scope) => !account.scopes!.includes(scope));
}

/** Vérifie le jeton auprès de GitHub et l'enregistre s'il est accepté. */
export function connectGithub(token: string): Promise<GithubAccount> {
  return call<GithubAccount>("github_connect", { token });
}

/** Compte connecté, ou `null` si aucun jeton n'est enregistré. */
export function getGithubAccount(): Promise<GithubAccount | null> {
  return call<GithubAccount | null>("github_account");
}

/** Supprime le jeton. */
export function disconnectGithub(): Promise<void> {
  return call<void>("github_disconnect");
}

/** Ouvre une page de GitHub dans le navigateur. */
export function openInBrowser(url: string): Promise<void> {
  return call<void>("launcher_launch", { target: url, args: "" });
}
