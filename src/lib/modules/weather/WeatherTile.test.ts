import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe as group, expect, it, vi } from "vitest";
import { dayLabel, DEFAULT_PLACE, describe, formatTemperature, type Forecast } from ".";
import type { TileConfig } from "../../grid/defaultLayout";
import WeatherTile from "./WeatherTile.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const tile: TileConfig = { id: "weather-1", kind: "weather", title: "Météo", size: "2x2" };
const forecast: Forecast = {
  current: { temperature: 9.4, feelsLike: 6.8, code: 61, isDay: true, wind: 21 },
  days: [
    { date: "2026-11-27", code: 61, min: 5.1, max: 10.2 },
    { date: "2026-11-28", code: 3, min: 4.3, max: 11 },
    { date: "2026-11-29", code: 0, min: -0.4, max: 8 },
  ],
};

group("météo", () => {
  it("décrit les codes météo en français", () => {
    expect(describe(0)).toEqual({ label: "Ciel dégagé", icon: "clear" });
    expect(describe(63).label).toBe("Pluie");
    expect(describe(75).icon).toBe("snow");
    expect(describe(99).label).toBe("Orage et grêle");
    expect(describe(1234).label).toBe("Temps inconnu");
  });

  it("arrondit les températures sans « -0° »", () => {
    expect(formatTemperature(9.4)).toBe("9°");
    expect(formatTemperature(-0.4)).toBe("0°");
    expect(formatTemperature(-2.6)).toBe("-3°");
  });

  it("nomme les jours", () => {
    expect(dayLabel("2026-11-27", 0)).toBe("Auj.");
    expect(dayLabel("2026-11-28", 1)).toBe("sam.");
  });
});

group("WeatherTile", () => {
  // Accolades : une fonction renvoyée par beforeEach serait appelée après le test.
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it("affiche la météo de Caen par défaut, avec les prochains jours", async () => {
    mockedInvoke.mockResolvedValue(forecast);
    render(WeatherTile, { tile });

    expect(await screen.findByText("9°")).toBeTruthy();
    expect(mockedInvoke).toHaveBeenCalledWith("weather_forecast", {
      latitude: DEFAULT_PLACE.latitude,
      longitude: DEFAULT_PLACE.longitude,
    });
    expect(screen.getByText("Pluie faible")).toBeTruthy();
    expect(screen.getByText("Caen")).toBeTruthy();
    const days = screen.getByRole("list", { name: "Prochains jours" });
    expect(days.textContent).toContain("sam.");
    expect(days.textContent).toContain("0° / 8°");
  });

  it("utilise la ville de la tuile et affiche une erreur réseau", async () => {
    mockedInvoke.mockRejectedValue({ code: "other", message: "Le service météo ne répond pas" });
    const place = { name: "Bayeux", latitude: 49.27, longitude: -0.7, detail: "" };
    render(WeatherTile, { tile: { ...tile, size: "1x1", location: place } });

    expect((await screen.findByRole("alert")).textContent).toContain("ne répond pas");
    expect(mockedInvoke).toHaveBeenCalledWith("weather_forecast", {
      latitude: 49.27,
      longitude: -0.7,
    });
  });

  it("actualise à la demande", async () => {
    mockedInvoke.mockResolvedValue(forecast);
    render(WeatherTile, { tile: { ...tile, size: "2x1" } });
    await screen.findByText("9°");
    await fireEvent.click(screen.getByRole("button", { name: "Actualiser la météo" }));
    expect(mockedInvoke).toHaveBeenCalledTimes(2);
  });
});
