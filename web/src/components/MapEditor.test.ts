import { describe, expect, it } from 'vitest'
import { createEditorEntity, editorEntityHitBounds, setEditorSpikeDirection, snapToGrid } from './MapEditor'

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

  it('exposes all four spike directions and the crystal spinner', () => {
    expect(createEditorEntity('spikes-down', 8, 16)).toEqual(expect.objectContaining({
      name: 'spikesDown',
      direction: { x: 0, y: 1 },
      bounds: { x: 8, y: 16, width: 32, height: 3 },
    }))
    expect(createEditorEntity('spikes-left', 8, 16)).toEqual(expect.objectContaining({
      name: 'spikesLeft',
      direction: { x: -1, y: 0 },
      bounds: { x: 8, y: 16, width: 3, height: 32 },
    }))
    expect(createEditorEntity('crystal-spinner', 24, 32)).toEqual({
      kind: 'crystal_static_spinner',
      bounds: { x: 24, y: 32, width: 16, height: 12 },
      direction: { x: 0, y: 0 },
      name: 'spinner',
    })
  })

  it('rotates spike bounds while preserving their length and gives thin spikes a usable hit area', () => {
    const spikes = createEditorEntity('spikes-up', 40, 80)!
    const left = setEditorSpikeDirection(spikes, { x: -1, y: 0 })
    expect(left).toEqual(expect.objectContaining({
      name: 'spikesLeft',
      direction: { x: -1, y: 0 },
      bounds: { x: 40, y: 80, width: 3, height: 32 },
    }))
    expect(editorEntityHitBounds(spikes)).toEqual({ x: 40, y: 77, width: 32, height: 9 })
    expect(editorEntityHitBounds(left)).toEqual({ x: 37, y: 80, width: 9, height: 32 })
  })
})
