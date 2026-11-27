import { invoke } from "@tauri-apps/api/core";
import { render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { currentPosition, formatTime, type NowPlaying } from ".";
import type { TileConfig } from "../../grid/defaultLayout";
import MusicTile from "./MusicTile.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const tile: TileConfig = { id: "music-1", kind: "music", title: "Musique", size: "2x2" };
const track: NowPlaying = {
  title: "Clair de lune",
  artist: "Claude Debussy",
  album: "Suite bergamasque",
  source: "Spotify",
  status: "paused",
  positionMs: 95_000,
  durationMs: 300_000,
  updatedAt: 1_000,
  thumbnail: "data:image/png;base64,AQID",
  canPlayPause: true,
  canNext: true,
  canPrevious: false,
};

describe("MusicTile", () => {
  beforeEach(() => mockedInvoke.mockReset());

  it("affiche le titre, l'artiste, la pochette et la progression", async () => {
    mockedInvoke.mockResolvedValue(track);
    render(MusicTile, { tile });

    expect(await screen.findByText("Clair de lune")).toBeTruthy();
    expect(screen.getByText("Claude Debussy")).toBeTruthy();
    expect(screen.getByRole("img", { name: "Pochette de Suite bergamasque" })).toBeTruthy();
    expect(screen.getByText(/Spotify/).textContent).toContain("en pause");
    const bar = screen.getByRole("progressbar", { name: "Progression du morceau" });
    expect(bar.getAttribute("aria-valuetext")).toBe("1:35 sur 5:00");
  });

  it("affiche un état vide quand rien ne joue", async () => {
    mockedInvoke.mockResolvedValue(null);
    render(MusicTile, { tile });

    expect(await screen.findByText("Rien ne joue en ce moment.")).toBeTruthy();
  });

  it("fait avancer la position pendant la lecture seulement", () => {
    const playing = { ...track, status: "playing" as const };
    expect(currentPosition(playing, 11_000)).toBe(105_000);
    expect(currentPosition(playing, 10_000_000)).toBe(300_000);
    expect(currentPosition(track, 11_000)).toBe(95_000);
    expect(formatTime(3_723_000)).toBe("1:02:03");
  });
});
