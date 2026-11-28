import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import TitleBar from "./TitleBar.svelte";

// Fausse fenêtre Tauri : on vérifie seulement quelle action est demandée.
const { fakeWindow } = vi.hoisted(() => ({
  fakeWindow: {
    close: vi.fn(() => Promise.resolve()),
    minimize: vi.fn(() => Promise.resolve()),
    toggleMaximize: vi.fn(() => Promise.resolve()),
  },
}));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => fakeWindow }));

// Faux réglages : on regarde la modification demandée.
const { updateSettings } = vi.hoisted(() => ({
  updateSettings: vi.fn((change: (config: { window: { dock?: string } }) => void) => {
    const config = { window: { dock: "none" } };
    change(config);
    return Promise.resolve(config);
  }),
}));
vi.mock("../modules/settings/state.svelte", () => ({ updateSettings }));

describe("TitleBar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("affiche le titre dans une zone qui permet de déplacer la fenêtre", () => {
    render(TitleBar);

    const title = screen.getByText("Augmented PC Deck");
    expect(title.hasAttribute("data-tauri-drag-region")).toBe(true);
    expect(screen.getByRole("banner").hasAttribute("data-tauri-drag-region")).toBe(true);
  });

  it("ferme la fenêtre avec le rond rouge", async () => {
    render(TitleBar);

    await fireEvent.click(screen.getByRole("button", { name: /Fermer/ }));

    expect(fakeWindow.close).toHaveBeenCalled();
  });

  it("réduit et agrandit la fenêtre avec les ronds jaune et vert", async () => {
    render(TitleBar);

    await fireEvent.click(screen.getByRole("button", { name: "Réduire" }));
    await fireEvent.click(screen.getByRole("button", { name: "Agrandir" }));

    expect(fakeWindow.minimize).toHaveBeenCalled();
    expect(fakeWindow.toggleMaximize).toHaveBeenCalled();
  });

  it("ancre la fenêtre en encoche au bord gauche de l'écran", async () => {
    render(TitleBar);

    await fireEvent.click(
      screen.getByRole("button", { name: "Ancrer en encoche à gauche de l'écran" }),
    );

    expect(updateSettings).toHaveBeenCalledTimes(1);
    expect(await updateSettings.mock.results[0].value).toEqual({ window: { dock: "left" } });
  });
});
