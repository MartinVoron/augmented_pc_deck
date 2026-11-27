import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { clampRefreshMinutes, inboxUrl, type Inbox } from ".";
import type { TileConfig } from "../../grid/defaultLayout";
import GmailTile from "./GmailTile.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const inbox: Inbox = {
  email: "moi@gmail.com",
  unread: 12,
  messages: [
    {
      id: "m1",
      from: "La Poste",
      subject: "Votre colis arrive demain",
      snippet: "Bonjour, votre colis…",
      receivedAt: Date.now() - 2 * 3600_000,
      url: "https://mail.google.com/mail/u/?authuser=moi@gmail.com#inbox/t1",
    },
  ],
};

function tile(size: TileConfig["size"]): TileConfig {
  return { id: "gmail-1", kind: "gmail", title: "Gmail", size };
}

let response: () => Promise<unknown>;

beforeEach(() => {
  response = () => Promise.resolve(inbox);
  mockedInvoke.mockReset();
  mockedInvoke.mockImplementation((command) =>
    command === "gmail_unread" ? response() : Promise.resolve(),
  );
});

describe("fonctions", () => {
  it("ouvre la boîte de réception du bon compte", () => {
    expect(inboxUrl("moi@gmail.com")).toBe(
      "https://mail.google.com/mail/u/?authuser=moi%40gmail.com#inbox",
    );
  });

  it("garde l'intervalle dans les limites", () => {
    expect(clampRefreshMinutes(undefined)).toBe(2);
    expect(clampRefreshMinutes(120)).toBe(60);
  });
});

describe("GmailTile", () => {
  it("affiche le nombre de non lus et les derniers messages", async () => {
    render(GmailTile, { tile: tile("2x2") });

    expect(await screen.findByText("Votre colis arrive demain")).toBeTruthy();
    expect(screen.getByText("La Poste")).toBeTruthy();
    expect(screen.getByText("2 h")).toBeTruthy();
    expect(screen.getByText("12")).toBeTruthy();
    expect(mockedInvoke).toHaveBeenCalledWith("gmail_unread", { notify: true });
  });

  it("ouvre un message dans Gmail", async () => {
    render(GmailTile, { tile: tile("2x1") });

    await fireEvent.click(await screen.findByText("Votre colis arrive demain"));

    expect(mockedInvoke).toHaveBeenCalledWith("launcher_launch", {
      target: inbox.messages[0].url,
      args: "",
    });
  });

  it("en 1×1, n'affiche que le nombre", async () => {
    render(GmailTile, { tile: tile("1x1") });

    await fireEvent.click(await screen.findByRole("button", { name: /12 non lus/ }));

    expect(screen.queryByText("Votre colis arrive demain")).toBeNull();
    expect(mockedInvoke).toHaveBeenCalledWith("launcher_launch", {
      target: inboxUrl("moi@gmail.com"),
      args: "",
    });
  });

  it("invite à se connecter sans compte", async () => {
    response = () =>
      Promise.reject({
        code: "permission_denied",
        message: "accès refusé : aucun compte Gmail connecté (voir les paramètres)",
      });
    render(GmailTile, { tile: tile("2x2") });

    expect(await screen.findByText(/Paramètres › Gmail/)).toBeTruthy();
  });

  it("garde les messages affichés si une actualisation échoue", async () => {
    render(GmailTile, { tile: tile("2x2") });
    await screen.findByText("Votre colis arrive demain");

    response = () =>
      Promise.reject({
        code: "other",
        message: "Gmail ne répond pas : pas de connexion à Internet",
      });
    await fireEvent.click(screen.getByRole("button", { name: "Actualiser" }));

    expect((await screen.findByRole("alert")).textContent).toContain("pas de connexion");
    expect(screen.getByText("Votre colis arrive demain")).toBeTruthy();
  });
});
