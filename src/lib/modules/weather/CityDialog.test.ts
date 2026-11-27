import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Config } from "../settings";
import { loadSettings } from "../settings/state.svelte";
import CityDialog from "./CityDialog.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

let saved: Config | null;

const bayeux = { name: "Bayeux", latitude: 49.27, longitude: -0.7, detail: "Normandie, France" };

describe("CityDialog", () => {
  beforeEach(async () => {
    saved = null;
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation((command, args) => {
      switch (command) {
        case "settings_get":
          return Promise.resolve({
            version: 1,
            general: { language: "fr", notificationsEnabled: true },
            window: { alwaysOnTop: false, fullscreen: false },
            layout: {
              pages: [
                {
                  id: "page-1",
                  name: "Principal",
                  tiles: [{ id: "weather-1", kind: "weather", title: "Météo", size: "2x1" }],
                },
              ],
            },
          });
        case "settings_save":
          saved = (args as { config: Config }).config;
          return Promise.resolve(saved);
        case "weather_search":
          return Promise.resolve((args as { name: string }).name === "Bayeux" ? [bayeux] : []);
        default:
          return Promise.resolve();
      }
    });
    await loadSettings();
  });

  it("affiche Caen par défaut, cherche une ville et l'enregistre", async () => {
    const onclose = vi.fn();
    render(CityDialog, { tileId: "weather-1", onclose });
    expect(screen.getByText("Caen")).toBeTruthy();

    await fireEvent.input(screen.getByLabelText("Nom de la ville"), {
      target: { value: "Bayeux" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Rechercher" }));
    await fireEvent.click(await screen.findByRole("button", { name: /Bayeux/ }));

    await vi.waitFor(() => expect(onclose).toHaveBeenCalled());
    expect(saved?.layout.pages[0].tiles[0].location).toEqual(bayeux);
  });

  it("dit quand aucune ville n'est trouvée", async () => {
    render(CityDialog, { tileId: "weather-1", onclose: () => {} });
    await fireEvent.input(screen.getByLabelText("Nom de la ville"), {
      target: { value: "Zzzz" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Rechercher" }));
    expect(await screen.findByText(/Aucune ville trouvée/)).toBeTruthy();
  });
});
