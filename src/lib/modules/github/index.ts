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

/** Un élément à suivre : PR, issue ou notification. */
export interface ActivityItem {
  title: string;
  /** Dépôt, sous la forme `propriétaire/nom`. */
  repo: string;
  number: number | null;
  url: string;
  /** Date de dernière mise à jour (ISO 8601). */
  updatedAt: string;
  /** `pullRequest`, `issue`, ou le type d'une notification (`release`…). */
  kind: string;
  /** Raison d'une notification (`review_requested`, `mention`…). */
  reason: string | null;
}

/** Une liste de l'activité ; `error` si sa requête a échoué. */
export interface ActivitySection {
  items: ActivityItem[];
  total: number;
  error: string | null;
}

/** Tout ce qui attend une action sur GitHub. */
export interface GithubActivity {
  reviewRequests: ActivitySection;
  myPullRequests: ActivitySection;
  assignedIssues: ActivitySection;
  notifications: ActivitySection;
}

export type SectionKey = keyof GithubActivity;

/** Les quatre listes de la tuile, dans l'ordre d'affichage. */
export const SECTIONS: { key: SectionKey; label: string; short: string }[] = [
  { key: "reviewRequests", label: "PR à relire", short: "À relire" },
  { key: "myPullRequests", label: "Mes PR ouvertes", short: "Mes PR" },
  { key: "assignedIssues", label: "Issues assignées", short: "Issues" },
  { key: "notifications", label: "Notifications non lues", short: "Notifs" },
];

/** Intervalle de rafraîchissement par défaut, et ses limites, en minutes. */
export const DEFAULT_REFRESH_MINUTES = 5;
export const MIN_REFRESH_MINUTES = 1;
export const MAX_REFRESH_MINUTES = 60;

/** Ramène un intervalle saisi dans les limites (5 min s'il est illisible). */
export function clampRefreshMinutes(minutes: number | undefined): number {
  if (!minutes || !Number.isFinite(minutes)) return DEFAULT_REFRESH_MINUTES;
  return Math.min(MAX_REFRESH_MINUTES, Math.max(MIN_REFRESH_MINUTES, Math.round(minutes)));
}

/** Récupère l'activité GitHub. */
export function getGithubActivity(): Promise<GithubActivity> {
  return call<GithubActivity>("github_activity");
}

const REASONS: Record<string, string> = {
  review_requested: "relecture demandée",
  mention: "mentionné",
  team_mention: "équipe mentionnée",
  assign: "assigné",
  author: "auteur",
  comment: "commentaire",
  ci_activity: "intégration continue",
  state_change: "changement d'état",
  subscribed: "abonné",
  manual: "abonné",
  security_alert: "alerte de sécurité",
  invitation: "invitation",
};

/** Raison d'une notification, en français. */
export function reasonLabel(reason: string | null): string {
  if (!reason) return "";
  return REASONS[reason] ?? reason.replace(/_/g, " ");
}

/** Temps écoulé, en court : « à l'instant », « 5 min », « 3 h », « 2 j ». */
export function timeAgo(iso: string, now = Date.now()): string {
  const minutes = Math.floor((now - Date.parse(iso)) / 60_000);
  if (!Number.isFinite(minutes) || minutes < 1) return "à l'instant";
  if (minutes < 60) return `${minutes} min`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours} h`;
  return `${Math.floor(hours / 24)} j`;
}
