import { describe, expect, it } from "vitest";
import {
  DEFAULT_POMODORO,
  nextPhase,
  phaseDurationMs,
  phaseEndAlert,
  sanitizePomodoro,
} from "./pomodoro";
import { formatStopwatch, lapDuration, Stopwatch } from "./stopwatch.svelte";

describe("Pomodoro", () => {
  it("alterne travail et pauses, avec une longue pause toutes les 4 séances", () => {
    let progress = { phase: "work" as const, completedWork: 0 } as ReturnType<typeof nextPhase>;
    const phases: string[] = [];
    for (let i = 0; i < 8; i++) {
      progress = nextPhase(progress, DEFAULT_POMODORO);
      phases.push(progress.phase);
    }
    expect(phases).toEqual([
      "shortBreak",
      "work",
      "shortBreak",
      "work",
      "shortBreak",
      "work",
      "longBreak",
      "work",
    ]);
    expect(progress.completedWork).toBe(4);
  });

  it("annonce la phase suivante dans la notification", () => {
    expect(phaseEndAlert({ phase: "work", completedWork: 3 }, DEFAULT_POMODORO)).toEqual({
      title: "Séance de travail terminée",
      body: "Place à la longue pause (15 min).",
    });
    expect(phaseEndAlert({ phase: "shortBreak", completedWork: 1 }, DEFAULT_POMODORO).body).toBe(
      "On reprend le travail (25 min).",
    );
  });

  it("corrige les durées saisies", () => {
    expect(sanitizePomodoro({ workMins: 50, shortBreakMins: 0, longBreakMins: 999 })).toEqual({
      workMins: 50,
      shortBreakMins: 5,
      longBreakMins: 180,
      sessionsBeforeLongBreak: 4,
    });
    expect(phaseDurationMs("work", DEFAULT_POMODORO)).toBe(25 * 60_000);
  });
});

describe("Chronomètre", () => {
  it("compte le temps, se met en pause et enregistre des tours", () => {
    const stopwatch = new Stopwatch();
    stopwatch.start(1_000);
    stopwatch.lap(4_000);
    stopwatch.lap(10_500);
    stopwatch.pause(12_000);
    expect(stopwatch.elapsed(99_000)).toBe(11_000);
    expect(stopwatch.laps).toEqual([3_000, 9_500]);
    expect(lapDuration(stopwatch.laps, 1)).toBe(6_500);

    stopwatch.start(20_000);
    expect(stopwatch.elapsed(21_000)).toBe(12_000);
    stopwatch.reset();
    expect(stopwatch.elapsed(50_000)).toBe(0);
    expect(stopwatch.laps).toEqual([]);
  });

  it("affiche le temps au dixième", () => {
    expect(formatStopwatch(7_349)).toBe("0:07,3");
    expect(formatStopwatch(3_723_400)).toBe("1:02:03,4");
  });
});
