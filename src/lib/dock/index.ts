// Ancrage de la fenêtre en encoche sur un bord de l'écran (voir src-tauri/src/dock.rs).
import { call } from "../tauri";

/** Bord de l'écran où la fenêtre est ancrée (`none` : fenêtre normale). */
export type DockEdge = "none" | "top" | "bottom" | "left" | "right";

/** Nom de chaque bord, pour les paramètres. */
export const DOCK_EDGE_LABELS: Record<DockEdge, string> = {
  none: "Non, fenêtre normale",
  top: "En haut",
  bottom: "En bas",
  left: "À gauche",
  right: "À droite",
};

/** Apparence de l'encoche repliée : noire avec le logo, ou entièrement noire. */
export type NotchStyle = "logo" | "black";

/** État de l'ancrage, envoyé par le Rust à chaque changement. */
export interface DockStatus {
  edge: DockEdge;
  /** Panneau ouvert (la souris est dessus) ; sinon, encoche repliée. */
  expanded: boolean;
  /**
   * Le bureau a refusé de placer l'encoche contre le bord de l'écran : la fenêtre
   * est revenue en fenêtre normale.
   */
  refused?: boolean;
}

/** Événement envoyé par le Rust quand l'ancrage change. */
export const DOCK_EVENT = "dock-state";

/** Taille de l'encoche repliée, en pixels : longueur le long du bord, profondeur. */
export const NOTCH_LENGTH = 180;
export const NOTCH_DEPTH = 32;

/**
 * Taille des « oreilles » de l'encoche : les coins creusés qui la raccordent au bord
 * de l'écran, comme l'encoche d'un MacBook. Elles prennent place de chaque côté.
 */
export const NOTCH_EAR = 10;
/** Marge laissée autour du panneau pour que le rebond du ressort ne soit pas coupé. */
export const BOUNCE_ROOM = 16;

/** Lit l'état actuel de l'ancrage. */
export function getDockStatus(): Promise<DockStatus> {
  return call<DockStatus>("dock_status");
}

/** Longueur de l'encoche repliée quand elle montre une activité en direct. */
export const NOTCH_ACTIVE_LENGTH = 300;

/** Donne au Rust la longueur de l'encoche repliée, en pixels logiques. */
export function setDockNotch(length: number): Promise<void> {
  return call<void>("dock_set_notch", { length });
}

/**
 * Donne au Rust la taille du panneau ouvert, en pixels logiques. Il la réduit
 * à la taille de l'écran si besoin.
 */
export function setDockPanel(width: number, height: number): Promise<void> {
  return call<void>("dock_set_panel", { width, height });
}

/**
 * Dit au Rust si la souris est dans la fenêtre ancrée. Sous Linux, il ne peut pas
 * suivre la souris lui-même : c'est ce qui ouvre et replie le panneau.
 */
export function setDockPointer(inside: boolean): Promise<void> {
  return call<void>("dock_pointer", { inside });
}
