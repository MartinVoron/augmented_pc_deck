import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Config } from ".";
import AppearanceSection from "./AppearanceSection.svelte";
import { loadSettings } from "./state.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const config: Config = {
  version: 3,
  general: { language: "fr", notificationsEnabled: true },
  window: { alwaysOnTop: false, fullscreen: false },
  layout: { pages: [] },
};

function savedConfig(): Config {
  const call = [...mockedInvoke.mock.calls]
    .reverse()
    .find(([command]) => command === "settings_save");
  return (call![1] as { config: Config }).config;
}

describe("AppearanceSection", () => {
  beforeEach(async () => {
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation((command, args) => {
      if (command === "settings_get") return Promise.resolve(config);
      if (command === "settings_save") return Promise.resolve((args as { config: Config }).config);
      return Promise.resolve();
    });
    await loadSettings();
  });

  it("propose le thème Classique par défaut, sans mode clair ni flou", () => {
    render(AppearanceSection);

    expect((screen.getByRole("radio", { name: "Classique" }) as HTMLInputElement).checked).toBe(
      true,
    );
    const mode = screen.getByRole("combobox", { name: "Clair ou sombre" }) as HTMLSelectElement;
    expect(mode.disabled).toBe(true);
    const blur = screen.getByRole("switch", { name: "Flouter le bureau sous la fenêtre" });
    expect((blur as HTMLInputElement).disabled).toBe(true);
  });

  it("enregistre le thème Verre, puis le mode et le flou", async () => {
    render(AppearanceSection);

    await fireEvent.click(screen.getByRole("radio", { name: "Verre" }));
    expect(savedConfig().appearance).toEqual({
      theme: "glass",
      mode: "system",
      windowEffect: false,
    });

    const mode = screen.getByRole("combobox", { name: "Clair ou sombre" }) as HTMLSelectElement;
    expect(mode.disabled).toBe(false);
    await fireEvent.change(mode, { target: { value: "light" } });
    expect(savedConfig().appearance?.mode).toBe("light");

    await fireEvent.click(
      screen.getByRole("switch", { name: "Flouter le bureau sous la fenêtre" }),
    );
    expect(savedConfig().appearance).toEqual({ theme: "glass", mode: "light", windowEffect: true });
  });
});
