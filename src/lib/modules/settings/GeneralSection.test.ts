import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Config } from ".";
import GeneralSection from "./GeneralSection.svelte";
import { loadSettings } from "./state.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const config: Config = {
  version: 3,
  general: { language: "fr", notificationsEnabled: true },
  window: { alwaysOnTop: false, fullscreen: false },
  layout: { pages: [] },
};

describe("GeneralSection : encoche", () => {
  beforeEach(async () => {
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation((command, args) => {
      if (command === "settings_get") return Promise.resolve(config);
      if (command === "settings_save") return Promise.resolve((args as { config: Config }).config);
      if (command === "window_autostart_get") return Promise.resolve(false);
      return Promise.resolve();
    });
    await loadSettings();
  });

  it("propose une fenêtre normale par défaut", () => {
    render(GeneralSection);

    const edge = screen.getByRole("combobox", { name: "Ancrer au bord de l'écran" });
    expect((edge as HTMLSelectElement).value).toBe("none");
    const style = screen.getByRole("combobox", { name: "Encoche repliée" });
    expect((style as HTMLSelectElement).value).toBe("logo");
  });

  it("ancre la fenêtre au bord choisi", async () => {
    render(GeneralSection);

    await fireEvent.change(screen.getByRole("combobox", { name: "Ancrer au bord de l'écran" }), {
      target: { value: "left" },
    });

    expect(mockedInvoke).toHaveBeenCalledWith("settings_save", {
      config: expect.objectContaining({
        window: { alwaysOnTop: false, fullscreen: false, dock: "left" },
      }),
    });
  });

  it("enregistre l'apparence de l'encoche", async () => {
    render(GeneralSection);

    await fireEvent.change(screen.getByRole("combobox", { name: "Encoche repliée" }), {
      target: { value: "black" },
    });

    expect(mockedInvoke).toHaveBeenCalledWith("settings_save", {
      config: expect.objectContaining({
        window: { alwaysOnTop: false, fullscreen: false, notchStyle: "black" },
      }),
    });
  });
});
