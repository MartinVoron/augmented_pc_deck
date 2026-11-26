import { invoke } from "@tauri-apps/api/core";
import { render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import App from "./App.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("App", () => {
  it("affiche la version renvoyée par le Rust", async () => {
    vi.mocked(invoke).mockResolvedValue("0.1.0");

    render(App);

    expect(screen.getByRole("heading", { name: "Augmented PC Deck" })).toBeTruthy();
    expect(await screen.findByText("Version 0.1.0")).toBeTruthy();
  });

  it("affiche un message clair si le Rust ne répond pas", async () => {
    vi.mocked(invoke).mockRejectedValue({ code: "other", message: "pas de réponse" });

    render(App);

    expect(await screen.findByText(/pas joignable : pas de réponse/)).toBeTruthy();
  });
});
