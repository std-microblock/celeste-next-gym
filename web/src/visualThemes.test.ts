import { describe, expect, it } from 'vitest'
import { DEFAULT_VISUAL_THEME_ID, VISUAL_THEMES, isVisualThemeId, visualThemeById, type VisualThemeId } from './visualThemes'

describe('visual themes', () => {
  it('exposes five distinct chapter themes backed by original tilesets', () => {
    expect(VISUAL_THEMES).toHaveLength(5)
    expect(new Set(VISUAL_THEMES.map((theme) => theme.id)).size).toBe(VISUAL_THEMES.length)
    expect(new Set(VISUAL_THEMES.map((theme) => theme.tileset)).size).toBe(VISUAL_THEMES.length)
    for (const theme of VISUAL_THEMES) expect(theme.tileset).toMatch(/^tilesets\//)
  })

  it('validates persisted ids and safely resolves the default', () => {
    expect(isVisualThemeId('summit')).toBe(true)
    expect(isVisualThemeId('not-a-theme')).toBe(false)
    expect(isVisualThemeId(null)).toBe(false)
    expect(visualThemeById('not-a-theme' as VisualThemeId).id).toBe(DEFAULT_VISUAL_THEME_ID)
  })
})
