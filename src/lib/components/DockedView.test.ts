import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { NOTCH_ACTIVE_LENGTH, NOTCH_DEPTH } from "../dock";
import { foldedSize, windowForPanel } from "../dock/island";
import { panelLayout } from "../dock/panel";
import { dock } from "../dock/state.svelte";
import type { TileConfig } from "../grid/defaultLayout";
import type { Config } from "../modules/settings";
import { loadSettings } from "../modules/settings/state.svelte";
import DockedView from "./DockedView.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const tiles: TileConfig[] = [
  { id: "clock-1", kind: "clock", title: "Horloge", size: "1x1" },
  { id: "todos-1", kind: "todos", title: "Tâches", size: "2x2" },
];

function makeConfig(notchStyle: "logo" | "black" = "logo"): Config {
  return {
    version: 3,
    general: { language: "fr", notificationsEnabled: true },
    window: { alwaysOnTop: false, fullscreen: false, dock: "top", notchStyle },
    layout: { pages: [{ id: "home", name: "Accueil", home: true, tiles }] },
  };
}

let config: Config;

function root(): HTMLElement {
  return document.querySelector(".dock") as HTMLElement;
}

describe("DockedView", () => {
  beforeEach(async () => {
    config = makeConfig();
    dock.edge = "top";
    dock.expanded = false;
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation((command, args) => {
      if (command === "settings_get") return Promise.resolve(config);
      if (command === "settings_save") return Promise.resolve((args as { config: Config }).config);
      return Promise.resolve();
    });
    await loadSettings();
  });

  it("replié, n'affiche que l'encoche avec le logo", () => {
    render(DockedView, { pageId: "home", onselect: () => {} });

    expect(root().dataset.edge).toBe("top");
    expect(root().dataset.expanded).toBeUndefined();
    expect(document.querySelector(".notch img")).toBeTruthy();
    // Le panneau est là pour l'animation, mais inaccessible tant qu'il est replié.
    expect(document.querySelector(".panel")?.getAttribute("aria-hidden")).toBe("true");
    expect(screen.queryByRole("button", { name: /Détacher/ })).toBeNull();
  });

  it("l'encoche noire n'a pas de logo", async () => {
    config = makeConfig("black");
    await loadSettings();
    render(DockedView, { pageId: "home", onselect: () => {} });

    expect(document.querySelector(".notch img")).toBeNull();
  });

  it("couchée en haut, debout sur les côtés", async () => {
    render(DockedView, { pageId: "home", onselect: () => {} });
    const island = () => document.querySelector(".island") as HTMLElement;
    expect(island().dataset.foldedWidth).toBe(String(foldedSize("top").width));
    expect(island().style.height).toBe(`${NOTCH_DEPTH}px`);
    expect(document.querySelectorAll(".ear")).toHaveLength(2);

    dock.edge = "right";
    await tick();
    expect(island().dataset.foldedHeight).toBe(String(foldedSize("right").height));
    expect(island().style.width).toBe(`${NOTCH_DEPTH}px`);
  });

  it("donne au Rust la taille des tuiles de la page", async () => {
    render(DockedView, { pageId: "home", onselect: () => {} });
    await tick();

    // Le panneau, plus les oreilles et la marge du rebond.
    const expected = windowForPanel("top", panelLayout(["1x1", "2x2"], "top"));
    expect(mockedInvoke).toHaveBeenCalledWith("dock_set_panel", {
      width: expected.width,
      height: expected.height,
    });

    dock.edge = "left";
    await tick();
    const vertical = windowForPanel("left", panelLayout(["1x1", "2x2"], "left"));
    expect(mockedInvoke).toHaveBeenLastCalledWith("dock_set_panel", {
      width: vertical.width,
      height: vertical.height,
    });
  });

  it("montre le morceau en cours dans l'encoche repliée", async () => {
    mockedInvoke.mockImplementation((command, args) => {
      if (command === "settings_get") return Promise.resolve(config);
      if (command === "settings_save") return Promise.resolve((args as { config: Config }).config);
      if (command === "music_now_playing") {
        return Promise.resolve({
          title: "Clair de lune",
          artist: "Debussy",
          album: "",
          source: "Spotify",
          status: "playing",
          positionMs: 0,
          durationMs: 300000,
          updatedAt: 0,
          thumbnail: null,
          canPlayPause: true,
          canNext: true,
          canPrevious: true,
        });
      }
      return Promise.resolve();
    });
    render(DockedView, { pageId: "home", onselect: () => {} });

    expect(
      await screen.findByRole("img", { name: "Lecture en cours : Clair de lune" }),
    ).toBeTruthy();
    // L'encoche s'allonge pour la pochette, le titre et l'égaliseur.
    await vi.waitFor(() =>
      expect(mockedInvoke).toHaveBeenCalledWith("dock_set_notch", { length: NOTCH_ACTIVE_LENGTH }),
    );
    expect(document.querySelector(".notch img[src='/favicon.png']")).toBeNull();
  });

  it("montre le minuteur qui tourne dans l'encoche repliée", async () => {
    mockedInvoke.mockImplementation((command, args) => {
      if (command === "settings_get") return Promise.resolve(config);
      if (command === "settings_save") return Promise.resolve((args as { config: Config }).config);
      if (command === "timers_list") {
        return Promise.resolve({
          "t:1": {
            name: "Thé",
            status: "running",
            durationMs: 180000,
            remainingMs: 180000,
            endsAt: Date.now() + 125000,
          },
        });
      }
      return Promise.resolve(null);
    });
    render(DockedView, { pageId: "home", onselect: () => {} });

    const timer = await screen.findByRole("timer", { name: "Thé : temps restant" });
    expect(timer.textContent?.trim()).toMatch(/^2:0[4-6]$/);
  });

  it("ouvert, affiche les tuiles et se détache avec le bouton", async () => {
    render(DockedView, { pageId: "home", onselect: () => {} });
    dock.expanded = true;
    await tick();

    expect(root().dataset.expanded).toBe("true");
    // Le ressort étire l'encoche jusqu'au panneau.
    await vi.waitFor(() =>
      expect(
        parseFloat((document.querySelector(".island") as HTMLElement).style.height),
      ).toBeGreaterThan(NOTCH_DEPTH * 2),
    );
    expect(screen.getByRole("region", { name: "Horloge" })).toBeTruthy();
    expect(screen.getByRole("region", { name: "Tâches" })).toBeTruthy();

    await fireEvent.click(
      screen.getByRole("button", { name: "Détacher la fenêtre du bord de l'écran" }),
    );
    expect(mockedInvoke).toHaveBeenCalledWith("settings_save", {
      config: expect.objectContaining({
        window: expect.objectContaining({ dock: "none" }),
      }),
    });
  });
});
