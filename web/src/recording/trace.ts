import {
  buttonsToInput,
  type FrameButtons,
  type GymMap,
  type SimInput,
  type SimState,
} from '../model'

export const TRACE_FORMAT = 'celeste-next-gym-trace' as const
export const TRACE_VERSION = 1 as const
export const TRACE_TOLERANCE = 0.01

export interface PortableState {
  frame: number
  pos: [number, number]
  speed: [number, number]
  state: string | number
  facing: boolean | 'Left' | 'Right'
  dashes: number
  stamina: number
  on_ground: boolean
  ducking: boolean
  dead: boolean
  snapshot?: SimState
  fields?: Record<string, unknown>
}

export interface FrameTrace {
  format: typeof TRACE_FORMAT
  version: typeof TRACE_VERSION
  source: 'web' | 'game'
  recorded_at: string
  map: {
    sid: string
    room: string
    binary: string
    data?: GymMap
  }
  inputs: SimInput[]
  states: PortableState[]
}

export interface TraceComparison {
  matched: boolean
  compared_frames: number
  max_position_error: number
  max_speed_error: number
  first_mismatch: number | null
  reason: string | null
}

export function createWebTrace(
  map: GymMap,
  buttons: readonly FrameButtons[],
  states: readonly (SimState | undefined)[],
  endFrame: number,
  recordedAt = new Date().toISOString(),
): FrameTrace {
  const end = Math.max(0, Math.min(Math.round(endFrame), buttons.length))
  const selectedStates = states.slice(0, end + 1)
  const missing = selectedStates.findIndex((state) => state === undefined)
  if (missing >= 0) throw new Error(`帧 ${missing} 尚未计算，不能导出逐帧数据`)
  const inputs = buttons.slice(0, end).map((value, index) => (
    buttonsToInput(value, index > 0 ? buttons[index - 1] : undefined)
  ))
  return {
    format: TRACE_FORMAT,
    version: TRACE_VERSION,
    source: 'web',
    recorded_at: recordedAt,
    map: {
      sid: 'CelesteGymPlayground/Playground',
      room: 'playground',
      binary: 'maps/CelesteGymPlayground/Playground.bin',
      data: structuredClone(map),
    },
    inputs,
    states: selectedStates.map((state, frame) => portableState(state!, frame)),
  }
}

export function parseTrace(value: unknown): FrameTrace {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('逐帧数据必须是 JSON 对象')
  const trace = value as Partial<FrameTrace>
  if (trace.format !== TRACE_FORMAT || trace.version !== TRACE_VERSION) throw new Error('不是 Celeste Next Gym trace v1')
  if (trace.source !== 'web' && trace.source !== 'game') throw new Error('trace source 必须是 web 或 game')
  if (!Array.isArray(trace.inputs) || !Array.isArray(trace.states)) throw new Error('trace 缺少 inputs 或 states')
  if (trace.states.length !== trace.inputs.length + 1) throw new Error('trace states 数量必须等于 inputs + 1')
  trace.states.forEach((state, index) => validatePortableState(state, index))
  return trace as FrameTrace
}

export function compareTraces(
  actual: FrameTrace,
  expected: FrameTrace,
  tolerance = TRACE_TOLERANCE,
): TraceComparison {
  const compared = Math.min(actual.states.length, expected.states.length)
  let maxPosition = 0
  let maxSpeed = 0
  let firstMismatch: number | null = null
  let reason: string | null = null
  for (let frame = 0; frame < compared; frame += 1) {
    const left = actual.states[frame]
    const right = expected.states[frame]
    const positionError = maxVectorError(left.pos, right.pos)
    const speedError = maxVectorError(left.speed, right.speed)
    maxPosition = Math.max(maxPosition, positionError)
    maxSpeed = Math.max(maxSpeed, speedError)
    if (firstMismatch !== null) continue
    const coreMismatch = positionError > tolerance
      || speedError > tolerance
      || normalizeState(left.state) !== normalizeState(right.state)
      || normalizeFacing(left.facing) !== normalizeFacing(right.facing)
      || left.dashes !== right.dashes
      || Math.abs(left.stamina - right.stamina) > tolerance
      || left.on_ground !== right.on_ground
      || left.ducking !== right.ducking
      || left.dead !== right.dead
    if (coreMismatch) {
      firstMismatch = frame
      reason = `核心字段在 F${frame} 首次不一致`
    }
  }
  if (firstMismatch === null && actual.states.length !== expected.states.length) {
    firstMismatch = compared
    reason = `帧数不同：${actual.states.length} / ${expected.states.length}`
  }
  return {
    matched: firstMismatch === null,
    compared_frames: compared,
    max_position_error: maxPosition,
    max_speed_error: maxSpeed,
    first_mismatch: firstMismatch,
    reason,
  }
}

function portableState(state: SimState, frame: number): PortableState {
  return {
    frame,
    pos: [state.pos.x, state.pos.y],
    speed: [state.speed.x, state.speed.y],
    state: state.state,
    facing: state.facing,
    dashes: state.dashes,
    stamina: state.stamina,
    on_ground: state.on_ground,
    ducking: state.ducking,
    dead: state.dead,
    snapshot: structuredClone(state),
  }
}

function validatePortableState(value: unknown, frame: number): asserts value is PortableState {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error(`states[${frame}] 不是对象`)
  const state = value as Partial<PortableState>
  if (!vector(state.pos) || !vector(state.speed)) throw new Error(`states[${frame}] 的位置或速度无效`)
  if (typeof state.state !== 'string' && !Number.isInteger(state.state)) throw new Error(`states[${frame}].state 无效`)
  if (typeof state.facing !== 'boolean' && state.facing !== 'Left' && state.facing !== 'Right') throw new Error(`states[${frame}].facing 无效`)
  for (const key of ['dashes', 'stamina'] as const) {
    if (typeof state[key] !== 'number' || !Number.isFinite(state[key])) throw new Error(`states[${frame}].${key} 无效`)
  }
  for (const key of ['on_ground', 'ducking', 'dead'] as const) {
    if (typeof state[key] !== 'boolean') throw new Error(`states[${frame}].${key} 无效`)
  }
}

function vector(value: unknown): value is [number, number] {
  return Array.isArray(value) && value.length === 2 && value.every((item) => typeof item === 'number' && Number.isFinite(item))
}

function maxVectorError(left: [number, number], right: [number, number]): number {
  return Math.max(Math.abs(left[0] - right[0]), Math.abs(left[1] - right[1]))
}

function normalizeFacing(value: PortableState['facing']): boolean {
  return value === true || value === 'Right'
}

function normalizeState(value: PortableState['state']): string | number {
  if (typeof value === 'number') return value
  const known = ['Normal', 'Climb', 'Dash', 'Swim', 'Boost', 'RedDash', 'HitSquash', 'Launch', 'Pickup', 'DreamDash', 'SummitLaunch', 'Dummy', 'IntroWalk', 'IntroJump', 'IntroRespawn', 'IntroWakeUp', 'BirdDashTutorial', 'Frozen', 'ReflectionFall', 'StarFly', 'TempleFall', 'CassetteFly', 'Attract', 'IntroMoonJump', 'FlingBird', 'IntroThinkForABit']
  const index = known.indexOf(value)
  return index >= 0 ? index : value
}
