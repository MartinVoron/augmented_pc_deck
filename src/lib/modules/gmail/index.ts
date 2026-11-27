// Gmail : connexion OAuth en lecture seule, puis e-mails non lus
// (voir src-tauri/src/modules/gmail).
import { call } from "../../tauri";

/** Compte Gmail connecté. */
export interface GmailAccount {
  email: string;
}

/** Console Google Cloud, où se crée l'identifiant OAuth. */
export const CREDENTIALS_URL = "https://console.cloud.google.com/apis/credentials";

/** Guide pas à pas pour créer l'identifiant OAuth. */
export const GUIDE_URL =
  "https://github.com/MartinVoron/augmented_pc_deck/blob/main/docs/gmail-oauth.md";

/**
 * Ouvre la page de connexion de Google dans le navigateur, puis attend l'accord
 * de l'utilisateur (5 minutes au plus).
 */
export function connectGmail(clientId: string, clientSecret: string): Promise<GmailAccount> {
  return call<GmailAccount>("gmail_connect", { clientId, clientSecret });
}

/** Compte connecté, ou `null`. */
export function getGmailAccount(): Promise<GmailAccount | null> {
  return call<GmailAccount | null>("gmail_account");
}

/** Révoque l'accès chez Google et efface la connexion. */
export function disconnectGmail(): Promise<void> {
  return call<void>("gmail_disconnect");
}
