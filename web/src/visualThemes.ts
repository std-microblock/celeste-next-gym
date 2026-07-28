export type VisualThemeId =
  | 'forsaken-city'
  | 'old-site'
  | 'celestial-resort'
  | 'golden-ridge'
  | 'summit'
  | 'sj-beginner-gym'
  | 'sj-intermediate-gym'
  | 'sj-advanced-gym'
  | 'sj-expert-gym'
  | 'sj-grandmaster-gym'
  | 'sj-beginner-lobby'
  | 'sj-intermediate-lobby'
  | 'sj-advanced-lobby'
  | 'sj-expert-lobby'
  | 'sj-grandmaster-lobby'

export type VisualThemeCollectionId = 'celeste' | 'strawberry-jam'
export type VisualThemeTileLayout = 'vanilla' | 'sj-gym'

export interface VisualThemeLayer {
  key: string
  opacity?: number
  repeat?: boolean
  y?: number
}

export interface VisualTheme {
  id: VisualThemeId
  label: string
  chapter: string
  collection: VisualThemeCollectionId
  tileset: string
  /** Atlas prefix without the _up/_down/_left/_right suffix. */
  spike: string
  spinner: {
    foreground: string
    background: string
    rainbow?: boolean
  }
  tileLayout?: VisualThemeTileLayout
  background: string
  layers: readonly VisualThemeLayer[]
  stars?: boolean
}

