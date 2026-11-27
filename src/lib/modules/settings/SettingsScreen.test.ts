import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Config } from ".";
import SettingsScreen from "./SettingsScreen.svelte";
import { loadSettings } from "./state.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

function makeConfig(): Config {
  return {
    version: 1,
    general: { language: "fr", notificationsEnabled: true },
    window: { alwaysOnTop: false, fullscreen: false },
    layout: { pages: [] },
  };
}

// Faux Rust : répond selon la commande appelée.
function fakeBackend(command: string, args?: Record<string, unknown>): Promise<unknown> {
  switch (command) {
    case "settings_get":
      return Promise.resolve(makeConfig());
    case "settings_save":
      return Promise.resolve(args?.config);
    case "window_autostart_get":
      return Promise.resolve(false);
    case "window_autostart_set":
      return Promise.resolve();
    case "claude_status":
      return Promise.resolve({
        server: { state: "listening" },
        port: 47821,
        scriptPath: "C:\\deck\\claude-hook.ps1",
        scriptInstalled: false,
        settingsSnippet: "{}",
        settingsFile: "~/.claude/settings.json",
      });
    default:
      return Promise.reject({ code: "other", message: `commande inconnue : ${command}` });
  }
}

describe("SettingsScreen", () => {
  beforeEach(async () => {
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation((command, args) =>
      fakeBackend(command, args as Record<string, unknown>),
    );
    await loadSettings();
  });

  it("affiche la section Général et une section par module", () => {
    render(SettingsScreen, { onclose: () => {} });

    expect(screen.getByRole("heading", { name: "Général" })).toBeTruthy();
    for (const label of ["Raccourcis", "Minuteurs", "Musique", "GitHub", "Gmail", "Claude"]) {
      expect(screen.getByRole("button", { name: label })).toBeTruthy();
    }
  });

  it("enregistre tout de suite un changement des notifications", async () => {
    render(SettingsScreen, { onclose: () => {} });

    const toggle = screen.getByRole("switch", { name: "Afficher les notifications Windows" });
    expect((toggle as HTMLInputElement).checked).toBe(true);
    await fireEvent.click(toggle);

    expect(mockedInvoke).toHaveBeenCalledWith("settings_save", {
      config: expect.objectContaining({
        general: { language: "fr", notificationsEnabled: false },
        window: { alwaysOnTop: false, fullscreen: false },
      }),
    });
  });

  it("règle la fenêtre et le démarrage avec Windows", async () => {
    render(SettingsScreen, { onclose: () => {} });

    await fireEvent.click(screen.getByRole("switch", { name: "Toujours au premier plan" }));
    expect(mockedInvoke).toHaveBeenCalledWith("settings_save", {
      config: expect.objectContaining({ window: { alwaysOnTop: true, fullscreen: false } }),
    });

    const autostart = screen.getByRole("switch", { name: "Lancer au démarrage de Windows" });
    await vi.waitFor(() => expect((autostart as HTMLInputElement).disabled).toBe(false));
    await fireEvent.click(autostart);
    expect(mockedInvoke).toHaveBeenCalledWith("window_autostart_set", { enabled: true });
  });

  it("affiche la mise en place du hook de Claude", async () => {
    render(SettingsScreen, { onclose: () => {} });

    await fireEvent.click(screen.getByRole("button", { name: "Claude" }));

    expect(await screen.findByText(/127.0.0.1:47821/)).toBeTruthy();
  });

  it("indique qu'un module n'a pas encore de réglage", async () => {
    render(SettingsScreen, { onclose: () => {} });

    await fireEvent.click(screen.getByRole("button", { name: "Minuteurs" }));

    expect(screen.getByText("Aucun réglage pour l'instant.")).toBeTruthy();
  });

  it("se ferme avec le bouton Fermer", async () => {
    const onclose = vi.fn();
    render(SettingsScreen, { onclose });

    await fireEvent.click(screen.getByRole("button", { name: "Fermer les paramètres" }));

    expect(onclose).toHaveBeenCalled();
  });
});
