import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createRawSnippet } from "svelte";
import TileBoundary from "./TileBoundary.svelte";

// Le journal est remplacé par une fausse fonction pour vérifier ce qui y est écrit.
const { writeError } = vi.hoisted(() => ({ writeError: vi.fn() }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: writeError }));

let shouldFail = true;

// Contenu de tuile qui plante tant que `shouldFail` vaut true.
const content = createRawSnippet(() => ({
  render: () => "<p>Contenu de la tuile</p>",
  setup: () => {
    if (shouldFail) throw new Error("plus de réseau");
  },
}));

describe("TileBoundary", () => {
  beforeEach(() => {
    writeError.mockReset();
    vi.spyOn(console, "error").mockImplementation(() => {});
  });

  it("affiche le contenu quand tout va bien", () => {
    shouldFail = false;
    render(TileBoundary, { name: "GitHub", children: content });

    expect(screen.getByText("Contenu de la tuile")).toBeTruthy();
  });

  it("affiche un message clair et écrit l'erreur dans le journal", async () => {
    shouldFail = true;
    render(TileBoundary, { name: "GitHub", children: content });

    expect(screen.getByRole("alert").textContent).toContain("GitHub ne répond plus");
    expect(screen.getByText("plus de réseau")).toBeTruthy();
    expect(writeError).toHaveBeenCalledWith(expect.stringContaining("[tuile GitHub]"));

    shouldFail = false;
    await fireEvent.click(screen.getByRole("button", { name: "Réessayer" }));

    expect(await screen.findByText("Contenu de la tuile")).toBeTruthy();
  });
});
