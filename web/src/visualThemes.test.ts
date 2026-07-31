import { describe, expect, it } from "vitest";
import strawberryJamAtlas from "../public/assets/strawberry-jam/gameplay/theme-selected.json";
import {
  DEFAULT_VISUAL_THEME_ID,
  VISUAL_THEME_COLLECTIONS,
  VISUAL_THEMES,
  isVisualThemeId,
  visualThemeById,
  type VisualThemeId,
} from "./visualThemes";

describe("visual themes", () => {
  it("exposes the five original themes, ten curated Strawberry Jam themes, and extracted room themes", () => {
    const curated = VISUAL_THEMES.filter(
      (theme) => theme.collection === "celeste" || theme.collection === "strawberry-jam",
    );
    expect(curated).toHaveLength(15);
    const rooms = VISUAL_THEMES.filter(
      (theme) => theme.collection === "strawberry-jam-rooms",
    );
    expect(rooms.length).toBeGreaterThan(0);
    expect(VISUAL_THEMES).toHaveLength(curated.length + rooms.length);
    expect(new Set(VISUAL_THEMES.map((theme) => theme.id)).size).toBe(
      VISUAL_THEMES.length,
    );
    expect(new Set(curated.map((theme) => theme.tileset)).size).toBe(
      curated.length,
    );
    expect(
      VISUAL_THEMES.filter((theme) => theme.collection === "celeste"),
    ).toHaveLength(5);
    expect(
      VISUAL_THEMES.filter((theme) => theme.collection === "strawberry-jam"),
    ).toHaveLength(10);
    expect(VISUAL_THEME_COLLECTIONS.map((collection) => collection.id)).toEqual(
      ["celeste", "strawberry-jam", "strawberry-jam-rooms"],
    );
    for (const theme of VISUAL_THEMES)
      expect(theme.tileset).toMatch(/^tilesets\//);
    expect(VISUAL_THEME_COLLECTIONS.map((collection) => collection.id)).toContain(
      "strawberry-jam-rooms",
    );
    expect(visualThemeById("celestial-resort").spike).toBe("default");
    expect(visualThemeById("golden-ridge").spike).toBe("cliffside");
    expect(visualThemeById("summit").spike).toBe("outline");
  });

  it("includes every Strawberry Jam gym tier with its native autotiler layout", () => {
    const gyms = VISUAL_THEMES.filter((theme) => theme.id.endsWith("-gym"));
    expect(gyms.map((theme) => theme.id)).toEqual([
      "sj-beginner-gym",
      "sj-intermediate-gym",
      "sj-advanced-gym",
      "sj-expert-gym",
      "sj-grandmaster-gym",
    ]);
    for (const gym of gyms) {
      expect(gym.tileLayout).toBe("sj-gym");
      expect(gym.layers).toEqual([expect.objectContaining({ repeat: true })]);
    }
  });

  it("backs every Strawberry Jam theme reference with a packed source texture", () => {
    const entries = strawberryJamAtlas.entries as Record<string, unknown>;
    for (const theme of VISUAL_THEMES.filter(
      (candidate) => candidate.collection === "strawberry-jam",
    )) {
      expect(entries[theme.tileset]).toBeDefined();
      for (const layer of theme.layers) {
        expect(entries[layer.key]).toBeDefined();
      }
      for (const direction of ["up", "down", "left", "right"]) {
        // powerav ships its downward frame under the misspelled spacet_down00
        // name, so space_down is absent exactly as in the original mod.
        if (theme.spike === "SJ2021/powerav/space" && direction === "down")
          continue;
        expect(
          Object.keys(entries).some((key) =>
            key.startsWith(`danger/spikes/${theme.spike}_${direction}`),
          ),
        ).toBe(true);
      }
      if (!theme.spinner.foreground.startsWith("danger/crystal/")) {
        expect(
          Object.keys(entries).some((key) =>
            key.startsWith(theme.spinner.foreground),
          ),
        ).toBe(true);
        expect(
          Object.keys(entries).some((key) =>
            key.startsWith(theme.spinner.background),
          ),
        ).toBe(true);
      }
    }

    for (const theme of VISUAL_THEMES.filter(
      (candidate) => candidate.tileLayout === "sj-gym",
    )) {
      const entry =
        strawberryJamAtlas.entries[
          theme.tileset as keyof typeof strawberryJamAtlas.entries
        ];
      expect(entry).toEqual(
        expect.objectContaining({ width: 24, height: 136 }),
      );
    }
  });

  it("validates persisted ids and safely resolves the default", () => {
    expect(isVisualThemeId("summit")).toBe(true);
    expect(isVisualThemeId("not-a-theme")).toBe(false);
    expect(isVisualThemeId(null)).toBe(false);
    expect(visualThemeById("not-a-theme" as VisualThemeId).id).toBe(
      DEFAULT_VISUAL_THEME_ID,
    );
  });

  it("carries per-room tileset rules and parallax layers from the extraction", () => {
    const rooms = VISUAL_THEMES.filter(
      (theme) => theme.collection === "strawberry-jam-rooms",
    );
    const gym = rooms.find((theme) => theme.chapter.includes("0-Gyms/1-Beginner"));
    expect(gym).toBeDefined();
    expect(gym?.tileset).toBe("tilesets/SJ2021/Gym/BeginnerGym");
    expect(gym?.tileRules?.length).toBe(46);
    expect(gym?.layers[0]).toEqual(
      expect.objectContaining({ key: "bgs/SJ2021/Gym/begGymDarkBG", scrollX: 1 }),
    );
    const lobby = rooms.find((theme) =>
      theme.chapter.includes("0-Lobbies/1-Beginner.bin"),
    );
    expect(lobby?.layers.some((layer) => layer.scrollX !== undefined)).toBe(true);
  });
});
