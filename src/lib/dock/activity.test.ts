import { describe, expect, it } from "vitest";
import type { NowPlaying } from "../modules/music";
import type { TimerRun } from "../modules/timers";
import { pickActivity } from "./activity.svelte";

function track(status: NowPlaying["status"]): NowPlaying {
  return {
    title: "Clair de lune",
    artist: "Debussy",
    album: "",
    source: "Spotify",
    status,
    positionMs: 0,
    durationMs: 300000,
    updatedAt: 0,
    thumbnail: null,
    canPlayPause: true,
    canNext: true,
    canPrevious: true,
  };
}

function run(name: string, status: TimerRun["status"], endsAt: number | null): TimerRun {
  return { name, status, durationMs: 60000, remainingMs: 60000, endsAt };
}

describe("pickActivity", () => {
  it("montre la musique en lecture avant tout", () => {
    const activity = pickActivity(track("playing"), { a: run("Thé", "running", 10) });
    expect(activity?.kind).toBe("music");
  });

  it("montre sinon le minuteur qui finit le plus tôt", () => {
    const activity = pickActivity(track("paused"), {
      a: run("Pâtes", "running", 5000),
      b: run("Thé", "running", 2000),
      c: run("Pause", "paused", null),
    });
    expect(activity).toEqual({ kind: "timer", run: run("Thé", "running", 2000) });
  });

  it("ne montre rien sans lecture ni minuteur en marche", () => {
    expect(pickActivity(null, {})).toBeNull();
    expect(pickActivity(track("stopped"), { c: run("Pause", "paused", null) })).toBeNull();
  });
});
