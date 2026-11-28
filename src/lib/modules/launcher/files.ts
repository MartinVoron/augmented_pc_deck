// Choix de fichiers pour le lanceur : sélecteur du système et glisser-déposer.
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";

/** Ouvre le sélecteur de fichiers du système. Renvoie le chemin choisi, ou `null` si annulé. */
export async function pickPath(kind: "file" | "folder" | "image"): Promise<string | null> {
  const path = await open({
    directory: kind === "folder",
    multiple: false,
    title:
      kind === "folder"
        ? "Choisir un dossier"
        : kind === "image"
          ? "Choisir une icône"
          : "Choisir un programme ou un fichier",
    filters:
      kind === "image"
        ? [
            {
              name: "Images",
              extensions: ["png", "jpg", "jpeg", "gif", "webp", "svg", "ico", "bmp"],
            },
          ]
        : undefined,
  });
  return typeof path === "string" ? path : null;
}

/**
 * Écoute les fichiers déposés sur la fenêtre depuis l'Explorateur.
 * `ondrop` reçoit les chemins et l'élément de la page sous le curseur.
 * Renvoie une fonction qui arrête l'écoute.
 */
export function onFileDrop(ondrop: (paths: string[], element: Element | null) => void): () => void {
  let stop: (() => void) | undefined;
  let stopped = false;
  const listening = Promise.resolve()
    .then(() => getCurrentWebview())
    .then((webview) =>
      webview.onDragDropEvent((event) => {
        if (event.payload.type !== "drop" || event.payload.paths.length === 0) return;
        // Position en pixels physiques : on la ramène en pixels CSS.
        const ratio = window.devicePixelRatio || 1;
        const { x, y } = event.payload.position;
        ondrop(event.payload.paths, document.elementFromPoint(x / ratio, y / ratio));
      }),
    );
  listening
    .then((unlisten) => (stopped ? unlisten() : (stop = unlisten)))
    .catch(() => {
      // Hors de Tauri (tests), pas de glisser-déposer de fichiers.
    });
  return () => {
    stopped = true;
    stop?.();
  };
}
