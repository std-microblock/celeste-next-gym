import type { MapEntity } from "./model";
import type { VisualTheme } from "./visualThemes";

export type SpinnerVariant = "theme" | "blue" | "red" | "purple" | "rainbow";

const VANILLA_SPINNERS: Record<
  Exclude<SpinnerVariant, "theme">,
  VisualTheme["spinner"]
> = {
  blue: {
    foreground: "danger/crystal/fg_blue",
    background: "danger/crystal/bg_blue",
  },
  red: {
    foreground: "danger/crystal/fg_red",
    background: "danger/crystal/bg_red",
  },
  purple: {
    foreground: "danger/crystal/fg_purple",
    background: "danger/crystal/bg_purple",
  },
  rainbow: {
    foreground: "danger/crystal/fg_white",
    background: "danger/crystal/bg_white",
    rainbow: true,
  },
};


/** Spinner skin carried by decoded mod entities (VivHelper/CustomSpinner and
 * FrostHelper/IceSpinner). The Rust decoder stores the foreground prefix, so
 * the background sheet is the same path with /fg replaced by /bg. */
export function entitySpinnerStyle(
  entity: MapEntity,
): VisualTheme["spinner"] | undefined {
  if (!entity.texture) return undefined;
  const marker = entity.texture.lastIndexOf("/fg");
  if (marker < 0) return undefined;
  const background =
    entity.texture.slice(0, marker) +
    "/bg" +
    entity.texture.slice(marker + 3);
  return { foreground: entity.texture, background };
}

export function resolveSpinnerStyle(
  theme: VisualTheme,
  variant?: string,
): VisualTheme["spinner"] {
  if (variant && variant !== "theme" && variant in VANILLA_SPINNERS) {
    return VANILLA_SPINNERS[variant as keyof typeof VANILLA_SPINNERS];
  }
  return theme.spinner;
}

export function spinnerCenter(entity: MapEntity): { x: number; y: number } {
  return {
    x: entity.bounds.x + entity.bounds.width * 0.5,
    y: entity.bounds.y + entity.bounds.height * 0.5,
  };
}

export function spinnersConnect(left: MapEntity, right: MapEntity): boolean {
  const a = spinnerCenter(left);
  const b = spinnerCenter(right);
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  // CrystalStaticSpinner.CreateSprites links matching neighbors whose
  // squared center distance is below 576 (24 px).
  return dx * dx + dy * dy < 576;
}

export function spinnerHue(
  position: { x: number; y: number },
  frame: number,
): string {
  // Mirrors CrystalStaticSpinner.GetHue: 280 px loop, 50 units/second,
  // hue 0.4..0.8, saturation 0.4 and value 0.9.
  const loop =
    ((((Math.hypot(position.x, position.y) + (frame / 60) * 50) % 280) + 280) %
      280) /
    280;
  const yoYo = loop <= 0.5 ? loop * 2 : (1 - loop) * 2;
  const hue = 0.4 + yoYo * 0.4;
  const quantizedHue = Math.round(hue * 48) / 48;
  return hsvToHex(quantizedHue, 0.4, 0.9);
}

function hsvToHex(hue: number, saturation: number, value: number): string {
  const section = hue * 6;
  const index = Math.floor(section);
  const fraction = section - index;
  const p = value * (1 - saturation);
  const q = value * (1 - fraction * saturation);
  const t = value * (1 - (1 - fraction) * saturation);
  const [red, green, blue] = [
    [value, t, p],
    [q, value, p],
    [p, value, t],
    [p, q, value],
    [t, p, value],
    [value, p, q],
  ][index % 6];
  return `#${[red, green, blue]
    .map((channel) =>
      Math.round(channel * 255)
        .toString(16)
        .padStart(2, "0"),
    )
    .join("")}`;
}
