import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "./App.svelte";
import { DEFAULT_PAGES } from "./lib/grid/defaultLayout";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const config = {
  version: 1,
  general: { language: "fr", notificationsEnabled: true },
  window: { alwaysOnTop: false, fullscreen: false },
  layout: { pages: DEFAULT_PAGES },
};

// Faux Rust qui répond normalement à chaque commande.
function backendOk(command: string): Promise<unknown> {
  return Promise.resolve(command === "settings_get" ? config : "0.1.0");
}

describe("App", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation(backendOk);
  });

  it("affiche une tuile par élément de la disposition enregistrée", async () => {
    render(App);
    await screen.findByText("Augmented PC Deck 0.1.0");

    for (const tile of DEFAULT_PAGES[0].tiles) {
      expect(screen.getByRole("region", { name: tile.title })).toBeTruthy();
    }
  });

  it("affiche la version renvoyée par le Rust", async () => {
    render(App);

    expect(await screen.findByText("Augmented PC Deck 0.1.0")).toBeTruthy();
  });

  it("ouvre l'écran des paramètres", async () => {
    render(App);
    await screen.findByText("Augmented PC Deck 0.1.0");

    await fireEvent.click(screen.getByRole("button", { name: "Ouvrir les paramètres" }));

    expect(screen.getByRole("heading", { name: "Paramètres" })).toBeTruthy();
    expect(screen.getByRole("switch", { name: "Afficher les notifications" })).toBeTruthy();
  });

  it("affiche un message clair si le Rust ne répond pas", async () => {
    mockedInvoke.mockRejectedValue({ code: "other", message: "pas de réponse" });

    render(App);

    expect(await screen.findByText(/pas joignable : pas de réponse/)).toBeTruthy();
  });

  it("devient une encoche quand la fenêtre est ancrée", async () => {
    mockedInvoke.mockImplementation((command) =>
      command === "dock_status"
        ? Promise.resolve({ edge: "bottom", expanded: false })
        : backendOk(command),
    );

    render(App);

    await vi.waitFor(() =>
      expect(document.querySelector(".dock[data-edge='bottom']")).toBeTruthy(),
    );
    expect(document.documentElement.hasAttribute("data-docked")).toBe(true);
    expect(screen.queryByRole("button", { name: "Ouvrir les paramètres" })).toBeNull();
  });
});
