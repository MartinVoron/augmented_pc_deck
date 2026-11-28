import { describe, expect, it } from "vitest";
import {
  appliedAppearance,
  applyToDocument,
  DEFAULT_APPEARANCE,
  resolveScheme,
  sanitizeAppearance,
} from ".";

describe("apparence", () => {
  it("prend le thème Classique pour une ancienne configuration", () => {
    expect(sanitizeAppearance(undefined)).toEqual(DEFAULT_APPEARANCE);
    expect(sanitizeAppearance({ theme: "neon" as never, mode: "sepia" as never })).toEqual(
      DEFAULT_APPEARANCE,
    );
  });

  it("garde le Classique en sombre, quel que soit le mode", () => {
    const classic = { ...DEFAULT_APPEARANCE, mode: "light" as const };
    expect(resolveScheme(classic, false)).toBe("dark");
  });

  it("suit Windows en mode « Comme Windows »", () => {
    const glass = { theme: "glass" as const, mode: "system" as const, windowEffect: false };
    expect(resolveScheme(glass, true)).toBe("dark");
    expect(resolveScheme(glass, false)).toBe("light");
    expect(resolveScheme({ ...glass, mode: "light" }, true)).toBe("light");
    expect(resolveScheme({ ...glass, mode: "dark" }, false)).toBe("dark");
  });

  it("n'active le flou de fenêtre qu'avec le thème Verre", () => {
    expect(appliedAppearance({ theme: "classic", windowEffect: true }, true, false)).toEqual({
      theme: "classic",
      scheme: "dark",
      windowEffect: false,
    });
    expect(appliedAppearance({ theme: "glass", windowEffect: true }, false, false)).toEqual({
      theme: "glass",
      scheme: "light",
      windowEffect: true,
    });
  });

  it("revient au Classique sans effet quand la fenêtre est ancrée en encoche", () => {
    expect(
      appliedAppearance({ theme: "glass", mode: "light", windowEffect: true }, false, true),
    ).toEqual({ theme: "classic", scheme: "dark", windowEffect: false });
  });

  it("pose les attributs lus par theme.css", () => {
    const root = document.createElement("html");
    applyToDocument(root, { theme: "glass", scheme: "light", windowEffect: true });
    expect(root.dataset.theme).toBe("glass");
    expect(root.dataset.scheme).toBe("light");
    expect(root.hasAttribute("data-window-effect")).toBe(true);

    applyToDocument(root, { theme: "classic", scheme: "dark", windowEffect: false });
    expect(root.dataset.theme).toBe("classic");
    expect(root.hasAttribute("data-window-effect")).toBe(false);
  });
});
