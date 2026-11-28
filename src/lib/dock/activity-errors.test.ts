import { describe, expect, it, vi } from "vitest";

// Faux modules : on garde la tâche de lecture pour l'appeler à la main.
const { tasks, getNowPlaying, logError } = vi.hoisted(() => ({
  tasks: [] as (() => Promise<void> | void)[],
  getNowPlaying: vi.fn(),
  logError: vi.fn(),
}));
vi.mock("../polling", () => ({
  poll: (task: () => Promise<void> | void) => {
    tasks.push(task);
    return () => {};
  },
}));
vi.mock("../modules/music", () => ({ getNowPlaying }));
vi.mock("../modules/timers", () => ({
  listRuns: () => Promise.resolve({}),
  onTimersChanged: () => () => {},
}));
vi.mock("../logging", () => ({ logError }));

import { live, watchActivity } from "./activity.svelte";

describe("watchActivity", () => {
  it("ne note qu'une fois l'erreur d'un lecteur de musique injoignable", async () => {
    const stop = watchActivity();
    const read = tasks[0];
    getNowPlaying.mockRejectedValue(new Error("D-Bus injoignable"));

    await read();
    await read();
    await read();
    expect(logError).toHaveBeenCalledTimes(1);
    expect(live.track).toBeNull();

    // Le lecteur répond de nouveau, puis retombe : l'erreur est notée encore.
    getNowPlaying.mockResolvedValueOnce(null);
    await read();
    await read();
    expect(logError).toHaveBeenCalledTimes(2);
    stop();
  });
});
