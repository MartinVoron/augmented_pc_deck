// Claude : notifications de Claude Code, reçues par un petit serveur local
// (voir src-tauri/src/modules/claude).
import { call } from "../../tauri";

/** Ce qui s'est passé : Claude attend une réponse, ou a fini. */
export type ClaudeEventKind = "waiting" | "done";

/** Un événement reçu de Claude Code. */
export interface ClaudeEvent {
  id: number;
  kind: ClaudeEventKind;
  /** Nom du dossier du projet. */
  project: string;
  message: string;
  /** Heure de réception, en millisecondes depuis 1970. */
  receivedAt: number;
  sessionId: string;
}

/** État du serveur local. */
export type ServerState =
  { state: "starting" } | { state: "listening" } | { state: "failed"; error: string };

/** État de la mise en place, pour les paramètres. */
export interface ClaudeStatus {
  server: ServerState;
  port: number;
  scriptPath: string;
  scriptInstalled: boolean;
  /** Bloc à ajouter dans `~/.claude/settings.json`. */
  settingsSnippet: string;
  /** Emplacement des réglages de Claude Code (`%USERPROFILE%\.claude\settings.json` ou `~/.claude/settings.json`). */
  settingsFile: string;
}

/** Événement envoyé par le Rust à chaque nouvel événement. */
export const CHANGED_EVENT = "claude-events-changed";

/** Guide de mise en place du hook. */
export const GUIDE_URL =
  "https://github.com/MartinVoron/augmented_pc_deck/blob/main/docs/claude-hook.md";

export function getClaudeStatus(): Promise<ClaudeStatus> {
  return call<ClaudeStatus>("claude_status");
}

/** Crée (ou recrée) le script du hook avec le jeton actuel. */
export function installHook(): Promise<ClaudeStatus> {
  return call<ClaudeStatus>("claude_install_hook");
}

/** Remplace le jeton ; le script est réécrit s'il existe. */
export function regenerateToken(): Promise<ClaudeStatus> {
  return call<ClaudeStatus>("claude_regenerate_token");
}

/** Événements reçus depuis le lancement, du plus récent au plus ancien. */
export function getClaudeEvents(): Promise<ClaudeEvent[]> {
  return call<ClaudeEvent[]>("claude_events");
}

/** Efface tous les événements. */
export function clearClaudeEvents(): Promise<void> {
  return call<void>("claude_clear");
}

/**
 * Sessions qui attendent encore une réponse : celles dont le dernier événement
 * est « attend une réponse » (un « terminé » plus récent l'annule).
 * Les événements arrivent du plus récent au plus ancien.
 */
export function waitingSessions(events: ClaudeEvent[]): ClaudeEvent[] {
  const seen = new Set<string>();
  const waiting: ClaudeEvent[] = [];
  for (const event of events) {
    const key = event.sessionId || `${event.project}:${event.id}`;
    if (seen.has(key)) continue;
    seen.add(key);
    if (event.kind === "waiting") waiting.push(event);
  }
  return waiting;
}
