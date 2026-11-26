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
    layout: { tiles: [] },
  };
}

let tokens: Record<string, boolean>;

// Faux Rust : répond selon la commande appelée.
function fakeBackend(command: string, args?: Record<string, unknown>): Promise<unknown> {
  switch (command) {
    case "settings_get":
      return Promise.resolve(makeConfig());
    case "settings_save":
      return Promise.resolve(args?.config);
    case "tokens_status":
      return Promise.resolve({ ...tokens });
    case "tokens_delete":
      tokens[args?.kind as string] = false;
      return Promise.resolve();
    default:
      return Promise.reject({ code: "other", message: `commande inconnue : ${command}` });
  }
}

describe("SettingsScreen", () => {
  beforeEach(async () => {
    tokens = { github: true, gmail: false, claude: false };
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
      }),
    });
  });

  it("supprime un jeton après confirmation", async () => {
    render(SettingsScreen, { onclose: () => {} });

    await fireEvent.click(screen.getByRole("button", { name: "GitHub" }));
    await fireEvent.click(await screen.findByRole("button", { name: "Supprimer le jeton" }));
    await fireEvent.click(screen.getByRole("button", { name: "Confirmer la suppression" }));

    expect(mockedInvoke).toHaveBeenCalledWith("tokens_delete", { kind: "github" });
    expect(await screen.findByText("Aucun jeton GitHub enregistré.")).toBeTruthy();
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
