import { describe, expect, it } from "vitest";
import { BOUNCE_ROOM, NOTCH_DEPTH, NOTCH_EAR, NOTCH_LENGTH } from ".";
import { foldedSize, openSize, windowForPanel } from "./island";

describe("géométrie de l'encoche", () => {
  it("laisse la place des oreilles dans la fenêtre repliée", () => {
    expect(foldedSize("top")).toEqual({
      width: NOTCH_LENGTH - 2 * NOTCH_EAR,
      height: NOTCH_DEPTH,
    });
    expect(foldedSize("right")).toEqual({
      width: NOTCH_DEPTH,
      height: NOTCH_LENGTH - 2 * NOTCH_EAR,
    });
    expect(foldedSize("bottom", 300).width).toBe(300 - 2 * NOTCH_EAR);
  });

  it("retrouve le panneau dans la fenêtre demandée au Rust", () => {
    const panel = { width: 640, height: 360 };
    for (const edge of ["top", "bottom", "left", "right"] as const) {
      expect(openSize(edge, windowForPanel(edge, panel))).toEqual(panel);
    }
    expect(windowForPanel("top", panel)).toEqual({
      width: 640 + 2 * NOTCH_EAR + BOUNCE_ROOM,
      height: 360 + BOUNCE_ROOM,
    });
  });

  it("ne rapetisse jamais sous l'encoche repliée", () => {
    expect(openSize("top", { width: 50, height: 10 })).toEqual(foldedSize("top"));
  });
});
