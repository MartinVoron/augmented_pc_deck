import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { TileConfig } from "../grid/defaultLayout";
import type { Config } from "../modules/settings";
import { loadSettings } from "../modules/settings/state.svelte";
import Dashboard from "./Dashboard.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const startTiles: TileConfig[] = [
  { id: "timers-1", kind: "timers", title: "Minuteurs", size: "1x1" },
  { id: "music-1", kind: "music", title: "Musique", size: "2x2" },
];

let saved: Config | null;

/** Titres des tuiles, dans l'ordre de la grille. */
function tileTitles(): string[] {
  return screen.getAllByRole("region").map((tile) => tile.getAttribute("aria-label") ?? "");
}

/** Dernière disposition envoyée au Rust. */
function savedTiles(): TileConfig[] {
  return (saved?.layout.tiles ?? []) as TileConfig[];
}

describe("Dashboard", () => {
  beforeEach(async () => {
    saved = null;
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation((command, args) => {
      if (command === "settings_get") {
        return Promise.resolve({
          version: 1,
          general: { language: "fr", notificationsEnabled: true },
          layout: { tiles: startTiles },
        });
      }
      if (command === "settings_save") {
        saved = (args as { config: Config }).config;
        return Promise.resolve(saved);
      }
      return Promise.reject({ code: "other", message: command });
    });
    await loadSettings();
  });

  it("affiche la disposition enregistrée, sans outils hors du mode édition", () => {
    render(Dashboard);

    expect(tileTitles()).toEqual(["Minuteurs", "Musique"]);
    expect(screen.queryByRole("button", { name: /Supprimer/ })).toBeNull();
  });

  it("ajoute une tuile et l'enregistre", async () => {
    render(Dashboard, { editing: true });

    await fireEvent.change(screen.getByRole("combobox", { name: "Module de la nouvelle tuile" }), {
      target: { value: "gmail" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Ajouter" }));

    expect(savedTiles().map((t) => t.id)).toEqual(["timers-1", "music-1", "gmail-1"]);
    expect(await screen.findByRole("region", { name: "Gmail" })).toBeTruthy();
  });

  it("supprime une tuile", async () => {
    render(Dashboard, { editing: true });

    await fireEvent.click(screen.getByRole("button", { name: "Supprimer Minuteurs" }));

    expect(savedTiles().map((t) => t.id)).toEqual(["music-1"]);
  });

  it("déplace une tuile avec les flèches", async () => {
    render(Dashboard, { editing: true });

    await fireEvent.click(screen.getByRole("button", { name: "Déplacer Minuteurs après" }));

    expect(savedTiles().map((t) => t.id)).toEqual(["music-1", "timers-1"]);
    expect(tileTitles()).toEqual(["Musique", "Minuteurs"]);
  });

  it("déplace une tuile par glisser-déposer", async () => {
    render(Dashboard, { editing: true });
    const [timers, music] = screen.getAllByRole("region");

    await fireEvent.dragStart(music);
    await fireEvent.dragOver(timers);
    await fireEvent.drop(timers);

    expect(savedTiles().map((t) => t.id)).toEqual(["music-1", "timers-1"]);
  });

  it("redimensionne une tuile", async () => {
    render(Dashboard, { editing: true });
    const timers = screen.getByRole("region", { name: "Minuteurs" });

    await fireEvent.change(within(timers).getByRole("combobox", { name: "Taille de Minuteurs" }), {
      target: { value: "2x1" },
    });

    expect(savedTiles()[0].size).toBe("2x1");
    expect(timers.dataset.size).toBe("2x1");
  });
});
