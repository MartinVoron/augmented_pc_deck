import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { formatDuration, remainingMs, type TimerRuns } from ".";
import type { TileConfig } from "../../grid/defaultLayout";
import CountdownList from "./CountdownList.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => {}) }));
const mockedInvoke = vi.mocked(invoke);

const tile: TileConfig = {
  id: "timers-1",
  kind: "timers",
  title: "Minuteurs",
  size: "1x1",
  timers: [
    { id: "minuteur-1", name: "Thé", durationSecs: 240 },
    { id: "minuteur-2", name: "Pâtes", durationSecs: 600 },
    { id: "minuteur-3", name: "Lessive", durationSecs: 3600 },
  ],
};

let runs: TimerRuns;

describe("CountdownList", () => {
  beforeEach(() => {
    runs = {
      "timers-1:minuteur-2": {
        name: "Pâtes",
        status: "running",
        durationMs: 600_000,
        remainingMs: 600_000,
        endsAt: Date.now() + 65_000,
      },
      "timers-1:minuteur-3": {
        name: "Lessive",
        status: "finished",
        durationMs: 3_600_000,
        remainingMs: 0,
        endsAt: null,
      },
    };
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation((command) => {
      if (command === "timers_list") return Promise.resolve(runs);
      return Promise.resolve(null);
    });
  });

  it("affiche le temps restant de chaque minuteur", async () => {
    render(CountdownList, { tile });

    expect(screen.getByRole("timer", { name: "Temps restant de Thé" }).textContent).toBe("4:00");
    expect(
      (await screen.findByRole("timer", { name: "Temps restant de Pâtes" })).textContent,
    ).toMatch(/^1:0[45]$/);
    expect(screen.getByRole("timer", { name: "Temps restant de Lessive" }).textContent).toBe(
      "Terminé",
    );
  });

  it("démarre, met en pause et remet à zéro", async () => {
    render(CountdownList, { tile });
    await screen.findByRole("button", { name: "Mettre en pause Pâtes" });

    await fireEvent.click(screen.getByRole("button", { name: "Démarrer Thé" }));
    expect(mockedInvoke).toHaveBeenCalledWith("timers_start", {
      key: "timers-1:minuteur-1",
      name: "Thé",
      durationMs: 240_000,
      alert: null,
    });

    await fireEvent.click(screen.getByRole("button", { name: "Mettre en pause Pâtes" }));
    expect(mockedInvoke).toHaveBeenCalledWith("timers_pause", { key: "timers-1:minuteur-2" });

    await fireEvent.click(screen.getByRole("button", { name: "Remettre à zéro Lessive" }));
    expect(mockedInvoke).toHaveBeenCalledWith("timers_reset", { key: "timers-1:minuteur-3" });
  });

  it("formate les durées et calcule le temps restant", () => {
    expect(formatDuration(0)).toBe("0:00");
    expect(formatDuration(4_001)).toBe("0:05");
    expect(formatDuration(3_723_000)).toBe("1:02:03");
    expect(remainingMs(undefined, 5_000, 0)).toBe(5_000);
    expect(
      remainingMs(
        { name: "", status: "paused", durationMs: 9, remainingMs: 3, endsAt: null },
        9,
        100,
      ),
    ).toBe(3);
  });
});
