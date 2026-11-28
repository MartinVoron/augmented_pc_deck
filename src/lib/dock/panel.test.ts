import { describe, expect, it } from "vitest";
import { packedRows, PANEL_BAR, panelLayout } from "./panel";

describe("packedRows", () => {
  it("range les tuiles dans le premier trou libre", () => {
    expect(packedRows(["1x1", "1x1", "1x1"], 2)).toBe(2);
    // La grande tuile passe sous les petites ; la petite suivante remplit le trou.
    expect(packedRows(["1x1", "2x2", "1x1"], 3)).toBe(2);
    expect(packedRows(["2x2", "2x1"], 2)).toBe(3);
    expect(packedRows([], 4)).toBe(0);
  });

  it("réduit une tuile trop large à la grille", () => {
    expect(packedRows(["2x1"], 1)).toBe(1);
  });
});

describe("panelLayout", () => {
  it("fait une bande de deux lignes en haut et en bas", () => {
    const sizes = ["1x1", "2x2", "1x1", "2x1"] as const;
    const layout = panelLayout([...sizes], "top");
    // 8 cases : 4 colonnes laisseraient un trou et une 3e ligne, donc 5 colonnes.
    expect(layout).toMatchObject({ columns: 5, rows: 2 });
    expect(layout.width).toBe(5 * 140 + 4 * 12 + 2 * 12);
    expect(layout.height).toBe(PANEL_BAR + 2 * 140 + 12 + 2 * 12);
    expect(panelLayout([...sizes], "bottom")).toEqual(layout);
  });

  it("prend juste la largeur utile sans trou", () => {
    expect(panelLayout(["1x1", "1x1", "1x1", "1x1"], "top")).toMatchObject({ columns: 2, rows: 2 });
    expect(panelLayout(["2x2", "2x2"], "bottom")).toMatchObject({ columns: 4, rows: 2 });
  });

  it("ne dépasse pas la largeur de l'écran", () => {
    const sizes = Array.from({ length: 20 }, () => "1x1" as const);
    const layout = panelLayout(sizes, "top", 800);
    // (800 - 12) / 152 = 5 colonnes au plus, donc 4 lignes.
    expect(layout).toMatchObject({ columns: 5, rows: 4 });
    expect(layout.width).toBeLessThanOrEqual(800);
  });

  it("fait une colonne de deux cases à gauche et à droite", () => {
    const layout = panelLayout(["1x1", "2x2", "1x1", "2x1"], "left");
    expect(layout).toMatchObject({ columns: 2, rows: 4 });
    expect(layout.width).toBe(2 * 140 + 12 + 2 * 12);
    expect(panelLayout(["1x1"], "right")).toMatchObject({ columns: 2, rows: 1 });
  });

  it("garde de la place pour le message d'une page vide", () => {
    expect(panelLayout([], "top")).toMatchObject({ columns: 2, rows: 1 });
  });
});
