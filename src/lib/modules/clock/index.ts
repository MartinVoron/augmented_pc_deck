// Horloge : mise en forme de l'heure et de la date, en français.

/** Heure : « 14:05 », ou « 14:05:09 » avec les secondes. */
export function formatTime(date: Date, seconds = false): string {
  return date.toLocaleTimeString("fr-FR", {
    hour: "2-digit",
    minute: "2-digit",
    ...(seconds ? { second: "2-digit" } : {}),
  });
}

/** Date complète : « Vendredi 27 novembre 2026 ». */
export function formatDate(date: Date): string {
  const text = date.toLocaleDateString("fr-FR", {
    weekday: "long",
    day: "numeric",
    month: "long",
    year: "numeric",
  });
  return text.charAt(0).toUpperCase() + text.slice(1);
}

/** Date courte : « ven. 27 nov. ». */
export function formatShortDate(date: Date): string {
  return date.toLocaleDateString("fr-FR", { weekday: "short", day: "numeric", month: "short" });
}

/** Délai jusqu'au prochain changement d'affichage (seconde ou minute suivante). */
export function nextTickDelay(date: Date, seconds = false): number {
  const ms = date.getMilliseconds();
  if (seconds) return 1000 - ms;
  return (60 - date.getSeconds()) * 1000 - ms;
}
