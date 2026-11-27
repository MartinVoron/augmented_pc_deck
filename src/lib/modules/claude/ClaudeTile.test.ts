import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { waitingSessions, type ClaudeEvent } from ".";
import type { TileConfig } from "../../grid/defaultLayout";
import ClaudeTile from "./ClaudeTile.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const minute = 60_000;

function event(
  id: number,
  kind: ClaudeEvent["kind"],
  sessionId: string,
  project: string,
  message: string,
  minutesAgo: number,
): ClaudeEvent {
  return { id, kind, sessionId, project, message, receivedAt: Date.now() - minutesAgo * minute };
}

// Du plus récent au plus ancien, comme le Rust.
const events: ClaudeEvent[] = [
  event(4, "waiting", "s1", "augmented_pc_deck", "Autoriser Bash : npm test ?", 1),
  event(3, "done", "s2", "site-perso", "Tâche terminée", 5),
  event(2, "waiting", "s2", "site-perso", "Quelle couleur pour le bouton ?", 20),
  event(1, "done", "s1", "augmented_pc_deck", "Tâche terminée", 90),
];

function tile(size: TileConfig["size"]): TileConfig {
  return { id: "claude-1", kind: "claude", title: "Claude", size };
}

let list: ClaudeEvent[];

beforeEach(() => {
  list = [...events];
  mockedInvoke.mockReset();
  mockedInvoke.mockImplementation((command) => {
    if (command === "claude_events") return Promise.resolve(list);
    if (command === "claude_clear") {
      list = [];
      return Promise.resolve();
    }
    return Promise.resolve();
  });
});

describe("waitingSessions", () => {
  it("garde seulement les sessions dont le dernier événement attend", () => {
    expect(waitingSessions(events).map((e) => e.id)).toEqual([4]);
  });
});

describe("ClaudeTile", () => {
  it("liste les notifications récentes et les sessions en attente", async () => {
    render(ClaudeTile, { tile: tile("2x2") });

    expect(await screen.findByText("Autoriser Bash : npm test ?")).toBeTruthy();
    expect(screen.getByText(/session en attente/).textContent).toContain("1");
    expect(screen.getAllByText("site-perso")).toHaveLength(2);
    expect(screen.getByText("1 h")).toBeTruthy();
  });

  it("efface les notifications", async () => {
    render(ClaudeTile, { tile: tile("2x1") });

    await fireEvent.click(await screen.findByRole("button", { name: "Effacer les notifications" }));

    expect(mockedInvoke).toHaveBeenCalledWith("claude_clear", undefined);
    expect(await screen.findByText(/Aucune notification/)).toBeTruthy();
  });

  it("en 1×1, montre le nombre de sessions en attente", async () => {
    render(ClaudeTile, { tile: tile("1x1") });

    expect(await screen.findByText("attend")).toBeTruthy();
    expect(screen.getByText("augmented_pc_deck")).toBeTruthy();
  });

  it("invite à installer le hook sans notification", async () => {
    list = [];
    render(ClaudeTile, { tile: tile("2x2") });

    expect(await screen.findByText(/Paramètres › Claude/)).toBeTruthy();
  });
});
