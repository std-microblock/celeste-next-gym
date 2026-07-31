import { describe, expect, it } from "vitest";
import {
  DEFAULT_VISUAL_THEME_ID,
  VISUAL_THEME_COLLECTIONS,
  VISUAL_THEMES,
  isVisualThemeId,
  visualThemeById,
  type VisualThemeId,
} from "./visualThemes";

describe("visual themes", () => {
  it("exposes generated map themes across the official, Strawberry Jam, and CNY collections", () => {
    expect(VISUAL_THEMES.length).toBeGreaterThan(100);
    expect(new Set(VISUAL_THEMES.map((theme) => theme.id)).size).toBe(
      VISUAL_THEMES.length,
    );
    const byCollection = new Map(
      VISUAL_THEME_COLLECTIONS.map((collection) => [collection.id, 0]),
    );
    for (const theme of VISUAL_THEMES) {
      byCollection.set(theme.collection, (byCollection.get(theme.collection) ?? 0) + 1);
    }
    expect(byCollection.get("celeste-rooms") ?? 0).toBeGreaterThan(0);
    expect(byCollection.get("strawberry-jam-rooms") ?? 0).toBeGreaterThan(50);
    expect(byCollection.get("cny2024-rooms") ?? 0).toBeGreaterThan(0);
    expect(VISUAL_THEME_COLLECTIONS.map((collection) => collection.id)).toEqual([
      "celeste-rooms",
      "strawberry-jam-rooms",
      "cny2024-rooms",
    ]);
    for (const theme of VISUAL_THEMES)
      expect(theme.tileset).toMatch(/^tilesets\//);
  });

  it("carries per-map tileset rules, spike types and parallax layers", () => {
    const gym = VISUAL_THEMES.find(
      (theme) =>
        theme.collection === "strawberry-jam-rooms" &&
        theme.chapter.includes("0-Gyms/1-Beginner"),
    );
    expect(gym).toBeDefined();
    expect(gym?.tileset).toBe("tilesets/SJ2021/Gym/BeginnerGym");
    expect(gym?.tileRules?.length).toBe(46);
    expect(gym?.spike).toBe("SJ2021/Gym/beg");
    expect(gym?.layers[0]).toEqual(
      expect.objectContaining({ key: "bgs/SJ2021/Gym/begGymDarkBG", scrollX: 1 }),
    );
    const lobby = VISUAL_THEMES.find(
      (theme) =>
        theme.collection === "strawberry-jam-rooms" &&
        theme.chapter.includes("0-Lobbies/1-Beginner.bin"),
    );
    expect(lobby?.layers.some((layer) => layer.scrollX !== undefined)).toBe(true);
    const vanilla = VISUAL_THEMES.find(
      (theme) =>
        theme.collection === "celeste-rooms" &&
        theme.chapter === "1-ForsakenCity.bin",
    );
    expect(vanilla?.tileset).toBe("tilesets/snow");
  });

  it("validates persisted ids and safely resolves the default", () => {
    const defaultTheme = visualThemeById(DEFAULT_VISUAL_THEME_ID);
    expect(defaultTheme.collection).toBe("celeste-rooms");
    expect(isVisualThemeId(DEFAULT_VISUAL_THEME_ID)).toBe(true);
    expect(isVisualThemeId("not-a-theme")).toBe(false);
    expect(visualThemeById("not-a-theme" as VisualThemeId).id).toBe(
      DEFAULT_VISUAL_THEME_ID,
    );
  });
});
