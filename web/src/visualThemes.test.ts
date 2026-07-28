import { describe, expect, it } from 'vitest'
import strawberryJamAtlas from '../public/assets/strawberry-jam/gameplay/theme-selected.json'
import { DEFAULT_VISUAL_THEME_ID, VISUAL_THEME_COLLECTIONS, VISUAL_THEMES, isVisualThemeId, visualThemeById, type VisualThemeId } from './visualThemes'

describe('visual themes', () => {
  it('exposes the five original themes and ten Strawberry Jam themes', () => {
    expect(VISUAL_THEMES).toHaveLength(15)
    expect(new Set(VISUAL_THEMES.map((theme) => theme.id)).size).toBe(VISUAL_THEMES.length)
    expect(new Set(VISUAL_THEMES.map((theme) => theme.tileset)).size).toBe(VISUAL_THEMES.length)
    expect(VISUAL_THEMES.filter((theme) => theme.collection === 'celeste')).toHaveLength(5)
    expect(VISUAL_THEMES.filter((theme) => theme.collection === 'strawberry-jam')).toHaveLength(10)
    expect(VISUAL_THEME_COLLECTIONS.map((collection) => collection.id)).toEqual(['celeste', 'strawberry-jam'])
    for (const theme of VISUAL_THEMES) expect(theme.tileset).toMatch(/^(tilesets|sj\/tilesets)\//)
    expect(visualThemeById('celestial-resort').spike).toBe('danger/spikes/default')
    expect(visualThemeById('golden-ridge').spike).toBe('danger/spikes/cliffside')
    expect(visualThemeById('summit').spike).toBe('danger/spikes/outline')
  })

  it('includes every Strawberry Jam gym tier with its native autotiler layout', () => {
    const gyms = VISUAL_THEMES.filter((theme) => theme.id.endsWith('-gym'))
    expect(gyms.map((theme) => theme.id)).toEqual([
      'sj-beginner-gym',
      'sj-intermediate-gym',
      'sj-advanced-gym',
      'sj-expert-gym',
      'sj-grandmaster-gym',
    ])
    for (const gym of gyms) {
      expect(gym.tileLayout).toBe('sj-gym')
      expect(gym.layers).toEqual([expect.objectContaining({ repeat: true })])
    }
  })

  it('backs every Strawberry Jam theme reference with a packed source texture', () => {
    const entries = strawberryJamAtlas.entries as Record<string, unknown>
    for (const theme of VISUAL_THEMES.filter((candidate) => candidate.collection === 'strawberry-jam')) {
      expect(entries[theme.tileset]).toBeDefined()
      for (const layer of theme.layers) {
        expect(entries[layer.key]).toBeDefined()
      }
      for (const direction of ['up', 'down', 'left', 'right']) {
        expect(Object.keys(entries).some((key) => key.startsWith(`${theme.spike}_${direction}`))).toBe(true)
      }
      if (theme.spinner.foreground.startsWith('sj/')) {
        expect(Object.keys(entries).some((key) => key.startsWith(theme.spinner.foreground))).toBe(true)
        expect(Object.keys(entries).some((key) => key.startsWith(theme.spinner.background))).toBe(true)
      }
    }

    for (const theme of VISUAL_THEMES.filter((candidate) => candidate.tileLayout === 'sj-gym')) {
      const entry = strawberryJamAtlas.entries[theme.tileset as keyof typeof strawberryJamAtlas.entries]
      expect(entry).toEqual(expect.objectContaining({ width: 24, height: 136 }))
    }
  })

  it('validates persisted ids and safely resolves the default', () => {
    expect(isVisualThemeId('summit')).toBe(true)
    expect(isVisualThemeId('not-a-theme')).toBe(false)
    expect(isVisualThemeId(null)).toBe(false)
    expect(visualThemeById('not-a-theme' as VisualThemeId).id).toBe(DEFAULT_VISUAL_THEME_ID)
  })
})
