import type { Vec2 } from './model'

const HAIR_OFFSETS: Record<string, string[]> = {
  idle: ['0,-2', '0,-2', '0,-2', '0,-2', '0,-1', '0,-1', '0,-1', '0,-1', '0,-1'],
  runSlow: ['1,-2', '1,-1', '1,-1', '1,-1', '1,-3', '1,-2', '1,-1', '1,-1', '1,-1', '1,-1', '1,-3', '1,-2'],
  runFast: ['1,-2', '1,-1', '1,-1', '1,-1', '1,-3', '1,-2', '1,-1', '1,-1', '1,-1', '1,-1', '1,-3', '1,-2'],
  dash: ['2,0', '2,0', '2,0', '2,1'],
  dreamDash: ['2,0', '1,0', '0,0', '0,0', '1,1', '2,1', '2,1', '2,0', '1,0', '1,0', '1,0', '1,0', '1,0', '1,0', '1,0', '1,0', '1,0', '0,0', '1,0', '1,0', '2,1'],
  jumpSlow: ['1,-3', '1,-3', '1,-2', '0,-2'],
  jumpFast: ['1,-3', '1,-3', '1,-2', '0,-2'],
  climb: ['0,-2', '0,-2', '0,-2', '0,-2', '-1,-2', '-1,-2', '0,-1', '1,-2:1', '2,-2:2'],
  duck: ['0,3'],
  swim: ['0,-2', '0,-2', '0,-2', '0,-2', '0,-2', '0,-2', '0,-3', '0,-3', '0,-2', '0,-2', '0,-2', '0,-2', '1,-1', '1,-1', '1,0', '1,0', '1,0', '1,0'],
  startStarFly: ['0,-2', '0,-2', '0,-2', '0,-2'],
  starFly: ['0,3'],
}

export function playerHairMetadata(key: string | undefined): { offset: Vec2; frame: number } {
  const name = key?.slice(key.lastIndexOf('/') + 1)
  if (!name) return { offset: { x: 0, y: -2 }, frame: 0 }
  const numbered = name.match(/^(.+?)(\d+)$/)
  const animation = numbered?.[1] ?? name
  const index = numbered ? Number(numbered[2]) : 0
  const raw = HAIR_OFFSETS[animation]?.[index] ?? '0,-2'
  const [position, frame = '0'] = raw.split(':')
  const [x, y] = position.split(',').map(Number)
  return { offset: { x, y }, frame: Number(frame) }
}
