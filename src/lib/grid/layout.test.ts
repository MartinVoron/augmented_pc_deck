import { describe, expect, it } from "vitest";
import { DEFAULT_LAYOUT, type TileConfig } from "./defaultLayout";
import { addTile, moveTile, removeTile, resizeTile, sanitizeLayout } from "./layout";

const tiles: TileConfig[] = [
  { id: "a", kind: "timers", title: "A", size: "1x1" },
  { id: "b", kind: "music", title: "B", size: "2x2" },
  { id: "c", kind: "gmail", title: "C", size: "1x1" },
];

const ids = (list: TileConfig[]) => list.map((tile) => tile.id);

describe("disposition de la grille", () => {
  it("ajoute une tuile 1×1 avec un identifiant libre", () => {
    const once = addTile([], "timers");
    const twice = addTile(once, "timers");

    expect(twice.map((t) => t.id)).toEqual(["timers-1", "timers-2"]);
    expect(twice[1]).toMatchObject({ kind: "timers", title: "Minuteurs", size: "1x1" });
  });

  it("supprime une tuile", () => {
    expect(ids(removeTile(tiles, "b"))).toEqual(["a", "c"]);
  });

  it("redimensionne une tuile sans toucher aux autres", () => {
    const result = resizeTile(tiles, "a", "2x1");

    expect(result[0].size).toBe("2x1");
    expect(result[1]).toBe(tiles[1]);
  });

  it("déplace une tuile", () => {
    expect(ids(moveTile(tiles, "a", 2))).toEqual(["b", "c", "a"]);
    expect(ids(moveTile(tiles, "c", 0))).toEqual(["c", "a", "b"]);
  });

  it("ne sort pas des limites en déplaçant", () => {
    expect(ids(moveTile(tiles, "a", -1))).toEqual(["a", "b", "c"]);
    expect(ids(moveTile(tiles, "c", 99))).toEqual(["a", "b", "c"]);
  });

  it("ignore les tuiles inconnues lues dans la configuration", () => {
    const result = sanitizeLayout([
      { id: "a", kind: "timers", title: "A", size: "1x1" },
      { id: "x", kind: "meteo", title: "Météo", size: "1x1" },
      { id: "y", kind: "music", title: "Musique", size: "3x3" },
    ]);

    expect(ids(result)).toEqual(["a"]);
  });

  it("utilise la disposition par défaut tant que la configuration n'est pas chargée", () => {
    expect(sanitizeLayout(undefined)).toBe(DEFAULT_LAYOUT);
  });
});
