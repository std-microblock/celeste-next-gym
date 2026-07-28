import type { GymMap, SimState } from '../model.ts'
import type { TrainingDocument, TrainingVariant } from './catalog.ts'

export const hold = (id: string, keys: string[]) => ({ id, keys, at: 0, held_time: 'hold::inf', verify: false })
export const press = (id: string, keys: string[], at: number | string) => ({ id, keys, at, verify: true })

export function room(name: string, roomName: string, options: {
  width?: number
  floorY?: number
  floorSegments?: GymMap['solids']
  spawn?: { x: number; y: number }
  entities?: GymMap['entities']
} = {}): GymMap {
  const width = options.width ?? 480
  const height = 270
  const floorY = options.floorY ?? 240
  return {
    name,
    room: roomName,
    bounds: { x: 0, y: 0, width, height },
    spawn: options.spawn ?? { x: 64, y: floorY },
    solids: [
      ...(options.floorSegments ?? [{ x: 0, y: floorY, width, height: height - floorY }]),
      { x: 0, y: 0, width: 8, height },
      { x: width - 8, y: 0, width: 8, height },
    ],
    entities: options.entities ?? [],
    source_package: null,
  }
}

export function snapshot(pos: { x: number; y: number }, overrides: Partial<SimState> = {}): SimState {
  return {
    pos,
    speed: { x: 0, y: 0 },
    state: 'Normal',
    facing: true,
    dashes: 1,
    stamina: 110,
    on_ground: true,
    ducking: false,
    can_dream_dash: true,
    dead: false,
    death_freeze_pending: false,
    respawn_frames: 0,
    dash_dir: { x: 0, y: 0 },
    ...overrides,
  }
}
