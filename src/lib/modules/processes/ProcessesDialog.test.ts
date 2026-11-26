import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { filterProcesses, sortProcesses, type ProcessInfo } from ".";
import ProcessesDialog from "./ProcessesDialog.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);
const MO = 1024 ** 2;

const list: ProcessInfo[] = [
  { pid: 10, name: "chrome.exe", cpu: 4.2, memory: 900 * MO },
  { pid: 20, name: "Code.exe", cpu: 12.5, memory: 600 * MO },
  { pid: 30, name: "svchost.exe", cpu: 0.1, memory: 20 * MO },
];

function names(): string[] {
  return screen
    .getAllByRole("row")
    .slice(1)
    .map((row) => row.querySelector(".name")?.textContent ?? "");
}

describe("ProcessesDialog", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation((command, args) => {
      if (command === "processes_list") return Promise.resolve(list);
      if (command === "processes_kill") {
        const { name } = args as { name: string };
        return name === "svchost.exe"
          ? Promise.reject({ code: "permission_denied", message: "Windows refuse d'arrêter" })
          : Promise.resolve(null);
      }
      return Promise.reject({ code: "other", message: command });
    });
  });

  it("trie par mémoire puis par processeur, et filtre par nom", async () => {
    render(ProcessesDialog, { onclose: () => {} });
    await screen.findByText("chrome.exe");
    expect(names()).toEqual(["chrome.exe", "Code.exe", "svchost.exe"]);

    await fireEvent.click(screen.getByRole("button", { name: "Processeur" }));
    expect(names()).toEqual(["Code.exe", "chrome.exe", "svchost.exe"]);

    await fireEvent.input(screen.getByRole("textbox", { name: "Filtrer par nom" }), {
      target: { value: "CODE" },
    });
    expect(names()).toEqual(["Code.exe"]);
  });

  it("arrête un processus seulement après confirmation", async () => {
    render(ProcessesDialog, { onclose: () => {} });
    await fireEvent.click(await screen.findByRole("button", { name: "Arrêter chrome.exe" }));
    expect(mockedInvoke).not.toHaveBeenCalledWith("processes_kill", expect.anything());

    await fireEvent.click(screen.getByRole("button", { name: "Oui, arrêter" }));
    expect(mockedInvoke).toHaveBeenCalledWith("processes_kill", { pid: 10, name: "chrome.exe" });
    expect((await screen.findByRole("status")).textContent).toContain("a été arrêté");
  });

  it("affiche un message si Windows refuse", async () => {
    render(ProcessesDialog, { onclose: () => {} });
    const row = (await screen.findByText("svchost.exe")).closest("tr") as HTMLElement;
    await fireEvent.click(within(row).getByRole("button", { name: /Arrêter/ }));
    await fireEvent.click(screen.getByRole("button", { name: "Oui, arrêter" }));

    expect((await screen.findByRole("alert")).textContent).toContain("Windows refuse");
  });

  it("trie par nom sans tenir compte des majuscules", () => {
    expect(sortProcesses(list, "name", "asc").map((p) => p.name)).toEqual([
      "chrome.exe",
      "Code.exe",
      "svchost.exe",
    ]);
    expect(filterProcesses(list, "  ")).toBe(list);
  });
});
