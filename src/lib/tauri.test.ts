import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AppError, call } from "./tauri";

// Les tests tournent sans Tauri : on remplace `invoke` par une fausse fonction.
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

describe("call", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it("renvoie le résultat de la commande", async () => {
    mockedInvoke.mockResolvedValue("0.1.0");

    await expect(call<string>("app_version")).resolves.toBe("0.1.0");
    expect(mockedInvoke).toHaveBeenCalledWith("app_version", undefined);
  });

  it("transforme une erreur du Rust en AppError", async () => {
    mockedInvoke.mockRejectedValue({ code: "not_found", message: "notes.txt est introuvable" });

    const error = await call("launcher_open").catch((e: unknown) => e);

    expect(error).toBeInstanceOf(AppError);
    expect(error).toMatchObject({ code: "not_found", message: "notes.txt est introuvable" });
  });

  it("accepte une erreur inattendue", async () => {
    mockedInvoke.mockRejectedValue("plantage");

    await expect(call("app_version")).rejects.toMatchObject({ code: "other", message: "plantage" });
  });
});
