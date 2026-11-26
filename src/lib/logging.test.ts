import { describe, expect, it } from "vitest";
import { describeError } from "./logging";

describe("describeError", () => {
  it("garde le message d'une erreur", () => {
    expect(describeError(new Error("disque plein"))).toContain("disque plein");
  });

  it("accepte une valeur qui n'est pas une erreur", () => {
    expect(describeError("plantage")).toBe("plantage");
  });
});
