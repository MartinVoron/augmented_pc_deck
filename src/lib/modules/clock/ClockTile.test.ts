import { render, screen } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { formatDate, formatShortDate, formatTime, nextTickDelay } from ".";
import type { TileConfig } from "../../grid/defaultLayout";
import ClockTile from "./ClockTile.svelte";

const tile: TileConfig = { id: "clock-1", kind: "clock", title: "Horloge", size: "2x1" };
const date = new Date(2026, 10, 27, 14, 5, 9, 250);

describe("horloge", () => {
  it("met l'heure et la date en forme, en français", () => {
    expect(formatTime(date)).toBe("14:05");
    expect(formatTime(date, true)).toBe("14:05:09");
    expect(formatDate(date)).toBe("Vendredi 27 novembre 2026");
    expect(formatShortDate(date)).toBe("ven. 27 nov.");
  });

  it("attend la minute ou la seconde suivante", () => {
    expect(nextTickDelay(date)).toBe(50_750);
    expect(nextTickDelay(date, true)).toBe(750);
  });
});

describe("ClockTile", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(date);
  });
  afterEach(() => vi.useRealTimers());

  it("affiche l'heure et la date complète, puis se met à jour", async () => {
    render(ClockTile, { tile });
    expect(screen.getByText("14:05")).toBeTruthy();
    expect(screen.getByText("Vendredi 27 novembre 2026")).toBeTruthy();

    await vi.advanceTimersByTimeAsync(51_000);
    expect(screen.getByText("14:06")).toBeTruthy();
  });

  it("affiche les secondes si demandé, et une date courte en 1×1", () => {
    render(ClockTile, { tile: { ...tile, size: "1x1", showSeconds: true } });
    expect(screen.getByText("14:05:09")).toBeTruthy();
    expect(screen.getByText("ven. 27 nov.")).toBeTruthy();
  });
});
