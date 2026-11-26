// Processus : usage du processeur, de la mémoire et des disques (voir src-tauri/src/modules/processes).
import { call } from "../../tauri";

export interface DiskUsage {
  /** Lettre du lecteur (`C:`). */
  name: string;
  used: number;
  total: number;
}

export interface Usage {
  /** Usage du processeur, en pourcentage. */
  cpu: number;
  memoryUsed: number;
  memoryTotal: number;
  disks: DiskUsage[];
}

/** Mesure l'usage actuel du PC. */
export function getUsage(): Promise<Usage> {
  return call<Usage>("processes_usage");
}

const GIGA = 1024 ** 3;
const number = new Intl.NumberFormat("fr-FR", { maximumFractionDigits: 1 });

/** Taille lisible : « 512 Mo », « 9,1 Go », « 1,8 To ». */
export function formatBytes(bytes: number): string {
  if (bytes >= 1024 * GIGA) return `${number.format(bytes / (1024 * GIGA))} To`;
  if (bytes >= GIGA) return `${number.format(bytes / GIGA)} Go`;
  return `${Math.round(bytes / 1024 ** 2)} Mo`;
}

/** Pourcentage entre 0 et 100 (0 si le total est inconnu). */
export function percent(used: number, total: number): number {
  return total > 0 ? Math.min(100, Math.max(0, (used / total) * 100)) : 0;
}
