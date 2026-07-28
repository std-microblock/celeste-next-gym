import type { MapEntity, Vec2 } from './model'

export type SpikeDirection = 'up' | 'down' | 'left' | 'right'

export function spikeTexturePrefixes(style: string, direction: SpikeDirection): string[] {
  const preferred = `${style}_${direction}`
  const fallback = `danger/spikes/default_${direction}`
  return preferred === fallback ? [preferred] : [preferred, fallback]
}

export interface SpikePlacement {
  x: number
  y: number
  originX: number
  originY: number
}

export function spikeDirection(direction: Vec2): SpikeDirection {
  if (direction.x < 0) return 'left'
  if (direction.x > 0) return 'right'
  if (direction.y > 0) return 'down'
  return 'up'
}

export function spikePlacement(
  box: MapEntity['bounds'],
  direction: SpikeDirection,
  index: number,
  frameWidth: number,
  frameHeight: number,
): SpikePlacement {
  const offset = (index + .5) * 8
  if (direction === 'up') {
    return { x: box.x + offset, y: box.y + box.height + 1, originX: frameWidth / 2, originY: frameHeight }
  }
  if (direction === 'down') {
    return { x: box.x + offset, y: box.y - 1, originX: frameWidth / 2, originY: 0 }
  }
  if (direction === 'left') {
    return { x: box.x + box.width + 1, y: box.y + offset, originX: frameWidth, originY: frameHeight / 2 }
  }
  return { x: box.x - 1, y: box.y + offset, originX: 0, originY: frameHeight / 2 }
}
