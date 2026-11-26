import { invoke } from "@tauri-apps/api/core";
import { render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { formatBytes, percent } from ".";
import ProcessesTile from "./ProcessesTile.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);
const GO = 1024 ** 3;

describe("ProcessesTile", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
    mockedInvoke.mockResolvedValue({
      cpu: 23.4,
      memoryUsed: 9.1 * GO,
      memoryTotal: 16 * GO,
      disks: [{ name: "C:", used: 470 * GO, total: 476 * GO }],
    });
  });

  it("affiche le processeur, la mémoire et les disques", async () => {
    render(ProcessesTile);

    const cpu = await screen.findByRole("meter", { name: "Processeur" });
    expect(cpu.getAttribute("aria-valuetext")).toBe("23 %");
    expect(screen.getByRole("meter", { name: "Mémoire" }).getAttribute("aria-valuetext")).toBe(
      "9,1 Go / 16 Go",
    );
    const disk = screen.getByRole("meter", { name: "Disque C:" });
    expect(disk.getAttribute("aria-valuenow")).toBe("99");
    expect(disk.closest("[data-level]")?.getAttribute("data-level")).toBe("danger");
  });

  it("affiche un message si la mesure échoue", async () => {
    mockedInvoke.mockRejectedValue({ code: "other", message: "mesures indisponibles" });
    render(ProcessesTile);

    expect((await screen.findByRole("alert")).textContent).toBe("mesures indisponibles");
  });

  it("formate les tailles et les pourcentages", () => {
    expect(formatBytes(512 * 1024 ** 2)).toBe("512 Mo");
    expect(formatBytes(1.85 * 1024 * GO)).toBe("1,9 To");
    expect(percent(5, 0)).toBe(0);
    expect(percent(150, 100)).toBe(100);
  });
});
