// Musique : morceau en cours, via les sessions média de Windows
// (voir src-tauri/src/modules/music).
import { call } from "../../tauri";

export type PlaybackStatus = "playing" | "paused" | "stopped";

export interface NowPlaying {
  title: string;
  artist: string;
  album: string;
  /** Application qui joue (« Spotify », « Chrome »…). */
  source: string;
  status: PlaybackStatus;
  /** Position au moment `updatedAt`, en millisecondes. */
  positionMs: number | null;
  durationMs: number | null;
  /** Heure de la dernière mise à jour de la position (millisecondes depuis 1970). */
  updatedAt: number;
  /** Pochette en `data:` URL. */
  thumbnail: string | null;
  canPlayPause: boolean;
  canNext: boolean;
  canPrevious: boolean;
}

/** Morceau en cours, ou `null` quand rien ne joue. */
export function getNowPlaying(): Promise<NowPlaying | null> {
  return call<NowPlaying | null>("music_now_playing");
}

/** Position actuelle : pendant la lecture, elle avance depuis la dernière mise à jour. */
export function currentPosition(track: NowPlaying, now: number): number | null {
  if (track.positionMs === null) return null;
  const elapsed = track.status === "playing" ? Math.max(0, now - track.updatedAt) : 0;
  const position = track.positionMs + elapsed;
  return track.durationMs !== null ? Math.min(position, track.durationMs) : position;
}

/** Temps lisible : « 3:05 », « 1:02:03 ». */
export function formatTime(ms: number): string {
  const total = Math.floor(Math.max(0, ms) / 1000);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = String(total % 60).padStart(2, "0");
  return hours > 0
    ? `${hours}:${String(minutes).padStart(2, "0")}:${seconds}`
    : `${minutes}:${seconds}`;
}
