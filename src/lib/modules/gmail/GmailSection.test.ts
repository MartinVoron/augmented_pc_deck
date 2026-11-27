import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { GUIDE_URL } from ".";
import GmailSection from "./GmailSection.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

let email: string | null;
let finishLogin: (() => void) | null;

beforeEach(() => {
  email = null;
  finishLogin = null;
  mockedInvoke.mockReset();
  mockedInvoke.mockImplementation((command, args) => {
    const a = args as Record<string, unknown> | undefined;
    switch (command) {
      case "gmail_account":
        return Promise.resolve(email ? { email } : null);
      case "gmail_connect":
        if (a?.clientSecret === "mauvais") {
          return Promise.reject({
            code: "invalid_input",
            message: "donnée invalide : l'identifiant OAuth est incorrect",
          });
        }
        // La connexion attend que l'utilisateur ait fini dans le navigateur.
        return new Promise((resolve) => {
          finishLogin = () => resolve({ email: "moi@gmail.com" });
        });
      case "gmail_disconnect":
        email = null;
        return Promise.resolve();
      default:
        return Promise.resolve();
    }
  });
});

async function fill(secret: string) {
  await fireEvent.input(await screen.findByLabelText("ID client"), {
    target: { value: "123.apps.googleusercontent.com" },
  });
  await fireEvent.input(screen.getByLabelText("Code secret du client"), {
    target: { value: secret },
  });
  await fireEvent.click(screen.getByRole("button", { name: "Se connecter avec Google" }));
}

describe("GmailSection", () => {
  it("attend la connexion dans le navigateur, puis affiche le compte", async () => {
    render(GmailSection);

    await fill("GOCSPX-secret");
    expect(mockedInvoke).toHaveBeenCalledWith("gmail_connect", {
      clientId: "123.apps.googleusercontent.com",
      clientSecret: "GOCSPX-secret",
    });
    expect(await screen.findByText(/navigateur qui vient de s'ouvrir/)).toBeTruthy();

    finishLogin!();
    expect(await screen.findByText("moi@gmail.com")).toBeTruthy();
  });

  it("explique un identifiant refusé", async () => {
    render(GmailSection);

    await fill("mauvais");

    expect((await screen.findByRole("alert")).textContent).toContain("identifiant OAuth");
  });

  it("ouvre le guide de création de l'identifiant", async () => {
    render(GmailSection);

    await fireEvent.click(await screen.findByRole("button", { name: "Guide pas à pas" }));

    expect(mockedInvoke).toHaveBeenCalledWith("launcher_launch", { target: GUIDE_URL, args: "" });
  });

  it("déconnecte le compte après confirmation", async () => {
    email = "moi@gmail.com";
    render(GmailSection);

    await fireEvent.click(await screen.findByRole("button", { name: "Se déconnecter" }));
    await fireEvent.click(screen.getByRole("button", { name: "Confirmer" }));

    expect(mockedInvoke).toHaveBeenCalledWith("gmail_disconnect", undefined);
    expect(await screen.findByLabelText("ID client")).toBeTruthy();
  });
});

describe("GmailSection, compte connecté", () => {
  it("se reconnecte avec l'identifiant enregistré", async () => {
    email = "moi@gmail.com";
    render(GmailSection);

    await fireEvent.click(await screen.findByRole("button", { name: "Se reconnecter" }));

    expect(mockedInvoke).toHaveBeenCalledWith("gmail_connect", { clientId: "", clientSecret: "" });
  });
});
