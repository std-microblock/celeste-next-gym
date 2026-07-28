export type VisualThemeId = 'forsaken-city' | 'old-site' | 'celestial-resort' | 'golden-ridge' | 'summit'

export interface VisualThemeLayer {
  key: string
  opacity?: number
  y?: number
}

export interface VisualTheme {
  id: VisualThemeId
  label: string
  chapter: string
  tileset: string
  background: string
  layers: readonly VisualThemeLayer[]
  stars?: boolean
}

export const VISUAL_THEMES: readonly VisualTheme[] = [
  {
    id: 'forsaken-city',
    label: '遗忘之城',
    chapter: 'CHAPTER 1',
    tileset: 'tilesets/dirt',
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
    tileset: 'tilesets/stone',
    background: '#100c2f',
    layers: [],
    stars: true,
  },
  {
    id: 'celestial-resort',
    label: '天镜山庄',
    chapter: 'CHAPTER 3',
    tileset: 'tilesets/wood',
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
    tileset: 'tilesets/cliffside',
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
    tileset: 'tilesets/summit',
    background: '#2b2660',
    layers: [
      { key: 'bgs/07/bg0' },
      { key: 'bgs/07/00/bg1' },
      { key: 'bgs/07/00/bg2' },
    ],
  },
]

export const DEFAULT_VISUAL_THEME_ID: VisualThemeId = 'forsaken-city'

export function isVisualThemeId(value: string | null): value is VisualThemeId {
  return VISUAL_THEMES.some((theme) => theme.id === value)
}

export function visualThemeById(id: VisualThemeId): VisualTheme {
  return VISUAL_THEMES.find((theme) => theme.id === id) ?? VISUAL_THEMES[0]
}
