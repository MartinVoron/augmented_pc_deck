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

/** Un processus en cours d'exécution. */
export interface ProcessInfo {
  pid: number;
  name: string;
  /** Part du processeur de toute la machine, en pourcentage. */
  cpu: number;
  /** Mémoire utilisée, en octets. */
  memory: number;
}

export type SortKey = "name" | "cpu" | "memory";
export type SortDirection = "asc" | "desc";

/** Liste les processus en cours. */
export function listProcesses(): Promise<ProcessInfo[]> {
  return call<ProcessInfo[]>("processes_list");
}

/** Arrête un processus (le nom évite d'arrêter un autre programme au même numéro). */
export function killProcess(process: ProcessInfo): Promise<void> {
  return call<void>("processes_kill", { pid: process.pid, name: process.name });
}

/** Garde les processus dont le nom contient `query` (sans tenir compte des majuscules). */
export function filterProcesses(list: ProcessInfo[], query: string): ProcessInfo[] {
  const text = query.trim().toLocaleLowerCase("fr");
  return text ? list.filter((p) => p.name.toLocaleLowerCase("fr").includes(text)) : list;
}

/** Trie les processus ; à égalité, par nom puis par numéro pour un ordre stable. */
export function sortProcesses(
  list: ProcessInfo[],
  key: SortKey,
  direction: SortDirection,
): ProcessInfo[] {
  const sign = direction === "asc" ? 1 : -1;
  const byName = (a: ProcessInfo, b: ProcessInfo) =>
    a.name.localeCompare(b.name, "fr", { sensitivity: "base" }) || a.pid - b.pid;
  return [...list].sort((a, b) =>
    key === "name" ? sign * byName(a, b) : sign * (a[key] - b[key]) || byName(a, b),
  );
}
