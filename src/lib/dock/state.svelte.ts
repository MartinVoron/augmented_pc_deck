import { onEvent } from "../events";
import { logError } from "../logging";
import { DOCK_EVENT, getDockStatus, type DockStatus } from ".";

/** État de l'ancrage, tenu à jour par les événements du Rust. */
export const dock = $state<DockStatus>({ edge: "none", expanded: false });

const EDGES = new Set(["none", "top", "bottom", "left", "right"]);

function update(status: DockStatus) {
  if (!EDGES.has(status?.edge)) return;
  dock.edge = status.edge;
  dock.expanded = status.expanded === true;
}

/** Lit l'état de l'ancrage puis suit ses changements. Renvoie la fonction d'arrêt. */
export function watchDock(): () => void {
  const stop = onEvent<DockStatus>(DOCK_EVENT, update);
  getDockStatus()
    .then(update)
    .catch((e: unknown) => logError("ancrage", e));
  return stop;
}
