import { listen } from "@tauri-apps/api/event";

/**
 * Écoute un événement envoyé par le Rust. Renvoie une fonction qui arrête l'écoute,
 * utilisable directement comme valeur de retour de `onMount`.
 * Hors de Tauri (tests), l'écoute échoue sans bruit.
 */
export function onEvent<T>(name: string, handler: (payload: T) => void): () => void {
  let stop: (() => void) | undefined;
  let stopped = false;
  Promise.resolve()
    .then(() => listen<T>(name, (event) => handler(event.payload)))
    .then((unlisten) => (stopped ? unlisten() : (stop = unlisten)))
    .catch(() => {
      // Pas de Tauri : rien à écouter.
    });
  return () => {
    stopped = true;
    stop?.();
  };
}