export const VISUAL_THEMES: readonly VisualTheme[] = [
  {
    id: 'forsaken-city',
    label: '遗忘之城',
    chapter: 'CHAPTER 1',
    collection: 'celeste',
    tileset: 'tilesets/dirt',
    spike: 'danger/spikes/default',
    spinner: { foreground: 'danger/crystal/fg_blue', background: 'danger/crystal/bg_blue' },
    background: '#11172f',
    layers: [
      { key: 'bgs/01/bg0' },
      { key: 'bgs/01/bg1' },
      { key: 'bgs/01/bg2' },
    ],
  },
  {
    id: 'old-site',
    label: '旧址',
    chapter: 'CHAPTER 2',
    collection: 'celeste',
    tileset: 'tilesets/stone',
    spike: 'danger/spikes/default',
    spinner: { foreground: 'danger/crystal/fg_blue', background: 'danger/crystal/bg_blue' },
    background: '#100c2f',
    layers: [],
    stars: true,
  },
  {
    id: 'celestial-resort',
    label: '天镜山庄',
    chapter: 'CHAPTER 3',
    collection: 'celeste',
    tileset: 'tilesets/wood',
    spike: 'danger/spikes/default',
    spinner: { foreground: 'danger/crystal/fg_red', background: 'danger/crystal/bg_red' },
    background: '#181027',
    layers: [
      { key: 'bgs/03/bg0' },
      { key: 'bgs/03/bg1' },
      { key: 'bgs/03/bg2' },
      { key: 'bgs/03/bg3' },
      { key: 'bgs/03/fg0' },
    ],
  },
  {
    id: 'golden-ridge',
    label: '黄金山脊',
    chapter: 'CHAPTER 4',
    collection: 'celeste',
    tileset: 'tilesets/cliffside',
    spike: 'danger/spikes/cliffside',
    spinner: { foreground: 'danger/crystal/fg_blue', background: 'danger/crystal/bg_blue' },
    background: '#6d4d79',
    layers: [
      { key: 'bgs/04/bg0' },
      { key: 'bgs/04/bg1' },
      { key: 'bgs/04/bgCloud', y: 50, opacity: .9 },
    ],
  },
  {
    id: 'summit',
    label: '山顶',
    chapter: 'CHAPTER 7',
    collection: 'celeste',
    tileset: 'tilesets/summit',
    spike: 'danger/spikes/outline',
    spinner: { foreground: 'danger/crystal/fg_white', background: 'danger/crystal/bg_white', rainbow: true },
    background: '#2b2660',
    layers: [
      { key: 'bgs/07/bg0' },
      { key: 'bgs/07/00/bg1' },
      { key: 'bgs/07/00/bg2' },
    ],
  },
  {
    id: 'sj-beginner-gym',
    label: '初级训练场',
    chapter: 'BEGINNER GYM',
    collection: 'strawberry-jam',
    tileset: 'sj/tilesets/gym/beginner',
    spike: 'sj/spikes/gym/beg',
    spinner: { foreground: 'danger/crystal/fg_blue', background: 'danger/crystal/bg_blue' },
    tileLayout: 'sj-gym',
    background: '#071323',
    layers: [{ key: 'sj/bgs/gym/beginner-dark', repeat: true }],
  },
  {
    id: 'sj-intermediate-gym',
    label: '中级训练场',
    chapter: 'INTERMEDIATE GYM',
    collection: 'strawberry-jam',
    tileset: 'sj/tilesets/gym/intermediate',
    spike: 'sj/spikes/gym/int',
    spinner: { foreground: 'danger/crystal/fg_purple', background: 'danger/crystal/bg_purple' },
    tileLayout: 'sj-gym',
    background: '#210a0b',
    layers: [{ key: 'sj/bgs/gym/intermediate-dark', repeat: true }],
  },
  {
    id: 'sj-advanced-gym',
    label: '高级训练场',
    chapter: 'ADVANCED GYM',
    collection: 'strawberry-jam',
    tileset: 'sj/tilesets/gym/advanced',
    spike: 'sj/spikes/gym/adv',
    spinner: { foreground: 'danger/crystal/fg_red', background: 'danger/crystal/bg_red' },
    tileLayout: 'sj-gym',
    background: '#191707',
    layers: [{ key: 'sj/bgs/gym/advanced-dark', repeat: true }],
  },
  {
    id: 'sj-expert-gym',
    label: '专家训练场',
    chapter: 'EXPERT GYM',
    collection: 'strawberry-jam',
    tileset: 'sj/tilesets/gym/expert',
    spike: 'sj/spikes/gym/exp',
    spinner: { foreground: 'danger/crystal/fg_white', background: 'danger/crystal/bg_white', rainbow: true },
    tileLayout: 'sj-gym',
    background: '#1b0c04',
    layers: [{ key: 'sj/bgs/gym/expert-dark', repeat: true }],
  },
  {
    id: 'sj-grandmaster-gym',
    label: '宗师训练场',
    chapter: 'GRANDMASTER GYM',
    collection: 'strawberry-jam',
    tileset: 'sj/tilesets/gym/grandmaster',
    spike: 'sj/spikes/gym/gm',
    spinner: { foreground: 'danger/crystal/fg_white', background: 'danger/crystal/bg_white', rainbow: true },
    tileLayout: 'sj-gym',
    background: '#190419',
    layers: [{ key: 'sj/bgs/gym/grandmaster-dark', repeat: true }],
  },
  {
    id: 'sj-beginner-lobby',
    label: '蓝空海湾',
    chapter: 'BEGINNER LOBBY',
    collection: 'strawberry-jam',
    tileset: 'sj/tilesets/lobby/beginner-cliff',
    spike: 'sj/spikes/lobby/beginner-bramble',
    spinner: { foreground: 'sj/spinners/beginner/fg', background: 'sj/spinners/beginner/bg' },
    background: '#8dc8ec',
    layers: [
      { key: 'sj/bgs/lobby/beginner/sky' },
      { key: 'sj/bgs/lobby/beginner/clouds' },
      { key: 'sj/bgs/lobby/beginner/islands' },
    ],
  },
  {
    id: 'sj-intermediate-lobby',
    label: '森林遗迹',
    chapter: 'INTERMEDIATE LOBBY',
    collection: 'strawberry-jam',
    tileset: 'sj/tilesets/lobby/intermediate-girder',
    spike: 'sj/spikes/lobby/intermediate-pixel',
    spinner: { foreground: 'sj/spinners/intermediate/fg', background: 'sj/spinners/intermediate/bg' },
    background: '#25204f',
    layers: [
      { key: 'sj/bgs/lobby/intermediate/skybox' },
      { key: 'sj/bgs/lobby/intermediate/hills' },
      { key: 'sj/bgs/lobby/intermediate/foreground-hills' },
    ],
  },
  {
    id: 'sj-advanced-lobby',
    label: '落日山脊',
    chapter: 'ADVANCED LOBBY',
    collection: 'strawberry-jam',
    tileset: 'sj/tilesets/lobby/advanced-cloud',
    spike: 'sj/spikes/lobby/advanced-orange',
    spinner: { foreground: 'sj/spinners/advanced/fg', background: 'sj/spinners/advanced/bg' },
    background: '#351b55',
    layers: [
      { key: 'sj/bgs/lobby/advanced/sunset' },
      { key: 'sj/bgs/lobby/advanced/mountains' },
      { key: 'sj/bgs/lobby/advanced/dunes' },
    ],
  },
  {
    id: 'sj-expert-lobby',
    label: '星海花园',
    chapter: 'EXPERT LOBBY',
    collection: 'strawberry-jam',
    tileset: 'sj/tilesets/lobby/expert-vegetation',
    spike: 'sj/spikes/lobby/expert-space',
    spinner: { foreground: 'danger/crystal/fg_white', background: 'danger/crystal/bg_white', rainbow: true },
    background: '#090511',
    layers: [
      { key: 'sj/bgs/lobby/expert/space' },
      { key: 'sj/bgs/lobby/expert/nebulae' },
      { key: 'sj/bgs/lobby/expert/planets' },
    ],
  },
  {
    id: 'sj-grandmaster-lobby',
    label: '金色天际',
    chapter: 'GRANDMASTER LOBBY',
    collection: 'strawberry-jam',
    tileset: 'sj/tilesets/lobby/grandmaster-grass',
    spike: 'sj/spikes/lobby/grandmaster-marble',
    spinner: { foreground: 'danger/crystal/fg_purple', background: 'danger/crystal/bg_purple' },
    background: '#eed095',
    layers: [
      { key: 'sj/bgs/lobby/grandmaster/sky' },
      { key: 'sj/bgs/lobby/grandmaster/mountains' },
      { key: 'sj/bgs/lobby/grandmaster/clouds' },
    ],
  },
]

export const VISUAL_THEME_COLLECTIONS: readonly { id: VisualThemeCollectionId; label: string }[] = [
  { id: 'celeste', label: 'Celeste 原版' },
  { id: 'strawberry-jam', label: 'Strawberry Jam 2021' },
]

export const DEFAULT_VISUAL_THEME_ID: VisualThemeId = 'forsaken-city'

export function isVisualThemeId(value: string | null): value is VisualThemeId {
  return VISUAL_THEMES.some((theme) => theme.id === value)
}

export function visualThemeById(id: VisualThemeId): VisualTheme {
  return VISUAL_THEMES.find((theme) => theme.id === id) ?? VISUAL_THEMES[0]
}
