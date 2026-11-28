import { call } from "./tauri";

/** Sons standards du système (voir `src-tauri/src/notifications.rs`). */
export type NotificationSound = "default" | "reminder" | "mail" | "alarm";

export interface AppNotification {
  title: string;
  body: string;
  /** Sans son, la notification est silencieuse. */
  sound?: NotificationSound;
}

/**
 * Envoie une notification du système. Renvoie `false` si les notifications sont coupées.
 */
export function sendNotification(notification: AppNotification): Promise<boolean> {
  return call<boolean>("notifications_send", { notification });
}

/** Indique si les notifications sont activées. */
export function areNotificationsEnabled(): Promise<boolean> {
  return call<boolean>("notifications_is_enabled");
}

/** Active ou coupe toutes les notifications. */
export function setNotificationsEnabled(enabled: boolean): Promise<void> {
  return call<void>("notifications_set_enabled", { enabled });
}
