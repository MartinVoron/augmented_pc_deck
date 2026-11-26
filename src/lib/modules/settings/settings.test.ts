import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { getSettings, saveSettings, type Config } from ".";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const config: Config = {
  version: 1,
  general: { language: "fr", notificationsEnabled: false },
  layout: { tiles: [{ id: "timers-1", kind: "timers", title: "Minuteurs", size: "1x1" }] },
};

describe("settings", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it("lit la configuration", async () => {
    mockedInvoke.mockResolvedValue(config);

    await expect(getSettings()).resolves.toEqual(config);
    expect(mockedInvoke).toHaveBeenCalledWith("settings_get", undefined);
  });

  it("enregistre la configuration", async () => {
    mockedInvoke.mockResolvedValue(config);

    await saveSettings(config);

    expect(mockedInvoke).toHaveBeenCalledWith("settings_save", { config });
  });
});
