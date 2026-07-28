import { describe, expect, it } from 'vitest'
import { spikeDirection, spikePlacement, spikeTexturePrefixes } from './spikeRendering'

describe('spike rendering', () => {
  it('selects the texture direction from the collision normal', () => {
    expect(spikeDirection({ x: 0, y: -1 })).toBe('up')
    expect(spikeDirection({ x: 0, y: 1 })).toBe('down')
    expect(spikeDirection({ x: -1, y: 0 })).toBe('left')
    expect(spikeDirection({ x: 1, y: 0 })).toBe('right')
  })

  it('anchors each sprite to the entity position outside its three-pixel collider', () => {
    const horizontal = { x: 10, y: 20, width: 16, height: 3 }
    const vertical = { x: 10, y: 20, width: 3, height: 16 }

    expect(spikePlacement(horizontal, 'up', 0, 8, 9)).toEqual({ x: 14, y: 24, originX: 4, originY: 9 })
    expect(spikePlacement(horizontal, 'down', 0, 8, 9)).toEqual({ x: 14, y: 19, originX: 4, originY: 0 })
    expect(spikePlacement(vertical, 'left', 0, 9, 8)).toEqual({ x: 14, y: 24, originX: 9, originY: 4 })
    expect(spikePlacement(vertical, 'right', 0, 9, 8)).toEqual({ x: 9, y: 24, originX: 0, originY: 4 })
  })

  it('falls back to vanilla spikes when a custom theme is missing one direction', () => {
    expect(spikeTexturePrefixes('sj/spikes/gym/int', 'left')).toEqual([
      'sj/spikes/gym/int_left',
      'danger/spikes/default_left',
    ])
    expect(spikeTexturePrefixes('danger/spikes/default', 'up')).toEqual(['danger/spikes/default_up'])
  })
})
