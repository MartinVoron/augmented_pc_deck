// Activité en direct montrée dans l'encoche repliée, comme boring.notch :
// le morceau en cours, ou à défaut le minuteur qui tourne.
import { logError } from "../logging";
import { getNowPlaying, type NowPlaying } from "../modules/music";
import { listRuns, onTimersChanged, type TimerRun, type TimerRuns } from "../modules/timers";
import { poll } from "../polling";

export type Activity =
  { kind: "music"; track: NowPlaying } | { kind: "timer"; run: TimerRun } | null;

/** Intervalle de lecture du morceau en cours. */
const MUSIC_POLL_MS = 2000;

/**
 * Choisit l'activité à montrer : un morceau en lecture d'abord, sinon le minuteur
 * en marche qui finit le plus tôt, sinon rien.
 */
export function pickActivity(track: NowPlaying | null, runs: TimerRuns): Activity {
  if (track?.status === "playing") return { kind: "music", track };
  const running = Object.values(runs)
    .filter((run) => run.status === "running" && run.endsAt !== null)
    .sort((a, b) => (a.endsAt ?? 0) - (b.endsAt ?? 0));
  return running.length > 0 ? { kind: "timer", run: running[0] } : null;
}

/** Morceau et minuteurs actuels, tenus à jour par `watchActivity`. */
export const live = $state<{ track: NowPlaying | null; runs: TimerRuns }>({
  track: null,
  runs: {},
});

/** Suit le morceau en cours et les minuteurs. Renvoie la fonction d'arrêt. */
export function watchActivity(): () => void {
  // Sans lecteur de musique joignable, l'erreur revient à chaque lecture : elle
  // n'est notée qu'une fois, jusqu'à ce que la lecture remarche.
  let failing = false;
  const stopPolling = poll(async () => {
    try {
      live.track = await getNowPlaying();
      failing = false;
    } catch (e) {
      live.track = null;
      if (!failing) void logError("encoche", e);
      failing = true;
    }
  }, MUSIC_POLL_MS);
  const stopTimers = onTimersChanged((runs) => {
    live.runs = runs;
  });
  listRuns()
    .then((runs) => {
      live.runs = runs ?? {};
    })
    .catch((e: unknown) => logError("encoche", e));
  return () => {
    stopPolling();
    stopTimers();
  };
}
