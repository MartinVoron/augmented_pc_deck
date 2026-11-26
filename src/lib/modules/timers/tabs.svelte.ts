// Onglet choisi dans chaque tuile des minuteurs, gardé tant que l'application tourne.
export type TimersTab = "countdowns" | "stopwatch" | "pomodoro";

const tabs = $state<Record<string, TimersTab>>({});

export function selectedTab(tileId: string): TimersTab {
  return tabs[tileId] ?? "countdowns";
}

export function selectTab(tileId: string, tab: TimersTab): void {
  tabs[tileId] = tab;
}
