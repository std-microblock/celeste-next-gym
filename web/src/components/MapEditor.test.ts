import { describe, expect, it } from 'vitest'
import { createEditorEntity, snapToGrid } from './MapEditor'

describe('map editor helpers', () => {
  it('snaps coordinates relative to the room origin', () => {
    expect(snapToGrid(18)).toBe(16)
    expect(snapToGrid(18, 2)).toBe(18)
    expect(snapToGrid(-5, -16)).toBe(-8)
  })

  it('creates simulator-ready entities from palette templates', () => {
    expect(createEditorEntity('spikes', 40, 80)).toEqual({
      kind: 'spikes',
      bounds: { x: 40, y: 80, width: 32, height: 3 },
      direction: { x: 0, y: -1 },
      name: 'spikesUp',
    })
    expect(createEditorEntity('unknown', 0, 0)).toBeNull()
  })
})
