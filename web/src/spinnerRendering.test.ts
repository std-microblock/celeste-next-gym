import { describe, expect, it } from "vitest";
import type { MapEntity } from "./model";
import {
  resolveSpinnerStyle,
  spinnerCenter,
  spinnerHue,
  spinnersConnect,
} from "./spinnerRendering";
import { visualThemeById } from "./visualThemes";

function spinner(x: number, y: number): MapEntity {
  return {
    kind: "crystal_static_spinner",
    bounds: { x, y, width: 16, height: 12 },
    direction: { x: 0, y: 0 },
    name: "spinner",
  };
}

describe("crystal spinner rendering", () => {
  it("uses the theme art unless the editor selects a vanilla color override", () => {
    const theme = visualThemeById("sj-beginner-lobby");
    expect(resolveSpinnerStyle(theme)).toEqual({
      foreground: "danger/spikes/SJ2021/1-Beginner/brambles/fg",
      background: "danger/spikes/SJ2021/1-Beginner/brambles/bg"
    });
    expect(resolveSpinnerStyle(theme, "red")).toEqual({
      foreground: "danger/crystal/fg_red",
      background: "danger/crystal/bg_red",
    });
    expect(resolveSpinnerStyle(theme, "rainbow").rainbow).toBe(true);
  });

  it("uses the vanilla 24-pixel neighbor threshold for connector sprites", () => {
    const left = spinner(0, 0);
    const near = spinner(23, 0);
    const edge = spinner(24, 0);
    expect(spinnerCenter(left)).toEqual({ x: 8, y: 6 });
    expect(spinnersConnect(left, near)).toBe(true);
    expect(spinnersConnect(left, edge)).toBe(false);
  });

  it("produces a stable quantized rainbow tint", () => {
    expect(spinnerHue({ x: 32, y: 48 }, 120)).toMatch(/^#[0-9a-f]{6}$/);
    expect(spinnerHue({ x: 32, y: 48 }, 120)).toBe(
      spinnerHue({ x: 32, y: 48 }, 120),
    );
  });
});
