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
  | "celeste"
  | "strawberry-jam"
  | "strawberry-jam-rooms";
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

const VISUAL_THEMES_CURATED: readonly VisualTheme[] = [
  {
    id: "forsaken-city",
    label: "遗忘之城",
    chapter: "CHAPTER 1",
    collection: "celeste",
    tileset: "tilesets/dirt",
    spike: "default",
    spinner: {
      foreground: "danger/crystal/fg_blue",
      background: "danger/crystal/bg_blue",
    },
    background: "#11172f",
    layers: [
      { key: "bgs/01/bg0" },
      { key: "bgs/01/bg1" },
      { key: "bgs/01/bg2" },
    ],
  },
  {
    id: "old-site",
    label: "旧址",
    chapter: "CHAPTER 2",
    collection: "celeste",
    tileset: "tilesets/stone",
    spike: "default",
    spinner: {
      foreground: "danger/crystal/fg_blue",
      background: "danger/crystal/bg_blue",
    },
    background: "#100c2f",
    layers: [],
    stars: true,
  },
  {
    id: "celestial-resort",
    label: "天镜山庄",
    chapter: "CHAPTER 3",
    collection: "celeste",
    tileset: "tilesets/wood",
    spike: "default",
    spinner: {
      foreground: "danger/crystal/fg_red",
      background: "danger/crystal/bg_red",
    },
    background: "#181027",
    layers: [
      { key: "bgs/03/bg0" },
      { key: "bgs/03/bg1" },
      { key: "bgs/03/bg2" },
      { key: "bgs/03/bg3" },
      { key: "bgs/03/fg0" },
    ],
  },
  {
    id: "golden-ridge",
    label: "黄金山脊",
    chapter: "CHAPTER 4",
    collection: "celeste",
    tileset: "tilesets/cliffside",
    spike: "cliffside",
    spinner: {
      foreground: "danger/crystal/fg_blue",
      background: "danger/crystal/bg_blue",
    },
    background: "#6d4d79",
    layers: [
      { key: "bgs/04/bg0" },
      { key: "bgs/04/bg1" },
      { key: "bgs/04/bgCloud", y: 50, opacity: 0.9 },
    ],
  },
  {
    id: "summit",
    label: "山顶",
    chapter: "CHAPTER 7",
    collection: "celeste",
    tileset: "tilesets/summit",
    spike: "outline",
    spinner: {
      foreground: "danger/crystal/fg_white",
      background: "danger/crystal/bg_white",
      rainbow: true,
    },
    background: "#2b2660",
    layers: [
      { key: "bgs/07/bg0" },
      { key: "bgs/07/00/bg1" },
      { key: "bgs/07/00/bg2" },
    ],
  },
  {
    id: "sj-beginner-gym",
    label: "初级训练场",
    chapter: "BEGINNER GYM",
    collection: "strawberry-jam",
    tileset: "tilesets/SJ2021/Gym/BeginnerGym",
    spike: "SJ2021/Gym/beg",
    spinner: {
      foreground: "danger/SJ2021/Gym/Orb/fg_beg",
      background: "danger/SJ2021/Gym/Orb/bg_beg"
    },
    tileLayout: "sj-gym",
    background: "#071323",
    layers: [{ key: "bgs/SJ2021/Gym/begGymDarkBG", repeat: true }],
  },
  {
    id: "sj-intermediate-gym",
    label: "中级训练场",
    chapter: "INTERMEDIATE GYM",
    collection: "strawberry-jam",
    tileset: "tilesets/SJ2021/Gym/IntermediateGym",
    spike: "SJ2021/Gym/int",
    spinner: {
      foreground: "danger/SJ2021/Gym/Orb/fg_int",
      background: "danger/SJ2021/Gym/Orb/bg_int"
    },
    tileLayout: "sj-gym",
    background: "#210a0b",
    layers: [{ key: "bgs/SJ2021/Gym/intGymDarkBG", repeat: true }],
  },
  {
    id: "sj-advanced-gym",
    label: "高级训练场",
    chapter: "ADVANCED GYM",
    collection: "strawberry-jam",
    tileset: "tilesets/SJ2021/Gym/AdvancedGym",
    spike: "SJ2021/Gym/adv",
    spinner: {
      foreground: "danger/SJ2021/Gym/Orb/fg_adv",
      background: "danger/SJ2021/Gym/Orb/bg_adv"
    },
    tileLayout: "sj-gym",
    background: "#191707",
    layers: [{ key: "bgs/SJ2021/Gym/advGymDarkBG", repeat: true }],
  },
  {
    id: "sj-expert-gym",
    label: "专家训练场",
    chapter: "EXPERT GYM",
    collection: "strawberry-jam",
    tileset: "tilesets/SJ2021/Gym/ExpertGym",
    spike: "SJ2021/Gym/exp",
    spinner: {
      foreground: "danger/SJ2021/Gym/Orb/fg_exp",
      background: "danger/SJ2021/Gym/Orb/bg_exp"
    },
    tileLayout: "sj-gym",
    background: "#1b0c04",
    layers: [{ key: "bgs/SJ2021/Gym/expGymDarkBG", repeat: true }],
  },
  {
    id: "sj-grandmaster-gym",
    label: "宗师训练场",
    chapter: "GRANDMASTER GYM",
    collection: "strawberry-jam",
    tileset: "tilesets/SJ2021/Gym/GrandmasterGym",
    spike: "SJ2021/Gym/gm",
    spinner: {
      foreground: "danger/SJ2021/Gym/Orb/fg_gm",
      background: "danger/SJ2021/Gym/Orb/bg_gm"
    },
    tileLayout: "sj-gym",
    background: "#190419",
    layers: [{ key: "bgs/SJ2021/Gym/gmGymDarkBG", repeat: true }],
  },
  {
    id: "sj-beginner-lobby",
    label: "蓝空海湾",
    chapter: "BEGINNER LOBBY",
    collection: "strawberry-jam",
    tileset: "tilesets/SJ2021/BeginnerLobby/lobbyCliff",
    spike: "SJ2021/1-Beginner/bramble",
    spinner: {
      foreground: "danger/spikes/SJ2021/1-Beginner/brambles/fg",
      background: "danger/spikes/SJ2021/1-Beginner/brambles/bg"
    },
    background: "#8dc8ec",
    layers: [
      { key: "bgs/SJ2021/BeginnerLobby/main/sky" },
      { key: "bgs/SJ2021/BeginnerLobby/main/clouds" },
      { key: "bgs/SJ2021/BeginnerLobby/main/islands" },
    ],
  },
  {
    id: "sj-intermediate-lobby",
    label: "森林遗迹",
    chapter: "INTERMEDIATE LOBBY",
    collection: "strawberry-jam",
    tileset: "tilesets/SJ2021/Int_Lobby/IntGirderFg",
    spike: "SJ2021/pixelator/v",
    spinner: {
      foreground: "danger/SJ2021/Ceph/Spinner/fg",
      background: "danger/SJ2021/Ceph/Spinner/bg"
    },
    background: "#25204f",
    layers: [
      { key: "bgs/SJ2021/Int_Lobby/skybox" },
      { key: "bgs/SJ2021/Int_Lobby/bghills" },
      { key: "bgs/SJ2021/Int_Lobby/fghills" },
    ],
  },
  {
    id: "sj-advanced-lobby",
    label: "落日山脊",
    chapter: "ADVANCED LOBBY",
    collection: "strawberry-jam",
    tileset: "tilesets/SJ2021/Advanced_Lobby/advCloudSunset",
    spike: "SJ2021/Archire/orange",
    spinner: {
      foreground: "danger/SJ2021/Julia/Spinner/fg",
      background: "danger/SJ2021/Julia/Spinner/bg"
    },
    background: "#351b55",
    layers: [
      { key: "bgs/SJ2021/Advanced Lobby/sunset/sunset" },
      { key: "bgs/SJ2021/Advanced Lobby/sunset/sunsetmountains" },
      { key: "bgs/SJ2021/Advanced Lobby/sunset/sunsetdunes" },
    ],
  },
  {
    id: "sj-expert-lobby",
    label: "星海花园",
    chapter: "EXPERT LOBBY",
    collection: "strawberry-jam",
    tileset: "tilesets/SJ2021/ExpertLobby/spaceVegetation",
    spike: "SJ2021/powerav/space",
    spinner: {
      foreground: "danger/crystal/fg_white",
      background: "danger/crystal/bg_white",
      rainbow: true,
    },
    background: "#090511",
    layers: [
      { key: "bgs/SJ2021/ExpertLobby/space" },
      { key: "bgs/SJ2021/ExpertLobby/nebulae" },
      { key: "bgs/SJ2021/ExpertLobby/planets" },
    ],
  },
  {
    id: "sj-grandmaster-lobby",
    label: "金色天际",
    chapter: "GRANDMASTER LOBBY",
    collection: "strawberry-jam",
    tileset: "tilesets/SJ2021/Grandmaster/elysianGrass",
    spike: "SJ2021/Grandmaster/marble",
    spinner: {
      foreground: "danger/crystal/fg_purple",
      background: "danger/crystal/bg_purple",
    },
    background: "#eed095",
    layers: [
      { key: "bgs/SJ2021/GMLobby/sky" },
      { key: "bgs/SJ2021/GMLobby/mountains" },
      { key: "bgs/SJ2021/GMLobby/cloud-group-1" },
    ],
  },
];

export const VISUAL_THEMES: readonly VisualTheme[] = [
  ...VISUAL_THEMES_CURATED,
  ...ROOM_THEMES,
];

export const VISUAL_THEME_COLLECTIONS: readonly {
  id: VisualThemeCollectionId;
  label: string;
  /** URL prefix of the Gameplay atlas JSON/PNG pair this collection uses. */
  atlas?: string;
}[] = [
  { id: "celeste", label: "Celeste 原版", atlas: "assets/original/gameplay/gameplay-selected" },
  {
    id: "strawberry-jam",
    label: "Strawberry Jam 2021",
    atlas: "assets/strawberry-jam/gameplay/theme-selected",
  },
  {
    id: "strawberry-jam-rooms",
    label: "SJ2021 房间主题",
    atlas: "assets/strawberry-jam/gameplay/room-theme-assets"
  },
];

export const DEFAULT_VISUAL_THEME_ID: VisualThemeId = "forsaken-city";

export function isVisualThemeId(value: string | null): value is VisualThemeId {
  return VISUAL_THEMES.some((theme) => theme.id === value);
}

export function visualThemeById(id: VisualThemeId): VisualTheme {
  return VISUAL_THEMES.find((theme) => theme.id === id) ?? VISUAL_THEMES[0];
}
