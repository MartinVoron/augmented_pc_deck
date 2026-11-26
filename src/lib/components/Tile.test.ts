import { render, screen } from "@testing-library/svelte";
import { Timer } from "@lucide/svelte";
import { createRawSnippet } from "svelte";
import { describe, expect, it } from "vitest";
import Tile from "./Tile.svelte";

const content = createRawSnippet(() => ({ render: () => "<p>25:00</p>" }));

describe("Tile", () => {
  it("affiche le titre et le contenu", () => {
    render(Tile, { title: "Minuteurs", icon: Timer, children: content });

    expect(screen.getByRole("heading", { name: "Minuteurs" })).toBeTruthy();
    expect(screen.getByText("25:00")).toBeTruthy();
  });

  it.each([
    ["1x1", "span 1", "span 1"],
    ["2x1", "span 2", "span 1"],
    ["2x2", "span 2", "span 2"],
  ] as const)("occupe la bonne place pour la taille %s", (size, columns, rows) => {
    render(Tile, { title: "Musique", size });

    const tile = screen.getByRole("region", { name: "Musique" });
    expect(tile.dataset.size).toBe(size);
    expect(tile.style.gridColumn).toBe(columns);
    expect(tile.style.gridRow).toBe(rows);
  });

  it("est une tuile 1×1 par défaut", () => {
    render(Tile, { title: "GitHub" });

    expect(screen.getByRole("region", { name: "GitHub" }).dataset.size).toBe("1x1");
  });
});
