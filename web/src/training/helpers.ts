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

export function hyperVariant(options: {
  id: string
  title: string
  summary: string
  map: GymMap
  initial: SimState
  success?: string[]
  routeValidation?: { observeUntil: number | string; success: string[] }
}): TrainingVariant {
  const success = options.success ?? ['!final.dead', 'final.speed.x == 325']
  const document: TrainingDocument = {
    version: 1,
    id: `hyper-${options.id}`,
    technique_id: 'hyper',
    variant_id: options.id,
    variant_title: options.title,
    title: 'Hyper',
    summary: options.summary,
    entry: { input_id: 'diagonal_dash', hint: '按住右下并冲刺。', check: ['!current.dead', 'current.state == state::dash'], failure: { title: '需要右下冲', body: '按住右和下，再按 Dash。' } },
    fuzz: {
      version: 1,
      inputs: [hold('hold_diagonal', ['right', 'down']), press('diagonal_dash', ['dash'], 0), press('jump', ['jump'], 'jump_frame')],
      variables: [{ name: 'jump_frame', range: { from: 5, to: 30 } }],
      observe_until: 'jump_frame+1',
      success,
      objectives: [{ type: 'maximize', expression: 'final.speed.x' }],
      search: { bindings: {}, output: ['best', 'windows', 'coverage'] },
    },
    teaching: { steps: [
      { prompt: '按住右下并冲刺。', order_error: { title: '先右下冲', body: '第一步需要右下冲刺。' }, window_error: { title: '冲刺时机不对', body: '从训练开始时输入右下冲。' } },
      { prompt: '接近最佳点时按 Jump。', order_error: { title: '这里需要 Jump', body: '右下冲后的下一关键输入是跳跃。' }, window_error: { title: '跳跃错过窗口', body: '查看时间线中的绿色可行区间。' } },
    ] },
    assist: { result_sample_after_input_frames: 0, auto_slowdown: { enabled_by_default: true, radius_frames: 12, minimum_multiplier: .85 } },
  }
  return {
    id: options.id,
    title: options.title,
    summary: options.summary,
    map: options.map,
    initial: options.initial,
    document,
    validationFuzz: options.routeValidation ? { ...document.fuzz, observe_until: options.routeValidation.observeUntil, success: options.routeValidation.success } : document.fuzz,
  }
}
