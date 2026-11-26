import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { isUrl } from ".";
import type { TileConfig } from "../../grid/defaultLayout";
import LauncherTile from "./LauncherTile.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const tile: TileConfig = {
  id: "launcher-1",
  kind: "launcher",
  title: "Raccourcis",
  size: "2x1",
  shortcuts: [
    { id: "s1", name: "Bloc-notes", target: "C:\\Windows\\notepad.exe", args: "notes.txt" },
    { id: "s2", name: "Ancien jeu", target: "D:\\Jeux\\parti.exe" },
  ],
};

describe("LauncherTile", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation((command, args) => {
      if (command === "launcher_icon") return Promise.resolve(null);
      if (command === "launcher_launch") {
        const { target } = args as { target: string };
        return target.includes("parti")
          ? Promise.reject({
              code: "not_found",
              message: "« D:\\Jeux\\parti.exe » est introuvable",
            })
          : Promise.resolve(null);
      }
      return Promise.reject({ code: "other", message: command });
    });
  });

  it("lance la cible avec ses arguments", async () => {
    render(LauncherTile, { tile });

    await fireEvent.click(screen.getByRole("button", { name: "Bloc-notes" }));

    expect(mockedInvoke).toHaveBeenCalledWith("launcher_launch", {
      target: "C:\\Windows\\notepad.exe",
      args: "notes.txt",
    });
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("affiche un message clair si la cible n'existe plus", async () => {
    render(LauncherTile, { tile });

    await fireEvent.click(screen.getByRole("button", { name: "Ancien jeu" }));

    expect(await screen.findByRole("alert")).toHaveProperty(
      "textContent",
      "« D:\\Jeux\\parti.exe » est introuvable. Modifiez ou supprimez ce raccourci.",
    );
  });

  it("indique quand la tuile n'a aucun raccourci", () => {
    render(LauncherTile, { tile: { ...tile, shortcuts: [] } });

    expect(screen.getByText("Aucun raccourci pour l'instant.")).toBeTruthy();
  });

  it("reconnaît les URL sans confondre une lettre de lecteur", () => {
    expect(isUrl("https://github.com")).toBe(true);
    expect(isUrl("spotify:album:1")).toBe(true);
    expect(isUrl("C:\\Windows")).toBe(false);
  });
});
