import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { TileConfig } from "../../grid/defaultLayout";
import TimersTile from "./TimersTile.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => {}) }));
const mockedInvoke = vi.mocked(invoke);

const tile: TileConfig = {
  id: "timers-9",
  kind: "timers",
  title: "Minuteurs",
  size: "2x2",
  pomodoro: { workMins: 50, shortBreakMins: 10, longBreakMins: 20, sessionsBeforeLongBreak: 3 },
};

describe("TimersTile", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation((command) =>
      Promise.resolve(command === "timers_list" ? {} : null),
    );
  });

  it("passe d'un onglet à l'autre", async () => {
    render(TimersTile, { tile });
    expect(screen.getByRole("tab", { name: "Minuteurs" }).getAttribute("aria-selected")).toBe(
      "true",
    );

    await fireEvent.click(screen.getByRole("tab", { name: "Chrono" }));
    expect(screen.getByRole("tabpanel", { name: "Chrono" }).hidden).toBe(false);
    expect(screen.getByRole("timer", { name: "Temps du chronomètre" }).textContent?.trim()).toBe(
      "0:00,0",
    );
  });

  it("démarre une séance de travail Pomodoro avec les durées de la tuile", async () => {
    render(TimersTile, { tile });
    await fireEvent.click(screen.getByRole("tab", { name: "Pomodoro" }));

    expect(
      screen.getByRole("timer", { name: "Temps restant du Pomodoro" }).textContent?.trim(),
    ).toBe("50:00");
    await fireEvent.click(screen.getByRole("button", { name: "Démarrer" }));
    expect(mockedInvoke).toHaveBeenCalledWith("timers_start", {
      key: "timers-9:pomodoro",
      name: "Travail",
      durationMs: 50 * 60_000,
      alert: { title: "Séance de travail terminée", body: "Place à la pause (10 min)." },
    });
  });
});
