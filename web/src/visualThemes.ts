import { ROOM_THEMES } from "./roomThemes";
import type { TileSetDef } from "./tileRules";

export type VisualThemeId = | `room:${string}`
  | "forsaken-city"
  | "old-site"
  | "celestial-resort"
  | "golden-ridge"
  | "summit"
  | "sj-beginner-gym"
  | "sj-intermediate-gym"
  | "sj-advanced-gym"
  | "sj-expert-gym"
  | "sj-grandmaster-gym"
  | "sj-beginner-lobby"
  | "sj-intermediate-lobby"
  | "sj-advanced-lobby"
  | "sj-expert-lobby"
  | "sj-grandmaster-lobby";

export type VisualThemeCollectionId =
  | "celeste-rooms"
  | "strawberry-jam-rooms"
  | "cny2024-rooms";
export type VisualThemeTileLayout = "vanilla" | "sj-gym";

export interface VisualThemeLayer {
  key: string;
  opacity?: number;
  repeat?: boolean;
  y?: number;
  /** Parallax scroll factors: 0 = screen-fixed, 1 = world-anchored. */
  scrollX?: number;
  scrollY?: number;
  loopX?: boolean;
  loopY?: boolean;
  /** Automatic drift in pixels/second. */
  speedX?: number;
  speedY?: number;
}

export interface VisualTheme {
  id: VisualThemeId;
  label: string;
  chapter: string;
  collection: VisualThemeCollectionId;
  tileset: string;
  /** Original Celeste spike type (AreaData.Spike or spike entity type attr). */
  spike: string;
  spinner: {
    foreground: string;
    background: string;
    rainbow?: boolean;
    /**
     * "crystal" renders vanilla CrystalStaticSpinner shards from a 24x24
     * sheet; "sprite" draws the sheet centered (SJ custom spinners).
     * Defaults to "crystal" for danger/crystal/* and "sprite" otherwise.
     */
    kind?: "crystal" | "sprite";
  };
  tileLayout?: VisualThemeTileLayout;
  /** Per-room autotiler rules extracted from a mod's ForegroundTiles.xml. */
  tileRules?: readonly (readonly [string, readonly [number, number]])[];
  /** Interior fill tile for tileRules (the XML's mask="center" set). */
  centerTile?: readonly [number, number];
  /** Inner-edge fill tile (the XML's mask="padding" set). */
  paddedTile?: readonly [number, number];
  /** Autotile scan size of the dominant tileset (3x3 unless SJ 5x5). */
  tileScan?: readonly [number, number];
  /** Chars ignored when checking dominant-tileset neighbors. */
  tileIgnores?: string;
  /** Per-char tileset definitions for multi-tileset rooms. */
  tilesets?: Record<string, TileSetDef>;
  /** Small per-theme atlas fetched when this theme is selected. */
  atlasUrl?: string;
  /** Pre-generated thumbnail shown in the theme picker. */
  previewUrl?: string;
  background: string;
  layers: readonly VisualThemeLayer[];
  stars?: boolean;
}

export const VISUAL_THEMES: readonly VisualTheme[] = ROOM_THEMES;

export const VISUAL_THEME_COLLECTIONS: readonly {
  id: VisualThemeCollectionId;
  label: string;
}[] = [
  {
    id: "celeste-rooms",
    label: "Celeste \u5b98\u65b9"
  },
  {
    id: "strawberry-jam-rooms",
    label: "\u8349\u8393\u9171 2021"
  },
  {
    id: "cny2024-rooms",
    label: "\u6625\u8282\u5408\u4f5c 2024"
  },
];

export const DEFAULT_VISUAL_THEME_ID: VisualThemeId = "room:1-ForsakenCity";

export function isVisualThemeId(value: string | null): value is VisualThemeId {
  return VISUAL_THEMES.some((theme) => theme.id === value);
}

export function visualThemeById(id: VisualThemeId): VisualTheme {
  return (
    VISUAL_THEMES.find((theme) => theme.id === id) ??
    VISUAL_THEMES.find((theme) => theme.id === DEFAULT_VISUAL_THEME_ID) ??
    VISUAL_THEMES[0]
  );
}
