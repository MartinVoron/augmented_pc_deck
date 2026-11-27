import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../../App.svelte";
import type { Config, PageSettings } from "../modules/settings";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

let saved: Config | null;
let current: Config;

const startPages: PageSettings[] = [
  {
    id: "page-1",
    name: "Principal",
    tiles: [{ id: "timers-1", kind: "timers", title: "Minuteurs", size: "1x1" }],
  },
  {
    id: "page-2",
    name: "Musique",
    tiles: [{ id: "clock-1", kind: "clock", title: "Horloge", size: "2x1" }],
  },
];

function savedPages(): PageSettings[] {
  return saved?.layout.pages ?? [];
}

function tileTitles(): string[] {
  return screen.getAllByRole("region").map((tile) => tile.getAttribute("aria-label") ?? "");
}

describe("pages de tuiles", () => {
  beforeEach(() => {
    saved = null;
    current = {
      version: 2,
      general: { language: "fr", notificationsEnabled: true },
      window: { alwaysOnTop: false, fullscreen: false },
      layout: { pages: structuredClone(startPages) },
    };
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation((command, args) => {
      switch (command) {
        case "settings_get":
          return Promise.resolve(current);
        case "settings_save":
          saved = (args as { config: Config }).config;
          current = saved;
          return Promise.resolve(saved);
        case "app_version":
          return Promise.resolve("0.1.0");
        case "timers_list":
          return Promise.resolve([]);
        default:
          return Promise.resolve(null);
      }
    });
  });

  async function openApp() {
    render(App);
    await screen.findByText("Augmented PC Deck 0.1.0");
  }

  it("passe d'une page à l'autre avec les onglets", async () => {
    await openApp();
    expect(tileTitles()).toEqual(["Minuteurs"]);

    await fireEvent.click(screen.getByRole("tab", { name: "Musique" }));
    expect(tileTitles()).toEqual(["Horloge"]);
    expect(screen.getByRole("tab", { name: "Musique" }).getAttribute("aria-selected")).toBe("true");

    await fireEvent.keyDown(screen.getByRole("tablist", { name: "Pages de tuiles" }), {
      key: "ArrowLeft",
    });
    expect(tileTitles()).toEqual(["Minuteurs"]);
  });

  it("crée une page à partir d'un modèle et l'affiche", async () => {
    await openApp();
    await fireEvent.click(screen.getByRole("button", { name: /Modifier la disposition/ }));
    await fireEvent.click(screen.getByRole("button", { name: "Nouvelle page" }));

    await fireEvent.click(screen.getByRole("radio", { name: /Détente/ }));
    await fireEvent.click(screen.getByRole("button", { name: "Créer la page" }));

    await vi.waitFor(() => expect(savedPages()).toHaveLength(3));
    const page = savedPages()[2];
    expect(page.name).toBe("Détente");
    // L'horloge existe déjà sur une autre page : la nouvelle a un autre identifiant.
    expect(page.tiles.map((t) => t.id)).toEqual(["music-1", "clock-2", "weather-1"]);
    expect(screen.getByRole("tab", { name: /Détente/ }).getAttribute("aria-selected")).toBe("true");
  });

  it("renomme, déplace et supprime une page", async () => {
    await openApp();
    await fireEvent.click(screen.getByRole("button", { name: /Modifier la disposition/ }));
    await fireEvent.click(screen.getByRole("tab", { name: /Musique/ }));

    await fireEvent.click(screen.getByRole("button", { name: "Renommer la page Musique" }));
    await fireEvent.input(screen.getByRole("textbox", { name: "Nom de la page" }), {
      target: { value: "Soirée" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Valider le nom" }));
    await vi.waitFor(() => expect(savedPages()[1].name).toBe("Soirée"));

    await fireEvent.click(screen.getByRole("button", { name: "Déplacer la page Soirée à gauche" }));
    await vi.waitFor(() => expect(savedPages().map((p) => p.id)).toEqual(["page-2", "page-1"]));

    await fireEvent.click(screen.getByRole("button", { name: "Supprimer la page Soirée" }));
    expect(screen.getByText(/Supprimer « Soirée » et sa tuile/)).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Supprimer" }));
    await vi.waitFor(() => expect(savedPages().map((p) => p.id)).toEqual(["page-1"]));
    expect(screen.getByRole("button", { name: "Supprimer la page Principal" })).toHaveProperty(
      "disabled",
      true,
    );
  });

  it("s'ouvre sur l'écran d'accueil, qui ne peut pas être supprimé", async () => {
    current.layout.pages = [
      startPages[0],
      {
        id: "accueil",
        name: "Accueil",
        home: true,
        tiles: [{ id: "clock-2", kind: "clock", title: "Mon horloge", size: "2x1" }],
      },
    ];
    await openApp();
    await vi.waitFor(() => expect(tileTitles()).toEqual(["Mon horloge"]));
    expect(screen.getByRole("tab", { name: "Accueil" }).getAttribute("aria-selected")).toBe("true");

    await fireEvent.click(screen.getByRole("tab", { name: "Principal" }));
    expect(tileTitles()).toEqual(["Minuteurs"]);
    await fireEvent.click(screen.getByRole("tab", { name: "Accueil" }));
    expect(tileTitles()).toEqual(["Mon horloge"]);

    await fireEvent.click(screen.getByRole("button", { name: /Modifier la disposition/ }));
    const remove = screen.getByRole("button", { name: "Supprimer la page Accueil" });
    expect(remove).toHaveProperty("disabled", true);
    expect(remove.getAttribute("title")).toBe("L'écran d'accueil ne peut pas être supprimé");
  });

  it("cache les onglets quand il n'y a qu'une page", async () => {
    current.layout.pages = [startPages[0]];
    await openApp();
    await vi.waitFor(() =>
      expect(screen.queryByRole("tablist", { name: "Pages de tuiles" })).toBeNull(),
    );
  });
});
