import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
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
    expect(root().style.getPropertyValue("--notch-width")).toBe("180px");
    expect(root().style.getPropertyValue("--notch-height")).toBe("32px");

    dock.edge = "right";
    await tick();
    expect(root().style.getPropertyValue("--notch-width")).toBe("32px");
    expect(root().style.getPropertyValue("--notch-height")).toBe("180px");
  });

  it("donne au Rust la taille des tuiles de la page", async () => {
    render(DockedView, { pageId: "home", onselect: () => {} });
    await tick();

    const expected = panelLayout(["1x1", "2x2"], "top");
    expect(mockedInvoke).toHaveBeenCalledWith("dock_set_panel", {
      width: expected.width,
      height: expected.height,
    });

    dock.edge = "left";
    await tick();
    const vertical = panelLayout(["1x1", "2x2"], "left");
    expect(mockedInvoke).toHaveBeenLastCalledWith("dock_set_panel", {
      width: vertical.width,
      height: vertical.height,
    });
  });

  it("ouvert, affiche les tuiles et se détache avec le bouton", async () => {
    render(DockedView, { pageId: "home", onselect: () => {} });
    dock.expanded = true;
    await tick();

    expect(root().dataset.expanded).toBe("true");
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
