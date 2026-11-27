import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ClaudeStatus } from ".";
import ClaudeSection from "./ClaudeSection.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

let status: ClaudeStatus;

beforeEach(() => {
  status = {
    server: { state: "listening" },
    port: 47821,
    scriptPath: "C:\\Users\\moi\\AppData\\Roaming\\deck\\claude-hook.ps1",
    scriptInstalled: false,
    settingsSnippet: '{ "hooks": {} }',
  };
  mockedInvoke.mockReset();
  mockedInvoke.mockImplementation((command) => {
    if (command === "claude_install_hook" || command === "claude_regenerate_token") {
      status = { ...status, scriptInstalled: true };
    }
    return Promise.resolve(status);
  });
});

describe("ClaudeSection", () => {
  it("crée le script puis permet de copier les réglages", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    render(ClaudeSection);

    const copy = await screen.findByRole("button", { name: /Copier le bloc/ });
    expect((copy as HTMLButtonElement).disabled).toBe(true);

    await fireEvent.click(screen.getByRole("button", { name: /Créer le script/ }));
    expect(mockedInvoke).toHaveBeenCalledWith("claude_install_hook", undefined);
    expect(await screen.findByText(/claude-hook.ps1/)).toBeTruthy();

    await fireEvent.click(copy);
    expect(writeText).toHaveBeenCalledWith('{ "hooks": {} }');
    expect(await screen.findByText(/Copié/)).toBeTruthy();
  });

  it("explique un serveur qui n'a pas démarré", async () => {
    status.server = { state: "failed", error: "le port 47821 est indisponible" };
    render(ClaudeSection);

    expect((await screen.findByRole("alert")).textContent).toContain("port 47821");
  });

  it("régénère le jeton après confirmation", async () => {
    render(ClaudeSection);

    await fireEvent.click(await screen.findByRole("button", { name: /Régénérer le jeton/ }));
    await fireEvent.click(screen.getByRole("button", { name: "Confirmer" }));

    expect(mockedInvoke).toHaveBeenCalledWith("claude_regenerate_token", undefined);
    expect(await screen.findByText("Nouveau jeton créé.")).toBeTruthy();
  });
});
