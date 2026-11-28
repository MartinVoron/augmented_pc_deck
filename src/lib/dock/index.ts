// Ancrage de la fenêtre en encoche sur un bord de l'écran (voir src-tauri/src/dock.rs).
import { call } from "../tauri";

/** Bord de l'écran où la fenêtre est ancrée (`none` : fenêtre normale). */
export type DockEdge = "none" | "top" | "bottom" | "left" | "right";

/** Apparence de l'encoche repliée : noire avec le logo, ou entièrement noire. */
export type NotchStyle = "logo" | "black";

/** État de l'ancrage, envoyé par le Rust à chaque changement. */
export interface DockStatus {
  edge: DockEdge;
  /** Panneau ouvert (la souris est dessus) ; sinon, encoche repliée. */
  expanded: boolean;
}

/** Événement envoyé par le Rust quand l'ancrage change. */
export const DOCK_EVENT = "dock-state";

/** Taille de l'encoche repliée, en pixels : longueur le long du bord, profondeur. */
export const NOTCH_LENGTH = 180;
export const NOTCH_DEPTH = 32;

/** Lit l'état actuel de l'ancrage. */
export function getDockStatus(): Promise<DockStatus> {
  return call<DockStatus>("dock_status");
}

/**
 * Donne au Rust la taille du panneau ouvert, en pixels logiques. Il la réduit
 * à la taille de l'écran si besoin.
 */
export function setDockPanel(width: number, height: number): Promise<void> {
  return call<void>("dock_set_panel", { width, height });
}
