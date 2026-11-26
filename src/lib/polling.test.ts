import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { poll } from "./polling";

function setHidden(hidden: boolean) {
  Object.defineProperty(document, "hidden", { configurable: true, get: () => hidden });
  document.dispatchEvent(new Event("visibilitychange"));
}

describe("poll", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => {
    vi.useRealTimers();
    setHidden(false);
  });

  it("appelle la tâche tout de suite puis à intervalle régulier", async () => {
    const task = vi.fn();
    const stop = poll(task, 2000);
    await vi.advanceTimersByTimeAsync(4100);
    expect(task).toHaveBeenCalledTimes(3);
    stop();
    await vi.advanceTimersByTimeAsync(10000);
    expect(task).toHaveBeenCalledTimes(3);
  });

  it("se met en pause quand la fenêtre est masquée", async () => {
    const task = vi.fn();
    const stop = poll(task, 1000);
    await vi.advanceTimersByTimeAsync(10);
    setHidden(true);
    await vi.advanceTimersByTimeAsync(5000);
    expect(task).toHaveBeenCalledTimes(1);
    setHidden(false);
    await vi.advanceTimersByTimeAsync(10);
    expect(task).toHaveBeenCalledTimes(2);
    stop();
  });
});
