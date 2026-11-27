import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Config } from "../settings";
import { loadSettings } from "../settings/state.svelte";
import ShortcutsDialog from "./ShortcutsDialog.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({ onDragDropEvent: () => Promise.resolve(() => {}) }),
}));
const mockedInvoke = vi.mocked(invoke);
const mockedOpen = vi.mocked(open);

let saved: Config | null;

function savedShortcuts() {
  return saved?.layout.pages[0].tiles[0].shortcuts ?? [];
}

describe("ShortcutsDialog", () => {
  beforeEach(async () => {
    saved = null;
    mockedInvoke.mockReset();
    mockedOpen.mockReset();
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
                  tiles: [
                    {
                      id: "launcher-1",
                      kind: "launcher",
                      title: "Raccourcis",
                      size: "2x1",
                      shortcuts: [
                        { id: "raccourci-1", name: "Bloc-notes", target: "C:\\notepad.exe" },
                      ],
                    },
                  ],
                },
              ],
            },
          });
        case "settings_save":
          saved = (args as { config: Config }).config;
          return Promise.resolve(saved);
        case "launcher_icon":
          return Promise.resolve(null);
        case "launcher_read_image":
          return Promise.resolve("data:image/png;base64,AQID");
      }
      return Promise.reject({ code: "other", message: command });
    });
    await loadSettings();
  });

  it("ajoute un raccourci vers une URL, avec un nom proposé", async () => {
    render(ShortcutsDialog, { tileId: "launcher-1", onclose: () => {} });

    await fireEvent.click(screen.getByRole("button", { name: "Ajouter un raccourci" }));
    await fireEvent.input(screen.getByLabelText(/Cible/), {
      target: { value: "https://www.github.com" },
    });
    expect(screen.getByLabelText<HTMLInputElement>("Nom").value).toBe("github.com");
    await fireEvent.click(screen.getByRole("button", { name: "Enregistrer" }));

    expect(savedShortcuts()).toEqual([
      { id: "raccourci-1", name: "Bloc-notes", target: "C:\\notepad.exe" },
      { id: "raccourci-2", name: "github.com", target: "https://www.github.com" },
    ]);
  });

  it("choisit la cible avec le sélecteur de fichier", async () => {
    mockedOpen.mockResolvedValue("C:\\Jeux\\Jeu.exe");
    render(ShortcutsDialog, { tileId: "launcher-1", onclose: () => {} });

    await fireEvent.click(screen.getByRole("button", { name: "Ajouter un raccourci" }));
    await fireEvent.click(screen.getByRole("button", { name: "Fichier…" }));

    expect(await screen.findByDisplayValue("C:\\Jeux\\Jeu.exe")).toBeTruthy();
    expect(screen.getByLabelText<HTMLInputElement>("Nom").value).toBe("Jeu");
  });

  it("modifie le nom, les arguments et l'icône d'un raccourci", async () => {
    mockedOpen.mockResolvedValue("C:\\icones\\note.png");
    render(ShortcutsDialog, { tileId: "launcher-1", onclose: () => {} });

    await fireEvent.click(screen.getByRole("button", { name: "Modifier Bloc-notes" }));
    await fireEvent.input(screen.getByLabelText("Nom"), { target: { value: "Notes" } });
    await fireEvent.input(screen.getByLabelText("Arguments de lancement"), {
      target: { value: "todo.txt" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Icône personnalisée…" }));
    await screen.findByRole("button", { name: "Icône d'origine" });
    await fireEvent.click(screen.getByRole("button", { name: "Enregistrer" }));

    expect(savedShortcuts()).toEqual([
      {
        id: "raccourci-1",
        name: "Notes",
        target: "C:\\notepad.exe",
        args: "todo.txt",
        icon: "data:image/png;base64,AQID",
      },
    ]);
  });

  it("supprime un raccourci", async () => {
    render(ShortcutsDialog, { tileId: "launcher-1", onclose: () => {} });

    await fireEvent.click(screen.getByRole("button", { name: "Supprimer Bloc-notes" }));

    expect(savedShortcuts()).toEqual([]);
  });
});
