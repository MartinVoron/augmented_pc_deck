import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { deleteToken, getTokenStatus, saveToken } from "./tokens";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

describe("tokens", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it("lit l'état des jetons", async () => {
    mockedInvoke.mockResolvedValue({ github: true, gmail: false, claude: false });

    await expect(getTokenStatus()).resolves.toEqual({ github: true, gmail: false, claude: false });
    expect(mockedInvoke).toHaveBeenCalledWith("tokens_status", undefined);
  });

  it("enregistre puis supprime un jeton", async () => {
    mockedInvoke.mockResolvedValue(undefined);

    await saveToken("github", "ghp_exemple");
    await deleteToken("github");

    expect(mockedInvoke).toHaveBeenNthCalledWith(1, "tokens_set", {
      kind: "github",
      value: "ghp_exemple",
    });
    expect(mockedInvoke).toHaveBeenNthCalledWith(2, "tokens_delete", { kind: "github" });
  });
});
