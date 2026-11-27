import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { clampRefreshMinutes, reasonLabel, timeAgo, type GithubActivity } from ".";
import type { TileConfig } from "../../grid/defaultLayout";
import GithubTile from "./GithubTile.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const empty = { items: [], total: 0, error: null };

const activity: GithubActivity = {
  reviewRequests: {
    total: 2,
    error: null,
    items: [
      {
        title: "Ajouter le minuteur",
        repo: "o/r",
        number: 12,
        url: "https://github.com/o/r/pull/12",
        updatedAt: new Date(Date.now() - 3 * 3600_000).toISOString(),
        kind: "pullRequest",
        reason: null,
      },
    ],
  },
  myPullRequests: empty,
  assignedIssues: {
    ...empty,
    error: "accès refusé : le jeton GitHub n'a pas les droits nécessaires",
  },
  notifications: {
    total: 1,
    error: null,
    items: [
      {
        title: "Corriger le lanceur",
        repo: "o/r",
        number: 7,
        url: "https://github.com/o/r/pull/7",
        updatedAt: new Date().toISOString(),
        kind: "pullRequest",
        reason: "mention",
      },
    ],
  },
};

function tile(size: TileConfig["size"]): TileConfig {
  return { id: "github-1", kind: "github", title: "GitHub", size };
}

let response: () => Promise<unknown>;

beforeEach(() => {
  response = () => Promise.resolve(activity);
  mockedInvoke.mockReset();
  mockedInvoke.mockImplementation((command) => {
    if (command === "github_activity") return response();
    return Promise.resolve();
  });
});

describe("fonctions d'affichage", () => {
  it("écrit le temps écoulé en court", () => {
    const now = Date.parse("2026-11-27T12:00:00Z");
    expect(timeAgo("2026-11-27T11:59:40Z", now)).toBe("à l'instant");
    expect(timeAgo("2026-11-27T11:55:00Z", now)).toBe("5 min");
    expect(timeAgo("2026-11-27T09:00:00Z", now)).toBe("3 h");
    expect(timeAgo("2026-11-25T12:00:00Z", now)).toBe("2 j");
  });

  it("traduit la raison d'une notification", () => {
    expect(reasonLabel("review_requested")).toBe("relecture demandée");
    expect(reasonLabel("autre_raison")).toBe("autre raison");
    expect(reasonLabel(null)).toBe("");
  });

  it("garde l'intervalle dans les limites", () => {
    expect(clampRefreshMinutes(undefined)).toBe(5);
    expect(clampRefreshMinutes(0.2)).toBe(1);
    expect(clampRefreshMinutes(Number.NaN)).toBe(5);
    expect(clampRefreshMinutes(500)).toBe(60);
    expect(clampRefreshMinutes(2.4)).toBe(2);
  });
});

describe("GithubTile", () => {
  it("affiche les compteurs et la liste choisie", async () => {
    render(GithubTile, { tile: tile("2x2") });

    expect(await screen.findByText("Ajouter le minuteur")).toBeTruthy();
    expect(screen.getByText(/o\/r#12/).textContent).toContain("3 h");

    await fireEvent.click(screen.getByRole("button", { name: /Notifications non lues/ }));
    expect(screen.getByText(/mentionné/)).toBeTruthy();
  });

  it("ouvre un élément dans le navigateur", async () => {
    render(GithubTile, { tile: tile("2x1") });

    await fireEvent.click(await screen.findByText("Ajouter le minuteur"));

    expect(mockedInvoke).toHaveBeenCalledWith("launcher_launch", {
      target: "https://github.com/o/r/pull/12",
      args: "",
    });
  });

  it("montre l'erreur d'une seule liste sans cacher les autres", async () => {
    render(GithubTile, { tile: tile("2x2") });

    await fireEvent.click(await screen.findByRole("button", { name: /Issues assignées/ }));

    expect(screen.getByRole("alert").textContent).toContain("droits nécessaires");
    expect(screen.getByRole("button", { name: /PR à relire/ }).textContent).toContain("2");
  });

  it("en 1×1, un compteur ouvre la page GitHub de la liste", async () => {
    render(GithubTile, { tile: tile("1x1") });

    await fireEvent.click(await screen.findByRole("button", { name: /PR à relire/ }));

    expect(mockedInvoke).toHaveBeenCalledWith("launcher_launch", {
      target: "https://github.com/pulls/review-requested",
      args: "",
    });
  });

  it("invite à se connecter sans compte", async () => {
    response = () =>
      Promise.reject({
        code: "permission_denied",
        message: "accès refusé : aucun compte GitHub connecté (voir les paramètres)",
      });
    render(GithubTile, { tile: tile("2x2") });

    expect(await screen.findByText(/Paramètres › GitHub/)).toBeTruthy();
  });
});
