import { describe, expect, it } from "vitest";
import type { PageConfig } from "./defaultLayout";
import { addPage, cleanPageName, movePage, PAGE_TEMPLATES, removePage, renamePage } from "./pages";

const pages: PageConfig[] = [
  {
    id: "page-1",
    name: "Principal",
    tiles: [{ id: "github-1", kind: "github", title: "GitHub", size: "1x1" }],
  },
  { id: "page-2", name: "Vide", tiles: [] },
];

const template = (id: string) => PAGE_TEMPLATES.find((t) => t.id === id)!;

describe("pages", () => {
  it("crée une page à partir d'un modèle, avec des tuiles aux identifiants uniques", () => {
    const { pages: next, page } = addPage(pages, template("travail"));

    expect(next).toHaveLength(3);
    expect(page.id).toBe("page-3");
    expect(page.name).toBe("Travail");
    expect(page.tiles.map((t) => t.id)).toEqual([
      "github-2",
      "gmail-1",
      "claude-1",
      "todos-1",
      "launcher-1",
    ]);
    expect(page.tiles[0]).toMatchObject({ kind: "github", title: "GitHub", size: "2x1" });
  });

  it("crée une page vide avec le nom choisi", () => {
    const { page } = addPage(pages, template("vide"), "  Projets  ");
    expect(page).toEqual({ id: "page-3", name: "Projets", tiles: [] });
  });

  it("chaque modèle n'utilise que des modules connus", () => {
    for (const t of PAGE_TEMPLATES) {
      const { page } = addPage([], t);
      expect(page.tiles).toHaveLength(t.tiles.length);
    }
  });

  it("nettoie les noms", () => {
    expect(cleanPageName("   ")).toBe("Page");
    expect(cleanPageName("x".repeat(50))).toHaveLength(30);
    expect(renamePage(pages, "page-2", "Musique")[1].name).toBe("Musique");
  });

  it("supprime une page, sauf la dernière", () => {
    expect(removePage(pages, "page-1").map((p) => p.id)).toEqual(["page-2"]);
    const single = [pages[0]];
    expect(removePage(single, "page-1")).toBe(single);
  });

  it("ne supprime jamais l'écran d'accueil", () => {
    const withHome = [{ ...pages[0], home: true }, pages[1]];
    expect(removePage(withHome, "page-1")).toBe(withHome);
  });

  it("déplace une page sans sortir des limites", () => {
    expect(movePage(pages, "page-2", -1).map((p) => p.id)).toEqual(["page-2", "page-1"]);
    expect(movePage(pages, "page-1", -1)).toBe(pages);
    expect(movePage(pages, "page-2", 1)).toBe(pages);
  });
});
