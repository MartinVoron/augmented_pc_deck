import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { sendNotification, setNotificationsEnabled } from "./notifications";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

describe("notifications", () => {
  beforeEach(() => {
    mockedInvoke.mockReset();
  });

  it("envoie le titre, le texte et le son au Rust", async () => {
    mockedInvoke.mockResolvedValue(true);

    const sent = await sendNotification({ title: "Pomodoro", body: "Pause !", sound: "reminder" });

    expect(sent).toBe(true);
    expect(mockedInvoke).toHaveBeenCalledWith("notifications_send", {
      notification: { title: "Pomodoro", body: "Pause !", sound: "reminder" },
    });
  });

  it("coupe les notifications", async () => {
    mockedInvoke.mockResolvedValue(undefined);

    await setNotificationsEnabled(false);

    expect(mockedInvoke).toHaveBeenCalledWith("notifications_set_enabled", { enabled: false });
  });
});
