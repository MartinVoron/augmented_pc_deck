import { invoke } from "@tauri-apps/api/core";
import { render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import App from "./App.svelte";
import { DEFAULT_LAYOUT } from "./lib/grid/defaultLayout";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("App", () => {
  it("affiche une tuile par élément de la disposition par défaut", () => {
    vi.mocked(invoke).mockResolvedValue("0.1.0");

    render(App);

    for (const tile of DEFAULT_LAYOUT) {
      expect(screen.getByRole("region", { name: tile.title })).toBeTruthy();
    }
  });

  it("affiche la version renvoyée par le Rust", async () => {
    vi.mocked(invoke).mockResolvedValue("0.1.0");

    render(App);

    expect(await screen.findByText("Augmented PC Deck 0.1.0")).toBeTruthy();
  });

  it("affiche un message clair si le Rust ne répond pas", async () => {
    vi.mocked(invoke).mockRejectedValue({ code: "other", message: "pas de réponse" });

    render(App);

    expect(await screen.findByText(/pas joignable : pas de réponse/)).toBeTruthy();
  });
});
