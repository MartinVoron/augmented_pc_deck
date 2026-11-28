import { describe, expect, it } from "vitest";
import {
  appliedAppearance,
  applyToDocument,
  clampBlur,
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
    const glass = { theme: "glass" as const, mode: "system" as const, blur: 100 };
    expect(resolveScheme(glass, true)).toBe("dark");
    expect(resolveScheme(glass, false)).toBe("light");
    expect(resolveScheme({ ...glass, mode: "light" }, true)).toBe("light");
    expect(resolveScheme({ ...glass, mode: "dark" }, false)).toBe("dark");
  });

  it("garde le niveau de flou entre 0 et 100, 100 par défaut", () => {
    expect(clampBlur(undefined)).toBe(100);
    expect(clampBlur("50")).toBe(100);
    expect(clampBlur(-10)).toBe(0);
    expect(clampBlur(140)).toBe(100);
    expect(clampBlur(42.4)).toBe(42);
    expect(sanitizeAppearance({ theme: "glass", blur: 30 }).blur).toBe(30);
  });

  it("transmet le niveau de flou choisi", () => {
    expect(appliedAppearance({ theme: "glass", blur: 40 }, false, false)).toEqual({
      theme: "glass",
      scheme: "light",
      blur: 40,
    });
  });

  it("revient au Classique quand la fenêtre est ancrée en encoche", () => {
    expect(appliedAppearance({ theme: "glass", mode: "light", blur: 0 }, false, true)).toEqual({
      theme: "classic",
      scheme: "dark",
      blur: 100,
    });
  });

  it("pose les attributs lus par theme.css", () => {
    const root = document.createElement("html");
    applyToDocument(root, { theme: "glass", scheme: "light", blur: 40 });
    expect(root.dataset.theme).toBe("glass");
    expect(root.dataset.scheme).toBe("light");
    expect(root.style.getPropertyValue("--glass-level")).toBe("0.4");

    applyToDocument(root, { theme: "classic", scheme: "dark", blur: 100 });
    expect(root.dataset.theme).toBe("classic");
    expect(root.style.getPropertyValue("--glass-level")).toBe("1");
  });
});
