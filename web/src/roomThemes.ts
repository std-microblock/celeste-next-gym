import roomThemesData from "../public/assets/strawberry-jam/room-themes.json";
import type { VisualTheme, VisualThemeCollectionId, VisualThemeLayer } from "./visualThemes";

/**
 * Themes extracted from a mod pack by scripts/extract-mod-themes.mjs: one
 * theme per room, carrying the room's own tileset + autotiler rules, spike
 * type, and parallax background layers. They are listed under the
 * "strawberry-jam-rooms" collection and render through the same atlas the
 * curated Strawberry Jam themes use.
 */

interface RoomThemeData {
  id: string;
  label: string;
  mapFile: string;
  room: string;
  tileset: string | null;
  tileRules: readonly (readonly [string, readonly [number, number]])[] | null;
  centerTile: readonly [number, number] | null;
  spike: string;
  spinner: {
    foreground: string;
    background: string;
    kind?: "crystal" | "sprite";
  } | null;
  background: string;
  layers: readonly {
    key: string;
    scrollX: number;
    scrollY: number;
    loopX: boolean;
    loopY: boolean;
    speedX: number;
    speedY: number;
    opacity: number;
  }[];
}

export const ROOM_THEMES_COLLECTION_ID = "strawberry-jam-rooms" as const;

export const ROOM_THEMES: readonly VisualTheme[] = (
  roomThemesData.rooms as readonly RoomThemeData[]
).map((room) => ({
  id: `room:${room.id}`,
  label: room.label,
  chapter: room.mapFile,
  collection: ROOM_THEMES_COLLECTION_ID as VisualThemeCollectionId,
  tileset: room.tileset ?? "tilesets/dirt",
  ...(room.tileRules ? { tileRules: room.tileRules } : {}),
  ...(room.centerTile ? { centerTile: room.centerTile } : {}),
  spike: room.spike,
  spinner: room.spinner ?? {
    foreground: "danger/crystal/fg_blue",
    background: "danger/crystal/bg_blue",
  },
  background: room.background,
  layers: room.layers.map(
    (layer): VisualThemeLayer => ({
      key: layer.key,
      opacity: layer.opacity,
      scrollX: layer.scrollX,
      scrollY: layer.scrollY,
      loopX: layer.loopX,
      loopY: layer.loopY,
      speedX: layer.speedX,
      speedY: layer.speedY,
    }),
  ),
}));
