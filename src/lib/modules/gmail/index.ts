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

/** Un e-mail non lu. */
export interface GmailMessage {
  id: string;
  /** Nom de l'expéditeur (ou son adresse). */
  from: string;
  subject: string;
  /** Début du texte. */
  snippet: string;
  /** Date de réception, en millisecondes depuis 1970. */
  receivedAt: number;
  /** Page du message dans Gmail. */
  url: string;
}

/** Boîte de réception : nombre de non lus et derniers messages. */
export interface Inbox {
  email: string;
  unread: number;
  messages: GmailMessage[];
}

/** Intervalle de rafraîchissement par défaut, et ses limites, en minutes. */
export const DEFAULT_REFRESH_MINUTES = 2;
export const MIN_REFRESH_MINUTES = 1;
export const MAX_REFRESH_MINUTES = 60;

/** Ramène un intervalle saisi dans les limites (2 min s'il est illisible). */
export function clampRefreshMinutes(minutes: number | undefined): number {
  if (!minutes || !Number.isFinite(minutes)) return DEFAULT_REFRESH_MINUTES;
  return Math.min(MAX_REFRESH_MINUTES, Math.max(MIN_REFRESH_MINUTES, Math.round(minutes)));
}

/**
 * Récupère les non lus. Avec `notify`, chaque message arrivé depuis l'appel
 * précédent déclenche une notification Windows.
 */
export function getUnread(notify: boolean): Promise<Inbox> {
  return call<Inbox>("gmail_unread", { notify });
}

/** Boîte de réception dans Gmail, sur le bon compte. */
export function inboxUrl(email: string): string {
  return `https://mail.google.com/mail/u/?authuser=${encodeURIComponent(email)}#inbox`;
}
