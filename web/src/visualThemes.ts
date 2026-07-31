import { ROOM_THEMES } from "./roomThemes";

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
  background: string;
  layers: readonly VisualThemeLayer[];
  stars?: boolean;
}

export const VISUAL_THEMES: readonly VisualTheme[] = ROOM_THEMES;

export const VISUAL_THEME_COLLECTIONS: readonly {
  id: VisualThemeCollectionId;
  label: string;
  /** URL prefix of the Gameplay atlas JSON/PNG pair this collection uses. */
  atlas?: string;
}[] = [
  {
    id: "celeste-rooms",
    label: "Celeste \u5b98\u65b9",
    atlas: "assets/original/gameplay/gameplay-selected",
  },
  {
    id: "strawberry-jam-rooms",
    label: "\u8349\u8393\u9171 2021",
    atlas: "assets/strawberry-jam/gameplay/room-theme-assets",
  },
  {
    id: "cny2024-rooms",
    label: "\u6625\u8282\u5408\u4f5c 2024",
    atlas: "assets/cny2024/gameplay/room-theme-assets",
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
