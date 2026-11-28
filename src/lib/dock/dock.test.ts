import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { getDockStatus, setDockPanel } from ".";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

describe("ancrage", () => {
  beforeEach(() => mockedInvoke.mockReset());

  it("lit l'état de l'ancrage", async () => {
    mockedInvoke.mockResolvedValue({ edge: "top", expanded: false });

    await expect(getDockStatus()).resolves.toEqual({ edge: "top", expanded: false });
    expect(mockedInvoke).toHaveBeenCalledWith("dock_status", undefined);
  });

  it("envoie la taille du panneau", async () => {
    mockedInvoke.mockResolvedValue(undefined);

    await setDockPanel(800, 320);
    expect(mockedInvoke).toHaveBeenCalledWith("dock_set_panel", { width: 800, height: 320 });
  });
});
