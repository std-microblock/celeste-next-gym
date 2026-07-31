import type { GymMap, GymMapBackdrop } from "./model";

/**
 * Backdrop visibility and placement helpers that mirror Celeste's
 * Backdrop.IsVisible / Parallax.Render semantics.
 */

/** Convert a room glob (MapData.ParseLevelsList) into an anchored RegExp. */
export function backdropGlob(pattern: string): RegExp {
  const escaped = pattern.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return new RegExp(`^${escaped.replace(/\\\*/g, ".*")}$`);
}

export function backdropGlobMatches(pattern: string, room: string): boolean {
  return backdropGlob(pattern).test(room);
}

/**
 * Mirror Backdrop.IsVisible: not-flag, required flag, exclude list, then the
 * only list. Session flags default to unset (hidden backdrops stay hidden).
 */
export function backdropVisible(
  backdrop: GymMapBackdrop,
  room: string,
  flags: Record<string, boolean> = {},
): boolean {
  if (
    backdrop.not_flag &&
    backdrop.not_flag.length > 0 &&
    flags[backdrop.not_flag] === true
  )
    return false;
  if (backdrop.flag && backdrop.flag.length > 0 && flags[backdrop.flag] !== true)
    return false;
  if (backdrop.exclude) {
    for (const pattern of backdrop.exclude.split(",")) {
      if (pattern && backdropGlobMatches(pattern, room)) return false;
    }
  }
  if (backdrop.only) {
    const patterns = backdrop.only.split(",").filter((pattern) => pattern.length > 0);
    if (patterns.length > 0 && !patterns.some((pattern) => backdropGlobMatches(pattern, room)))
      return false;
  }
  return true;
}

/** Backdrops from the map's Style/Backgrounds visible in the given room. */
export function visibleMapBackdrops(
  map: GymMap,
  flags: Record<string, boolean> = {},
): GymMapBackdrop[] {
  if (!map.backdrops) return [];
  const room = map.room ?? map.name ?? "";
  return map.backdrops.filter((backdrop) => backdropVisible(backdrop, room, flags));
}

/**
 * World-space anchor for a parallax backdrop under the GameView camera
 * transform. Vanilla draws at screen (Position - Camera * Scroll); the web
 * world transform maps world X to screen offsetX + (X - camera.x) * scale, so
 * the equivalent world anchor is Position + Camera * (1 - Scroll).
 */
export function backdropWorldAnchor(
  backdrop: GymMapBackdrop,
  camera: { x: number; y: number },
  frame: number,
): { x: number; y: number } {
  const time = frame / 60;
  return {
    x: backdrop.x + backdrop.speed_x * time + camera.x * (1 - backdrop.scroll_x),
    y: backdrop.y + backdrop.speed_y * time + camera.y * (1 - backdrop.scroll_y),
  };
}
