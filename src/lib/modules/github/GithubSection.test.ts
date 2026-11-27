import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { missingScopes, NEW_TOKEN_URL, type GithubAccount } from ".";
import GithubSection from "./GithubSection.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const octocat: GithubAccount = {
  login: "octocat",
  name: "The Octocat",
  avatarUrl: "https://avatars.githubusercontent.com/u/1",
  htmlUrl: "https://github.com/octocat",
  scopes: ["repo", "notifications"],
};

let connected: GithubAccount | null;

beforeEach(() => {
  connected = null;
  mockedInvoke.mockReset();
  mockedInvoke.mockImplementation((command, args) => {
    const a = args as Record<string, unknown> | undefined;
    switch (command) {
      case "github_account":
        return Promise.resolve(connected);
      case "github_connect":
        if (a?.token === "ghp_bon") {
          connected = octocat;
          return Promise.resolve(octocat);
        }
        return Promise.reject({
          code: "permission_denied",
          message: "accès refusé : GitHub refuse le jeton : il est incorrect, révoqué ou expiré",
        });
      case "github_disconnect":
        connected = null;
        return Promise.resolve();
      case "launcher_launch":
        return Promise.resolve();
      default:
        return Promise.reject({ code: "other", message: command });
    }
  });
});

describe("missingScopes", () => {
  it("liste les droits manquants d'un jeton classique", () => {
    expect(missingScopes({ ...octocat, scopes: ["repo"] })).toEqual(["notifications"]);
    expect(missingScopes(octocat)).toEqual([]);
  });

  it("ne dit rien pour un jeton à accès précis", () => {
    expect(missingScopes({ ...octocat, scopes: null })).toEqual([]);
  });
});

describe("GithubSection", () => {
  it("connecte un compte avec un jeton accepté", async () => {
    render(GithubSection);

    const input = await screen.findByLabelText("Jeton personnel");
    await fireEvent.input(input, { target: { value: "ghp_bon" } });
    await fireEvent.click(screen.getByRole("button", { name: "Se connecter" }));

    expect(mockedInvoke).toHaveBeenCalledWith("github_connect", { token: "ghp_bon" });
    expect(await screen.findByText("Connecté en tant que @octocat")).toBeTruthy();
  });

  it("explique un jeton refusé", async () => {
    render(GithubSection);

    const input = await screen.findByLabelText("Jeton personnel");
    await fireEvent.input(input, { target: { value: "ghp_faux" } });
    await fireEvent.click(screen.getByRole("button", { name: "Se connecter" }));

    expect((await screen.findByRole("alert")).textContent).toContain("refuse le jeton");
  });

  it("ouvre la page de création d'un jeton", async () => {
    render(GithubSection);

    await fireEvent.click(await screen.findByRole("button", { name: "Créer un jeton sur GitHub" }));

    expect(mockedInvoke).toHaveBeenCalledWith("launcher_launch", {
      target: NEW_TOKEN_URL,
      args: "",
    });
  });

  it("déconnecte le compte après confirmation", async () => {
    connected = octocat;
    render(GithubSection);

    await fireEvent.click(await screen.findByRole("button", { name: "Se déconnecter" }));
    await fireEvent.click(screen.getByRole("button", { name: "Confirmer" }));

    expect(mockedInvoke).toHaveBeenCalledWith("github_disconnect", undefined);
    expect(await screen.findByLabelText("Jeton personnel")).toBeTruthy();
  });

  it("signale les droits manquants", async () => {
    connected = { ...octocat, scopes: ["repo"] };
    render(GithubSection);

    expect((await screen.findByRole("status")).textContent).toContain("« notifications »");
  });
});
