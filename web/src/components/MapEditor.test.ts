import { describe, expect, it } from 'vitest'
import { PLAYGROUND } from '../model'
import { createEditorEntity, resizeEditorBounds, snapToGrid } from './MapEditor'

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

  it('creates zip movers with a movable destination node', () => {
    expect(createEditorEntity('zip_mover', 40, 80)).toEqual({
      kind: 'zip_mover',
      bounds: { x: 40, y: 80, width: 32, height: 16 },
      direction: { x: 0, y: 0 },
      nodes: [{ x: 104, y: 80 }],
      name: 'zipMover',
    })
  })

  it('resizes from every corner on the editor grid', () => {
    const bounds = { x: 40, y: 40, width: 32, height: 24 }
    expect(resizeEditorBounds(bounds, 'nw', { x: 17, y: 25 }, PLAYGROUND)).toEqual({ x: 16, y: 24, width: 56, height: 40 })
    expect(resizeEditorBounds(bounds, 'se', { x: 91, y: 83 }, PLAYGROUND)).toEqual({ x: 40, y: 40, width: 48, height: 40 })
  })
})
