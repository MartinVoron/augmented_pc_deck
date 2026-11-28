import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Config } from "../modules/settings";
import { loadSettings, settings } from "../modules/settings/state.svelte";
import { dock, watchDock } from "./state.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

/** Rappel donné à `listen` : il reçoit les événements du Rust. */
let handler: ((event: { payload: unknown }) => void) | undefined;
vi.mock("@tauri-apps/api/event", () => ({
  listen: (_name: string, callback: (event: { payload: unknown }) => void) => {
    handler = callback;
    return Promise.resolve(() => {});
  },
}));

function makeConfig(edge: "none" | "top"): Config {
  return {
    version: 3,
    general: { language: "fr", notificationsEnabled: true },
    window: { alwaysOnTop: false, fullscreen: false, dock: edge },
    layout: { pages: [] },
  };
}

let stored: Config;

describe("watchDock", () => {
  beforeEach(async () => {
    stored = makeConfig("none");
    handler = undefined;
    dock.edge = "none";
    dock.expanded = false;
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation((command) => {
      if (command === "settings_get") return Promise.resolve(stored);
      if (command === "dock_status") return Promise.resolve({ edge: "none", expanded: false });
      return Promise.resolve();
    });
    await loadSettings();
  });

  it("suit l'ouverture et la fermeture de l'encoche", async () => {
    const stop = watchDock();
    await vi.waitFor(() => expect(handler).toBeDefined());

    handler?.({ payload: { edge: "top", expanded: true } });
    expect(dock).toEqual({ edge: "top", expanded: true });
    handler?.({ payload: { edge: "top", expanded: false } });
    expect(dock.expanded).toBe(false);
    stop();
  });

  it("relit la configuration quand le Rust ancre la fenêtre lui-même", async () => {
    const stop = watchDock();
    await vi.waitFor(() => expect(handler).toBeDefined());

    // La fenêtre a été lâchée contre le bord haut : le Rust l'a enregistré.
    stored = makeConfig("top");
    handler?.({ payload: { edge: "top", expanded: false } });

    await vi.waitFor(() => expect(settings.config?.window.dock).toBe("top"));
    stop();
  });

  it("ignore un état illisible", async () => {
    const stop = watchDock();
    await vi.waitFor(() => expect(handler).toBeDefined());

    handler?.({ payload: "n'importe quoi" });
    expect(dock.edge).toBe("none");
    stop();
  });
});
