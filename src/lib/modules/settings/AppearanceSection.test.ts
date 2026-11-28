import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Config } from ".";
import AppearanceSection from "./AppearanceSection.svelte";
import { blurPreview } from "../../appearance/state.svelte";
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
    const blur = screen.getByRole("slider", { name: "Flou des tuiles" }) as HTMLInputElement;
    expect(blur.disabled).toBe(true);
    expect(blur.value).toBe("100");
    expect(screen.queryByRole("switch")).toBeNull();
  });

  it("enregistre le thème Verre, puis le mode", async () => {
    render(AppearanceSection);

    await fireEvent.click(screen.getByRole("radio", { name: "Verre" }));
    expect(savedConfig().appearance).toEqual({ theme: "glass", mode: "system", blur: 100 });

    const mode = screen.getByRole("combobox", { name: "Clair ou sombre" }) as HTMLSelectElement;
    expect(mode.disabled).toBe(false);
    await fireEvent.change(mode, { target: { value: "light" } });
    expect(savedConfig().appearance?.mode).toBe("light");
  });

  it("affiche le flou en direct et l'enregistre au lâcher du curseur", async () => {
    render(AppearanceSection);
    await fireEvent.click(screen.getByRole("radio", { name: "Verre" }));
    const saves = () => mockedInvoke.mock.calls.filter(([c]) => c === "settings_save").length;
    const before = saves();

    const blur = screen.getByRole("slider", { name: "Flou des tuiles" }) as HTMLInputElement;
    expect(blur.disabled).toBe(false);
    await fireEvent.input(blur, { target: { value: "30" } });
    expect(blurPreview.value).toBe(30);
    expect(screen.getByText("30 %")).toBeTruthy();
    expect(saves()).toBe(before);

    await fireEvent.change(blur, { target: { value: "30" } });
    await vi.waitFor(() => expect(blurPreview.value).toBeNull());
    expect(saves()).toBe(before + 1);
    expect(savedConfig().appearance).toEqual({ theme: "glass", mode: "system", blur: 30 });
    expect(screen.getByText("30 %")).toBeTruthy();
  });

  it("descend jusqu'à 0 % (tuiles transparentes)", async () => {
    render(AppearanceSection);
    await fireEvent.click(screen.getByRole("radio", { name: "Verre" }));
    const blur = screen.getByRole("slider", { name: "Flou des tuiles" });
    await fireEvent.change(blur, { target: { value: "0" } });
    await vi.waitFor(() => expect(savedConfig().appearance?.blur).toBe(0));
  });

  it("enregistre le flou même sans lâcher détecté (WebKitGTK)", async () => {
    vi.useFakeTimers();
    try {
      render(AppearanceSection);
      await fireEvent.click(screen.getByRole("radio", { name: "Verre" }));
      const blur = screen.getByRole("slider", { name: "Flou des tuiles" });
      await fireEvent.input(blur, { target: { value: "40" } });
      await fireEvent.input(blur, { target: { value: "20" } });
      await vi.advanceTimersByTimeAsync(500);
      expect(savedConfig().appearance?.blur).toBe(20);
      expect(blurPreview.value).toBeNull();
    } finally {
      vi.useRealTimers();
    }
  });

  it("montre un aperçu du flou avec le thème Verre", async () => {
    const { container } = render(AppearanceSection);
    expect(container.querySelector(".blur-sample")).toBeNull();
    await fireEvent.click(screen.getByRole("radio", { name: "Verre" }));
    expect(container.querySelector(".blur-sample")).not.toBeNull();
  });
});
