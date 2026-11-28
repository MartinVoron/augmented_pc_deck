import { describe, expect, it } from "vitest";
import { portal } from "./portal";

describe("portal", () => {
  it("place l'élément dans body, puis le retire", () => {
    const parent = document.createElement("div");
    const node = document.createElement("div");
    parent.appendChild(node);
    document.body.appendChild(parent);

    const action = portal(node);
    expect(node.parentElement).toBe(document.body);

    action.destroy();
    expect(node.isConnected).toBe(false);
    parent.remove();
  });
});
