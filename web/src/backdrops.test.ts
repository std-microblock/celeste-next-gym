import { describe, expect, it } from "vitest";
import type { GymMap, GymMapBackdrop } from "./model";
import {
  backdropGlobMatches,
  backdropVisible,
  backdropWorldAnchor,
  visibleMapBackdrops,
} from "./backdrops";

function backdrop(overrides: Partial<GymMapBackdrop> = {}): GymMapBackdrop {
  return {
    kind: "parallax",
    texture: "bgs/01/bg0",
    x: 0,
    y: 0,
    scroll_x: 1,
    scroll_y: 1,
    speed_x: 0,
    speed_y: 0,
    loop_x: true,
    loop_y: true,
    flip_x: false,
    flip_y: false,
    color: "ffffff",
    alpha: 1,
    blend_mode: "alphablend",
    exclude: "",
    only: "",
    flag: "",
    not_flag: "",
    ...overrides,
  };
}

describe("backdrop visibility", () => {
  it("expands * globs the same way MapData.ParseLevelsList does", () => {
    expect(backdropGlobMatches("*", "sj2021beginnerlobby")).toBe(true);
    expect(backdropGlobMatches("a-*", "a-00")).toBe(true);
    expect(backdropGlobMatches("a-*", "b-00")).toBe(false);
    expect(backdropGlobMatches("Lobby", "Lobby")).toBe(true);
    expect(backdropGlobMatches("Lobby", "sj2021beginnerlobby")).toBe(false);
  });

  it("applies only/exclude and flag rules", () => {
    expect(backdropVisible(backdrop(), "a-00")).toBe(true);
    expect(backdropVisible(backdrop({ only: "a-*" }), "a-00")).toBe(true);
    expect(backdropVisible(backdrop({ only: "a-*" }), "b-00")).toBe(false);
    expect(backdropVisible(backdrop({ exclude: "a-00" }), "a-00")).toBe(false);
    expect(backdropVisible(backdrop({ flag: "lightMode" }), "a-00")).toBe(false);
    expect(
      backdropVisible(backdrop({ flag: "lightMode" }), "a-00", { lightMode: true }),
    ).toBe(true);
    expect(backdropVisible(backdrop({ not_flag: "dark" }), "a-00")).toBe(true);
    expect(backdropVisible(backdrop({ not_flag: "dark" }), "a-00", { dark: true })).toBe(false);
  });

  it("filters map backdrops by the room name", () => {
    const map: GymMap = {
      name: "chapter",
      room: "a-00",
      bounds: { x: 0, y: 0, width: 320, height: 180 },
      spawn: { x: 32, y: 160 },
      solids: [],
      entities: [],
      source_package: "TestMap",
      backdrops: [
        backdrop({ texture: "bgs/01/bg0" }),
        backdrop({ texture: "bgs/01/bg1", only: "b-*" }),
      ],
    };
    expect(visibleMapBackdrops(map).map((item) => item.texture)).toEqual([
      "bgs/01/bg0",
    ]);
  });

  it("anchors parallax layers at Position + Camera * (1 - Scroll)", () => {
    expect(backdropWorldAnchor(backdrop({ x: 0, y: 0, scroll_x: 0, scroll_y: 0 }), { x: 120, y: 40 }, 0)).toEqual({
      x: 120,
      y: 40,
    });
    expect(backdropWorldAnchor(backdrop({ x: 0, y: 0, scroll_x: 1, scroll_y: 1 }), { x: 120, y: 40 }, 0)).toEqual({
      x: 0,
      y: 0,
    });
    expect(backdropWorldAnchor(backdrop({ x: 0, y: 0, scroll_x: 0.05, scroll_y: 1 }), { x: 100, y: 0 }, 0)).toEqual({
      x: 95,
      y: 0,
    });
    expect(backdropWorldAnchor(backdrop({ speed_x: 30 }), { x: 0, y: 0 }, 120)).toEqual({
      x: 60,
      y: 0,
    });
  });
});
