import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { addTodo, editTodo, moveTodo, type Todo } from ".";
import type { TileConfig } from "../../grid/defaultLayout";
import type { Config } from "../settings";
import { loadSettings } from "../settings/state.svelte";
import TodosTile from "./TodosTile.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const todos: Todo[] = [
  { id: "tache-1", text: "Appeler le garage", done: true },
  { id: "tache-2", text: "Courses", done: false },
  { id: "tache-3", text: "Payer la facture", done: false },
];
const tile: TileConfig = { id: "todos-1", kind: "todos", title: "Tâches", size: "2x2", todos };

let saved: Config | null;

function savedTile() {
  return saved?.layout.tiles[0];
}

describe("TodosTile", () => {
  beforeEach(async () => {
    saved = null;
    mockedInvoke.mockReset();
    mockedInvoke.mockImplementation((command, args) => {
      if (command === "settings_get") {
        return Promise.resolve({
          version: 1,
          general: { language: "fr", notificationsEnabled: true },
          window: { alwaysOnTop: false, fullscreen: false },
          layout: { tiles: [tile] },
        });
      }
      if (command === "settings_save") {
        saved = (args as { config: Config }).config;
        return Promise.resolve(saved);
      }
      return Promise.reject({ code: "other", message: command });
    });
    await loadSettings();
  });

  it("ajoute une tâche", async () => {
    render(TodosTile, { tile });
    await fireEvent.input(screen.getByRole("textbox", { name: "Nouvelle tâche" }), {
      target: { value: "  Arroser les plantes " },
    });
    await fireEvent.submit(screen.getByRole("textbox", { name: "Nouvelle tâche" }));

    expect(savedTile()?.todos?.at(-1)).toEqual({
      id: "tache-4",
      text: "Arroser les plantes",
      done: false,
    });
  });

  it("coche, supprime et réordonne", async () => {
    render(TodosTile, { tile });
    expect(screen.getByText("2 à faire, 1 terminée")).toBeTruthy();

    await fireEvent.click(screen.getByRole("checkbox", { name: "Terminée : Courses" }));
    expect(savedTile()?.todos?.[1].done).toBe(true);

    await fireEvent.click(screen.getByRole("button", { name: "Monter Payer la facture" }));
    expect(savedTile()?.todos?.map((t) => t.id)).toEqual(["tache-1", "tache-3", "tache-2"]);

    await fireEvent.click(screen.getByRole("button", { name: "Supprimer Appeler le garage" }));
    expect(savedTile()?.todos?.map((t) => t.id)).toEqual(["tache-3", "tache-2"]);
  });

  it("modifie le texte d'une tâche", async () => {
    render(TodosTile, { tile });
    await fireEvent.click(screen.getByRole("button", { name: "Modifier Courses" }));
    const input = screen.getByRole("textbox", { name: "Texte de la tâche" });
    await fireEvent.input(input, { target: { value: "Courses du samedi" } });
    await fireEvent.keyDown(input, { key: "Enter" });

    expect(savedTile()?.todos?.[1].text).toBe("Courses du samedi");
  });

  it("masque les tâches terminées", async () => {
    render(TodosTile, { tile: { ...tile, hideDone: true } });
    expect(screen.queryByText("Appeler le garage")).toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: "Masquer les tâches terminées" }));
    expect(savedTile()?.hideDone).toBe(false);
  });

  it("modifie la liste sans perdre les tâches masquées", () => {
    expect(addTodo(todos, "   ")).toBe(todos);
    expect(editTodo(todos, "tache-2", " ").map((t) => t.id)).toEqual(["tache-1", "tache-3"]);
    const visible = todos.filter((t) => !t.done);
    expect(moveTodo(todos, "tache-2", -1, visible)).toBe(todos);
    expect(moveTodo(todos, "tache-3", -1, visible).map((t) => t.id)).toEqual([
      "tache-1",
      "tache-3",
      "tache-2",
    ]);
  });
});
