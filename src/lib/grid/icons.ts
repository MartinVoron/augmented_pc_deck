import {
  Bot,
  Clock,
  CloudSun,
  Cpu,
  GitPullRequest,
  ListTodo,
  Mail,
  Music,
  Rocket,
  Timer,
} from "@lucide/svelte";
import type { TileKind } from "./defaultLayout";

/** Icône commune de chaque module (jeu d'icônes Lucide). */
export const KIND_ICONS = {
  launcher: Rocket,
  processes: Cpu,
  timers: Timer,
  todos: ListTodo,
  music: Music,
  github: GitPullRequest,
  gmail: Mail,
  claude: Bot,
  clock: Clock,
  weather: CloudSun,
} satisfies Record<TileKind, unknown>;
