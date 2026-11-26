import { describe, expect, it } from "vitest";
import type { Shortcut } from ".";
import {
  defaultName,
  moveShortcut,
  removeShortcut,
  saveShortcut,
  shortcutFromTarget,
} from "./shortcuts";

const list: Shortcut[] = [
  { id: "raccourci-1", name: "A", target: "C:\\a.exe" },
  { id: "raccourci-2", name: "B", target: "C:\\b.exe" },
];

describe("raccourcis du lanceur", () => {
  it("propose un nom à partir de la cible", () => {
    expect(defaultName("C:\\Program Files\\Spotify\\Spotify.exe")).toBe("Spotify");
    expect(defaultName("D:\\Projets\\")).toBe("Projets");
    expect(defaultName('"C:\\Jeux\\Mon jeu.lnk"')).toBe("Mon jeu");
    expect(defaultName("https://www.github.com/notifications")).toBe("github.com");
    expect(defaultName("spotify:album:1")).toBe("spotify:album:1");
  });

  it("crée un raccourci avec un identifiant libre", () => {
    const shortcut = shortcutFromTarget(list, " C:\\Outils\\outil.exe ");
    expect(shortcut).toEqual({ id: "raccourci-3", name: "outil", target: "C:\\Outils\\outil.exe" });
  });

  it("ajoute puis modifie un raccourci, sans garder les champs vides", () => {
    const added = saveShortcut(list, { id: "raccourci-3", name: " C ", target: "c", args: " " });
    expect(added[2]).toEqual({ id: "raccourci-3", name: "C", target: "c" });

    const edited = saveShortcut(added, { ...list[0], args: "--vite" });
    expect(edited[0].args).toBe("--vite");
    expect(edited).toHaveLength(3);
  });

  it("supprime et réordonne", () => {
    expect(removeShortcut(list, "raccourci-1").map((s) => s.name)).toEqual(["B"]);
    expect(moveShortcut(list, "raccourci-2", -1).map((s) => s.name)).toEqual(["B", "A"]);
    expect(moveShortcut(list, "raccourci-2", 1)).toBe(list);
  });
});
