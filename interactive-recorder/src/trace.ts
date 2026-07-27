export const TRACE_FORMAT = 'celeste-next-gym-trace' as const
export const TRACE_VERSION = 1 as const
export const DEFAULT_TOLERANCE = 0.01

export interface TraceInput {
  readonly move_x: -1 | 0 | 1
  readonly move_y: -1 | 0 | 1
  readonly jump_pressed: boolean
  readonly jump_held: boolean
  readonly dash_pressed: boolean
  readonly crouch_dash_pressed: boolean
  readonly grab_held: boolean
}

export interface TraceState {
  readonly frame: number
  readonly pos: readonly [number, number]
  readonly speed: readonly [number, number]
  readonly state: string | number
  readonly facing: boolean | 'Left' | 'Right' | -1 | 1
  readonly dashes: number
  readonly stamina: number
  readonly on_ground: boolean
  readonly ducking: boolean
  readonly dead: boolean
  readonly snapshot?: Readonly<Record<string, unknown>>
  readonly fields?: Readonly<Record<string, unknown>>
}

export interface FrameTrace {
  readonly format: typeof TRACE_FORMAT
  readonly version: typeof TRACE_VERSION
  readonly source: 'web' | 'game'
  readonly recorded_at: string
  readonly map: {
    readonly sid: string
    readonly room: string
    readonly binary: string
    readonly sha256?: string
    readonly data?: Readonly<Record<string, unknown>>
  }
  readonly inputs: readonly TraceInput[]
  readonly states: readonly TraceState[]
}

export interface TraceComparison {
  readonly matched: boolean
  readonly compared_frames: number
  readonly max_position_error: number
  readonly max_speed_error: number
  readonly first_mismatch: number | null
  readonly differing_fields: readonly string[]
  readonly length: { readonly actual: number; readonly expected: number }
}

export function validateTrace(value: unknown): FrameTrace {
  const trace = record(value, 'trace')
  if (trace.format !== TRACE_FORMAT || trace.version !== TRACE_VERSION) {
    throw new Error(`trace must use ${TRACE_FORMAT} v${TRACE_VERSION}`)
  }
  if (trace.source !== 'web' && trace.source !== 'game') throw new Error('trace.source must be web or game')
  record(trace.map, 'trace.map')
  if (!Array.isArray(trace.inputs) || !Array.isArray(trace.states)) throw new Error('trace inputs/states must be arrays')
  if (trace.states.length !== trace.inputs.length + 1) throw new Error('trace states length must equal inputs length + 1')
  trace.inputs.forEach((input, frame) => validateInput(input, frame))
  trace.states.forEach((state, frame) => validateState(state, frame))
  return trace as unknown as FrameTrace
}

export function compareTraces(
  actual: FrameTrace,
  expected: FrameTrace,
  tolerance = DEFAULT_TOLERANCE,
): TraceComparison {
  if (!Number.isFinite(tolerance) || tolerance < 0 || tolerance > DEFAULT_TOLERANCE) {
    throw new Error(`tolerance must be between 0 and ${DEFAULT_TOLERANCE}`)
  }
  const compared = Math.min(actual.states.length, expected.states.length)
  let maxPosition = 0
  let maxSpeed = 0
  let firstMismatch: number | null = null
  let differingFields: string[] = []
  for (let frame = 0; frame < compared; frame += 1) {
    const left = actual.states[frame]!
    const right = expected.states[frame]!
    const positionError = vectorError(left.pos, right.pos)
    const speedError = vectorError(left.speed, right.speed)
    maxPosition = Math.max(maxPosition, positionError)
    maxSpeed = Math.max(maxSpeed, speedError)
    if (firstMismatch !== null) continue
    const fields: string[] = []
    if (positionError > tolerance) fields.push('position')
    if (speedError > tolerance) fields.push('speed')
    if (normalizeState(left.state) !== normalizeState(right.state)) fields.push('state')
    if (normalizeFacing(left.facing) !== normalizeFacing(right.facing)) fields.push('facing')
    if (left.dashes !== right.dashes) fields.push('dashes')
    if (Math.abs(left.stamina - right.stamina) > tolerance) fields.push('stamina')
    if (left.on_ground !== right.on_ground) fields.push('grounded')
    if (left.ducking !== right.ducking) fields.push('ducking')
    if (left.dead !== right.dead) fields.push('death')
    if (fields.length > 0) {
      firstMismatch = frame
      differingFields = fields
    }
  }
  if (firstMismatch === null && actual.states.length !== expected.states.length) {
    firstMismatch = compared
    differingFields = ['length']
  }
  return {
    matched: firstMismatch === null,
    compared_frames: compared,
    max_position_error: maxPosition,
    max_speed_error: maxSpeed,
    first_mismatch: firstMismatch,
    differing_fields: differingFields,
    length: { actual: actual.states.length, expected: expected.states.length },
  }
}

function validateInput(value: unknown, frame: number): void {
  const input = record(value, `inputs[${frame}]`)
  if (![-1, 0, 1].includes(input.move_x as number) || ![-1, 0, 1].includes(input.move_y as number)) {
    throw new Error(`inputs[${frame}] axes must be -1, 0, or 1`)
  }
  for (const key of ['jump_pressed', 'jump_held', 'dash_pressed', 'crouch_dash_pressed', 'grab_held']) {
    if (typeof input[key] !== 'boolean') throw new Error(`inputs[${frame}].${key} must be boolean`)
  }
}

function validateState(value: unknown, frame: number): void {
  const state = record(value, `states[${frame}]`)
  vector(state.pos, `states[${frame}].pos`)
  vector(state.speed, `states[${frame}].speed`)
  if (typeof state.state !== 'string' && !Number.isInteger(state.state)) throw new Error(`states[${frame}].state is invalid`)
  if (![true, false, 'Left', 'Right', -1, 1].includes(state.facing as never)) throw new Error(`states[${frame}].facing is invalid`)
  for (const key of ['dashes', 'stamina']) {
    if (typeof state[key] !== 'number' || !Number.isFinite(state[key])) throw new Error(`states[${frame}].${key} is invalid`)
  }
  for (const key of ['on_ground', 'ducking', 'dead']) {
    if (typeof state[key] !== 'boolean') throw new Error(`states[${frame}].${key} must be boolean`)
  }
}

function record(value: unknown, label: string): Record<string, unknown> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error(`${label} must be an object`)
  return value as Record<string, unknown>
}

function vector(value: unknown, label: string): asserts value is readonly [number, number] {
  if (!Array.isArray(value) || value.length !== 2 || value.some((item) => typeof item !== 'number' || !Number.isFinite(item))) {
    throw new Error(`${label} must be a finite two-element vector`)
  }
}

function vectorError(left: readonly [number, number], right: readonly [number, number]): number {
  return Math.max(Math.abs(left[0] - right[0]), Math.abs(left[1] - right[1]))
}

function normalizeFacing(value: TraceState['facing']): boolean {
  return value === true || value === 'Right' || value === 1
}

function normalizeState(value: TraceState['state']): string | number {
  if (typeof value === 'number') return value
  const known = ['Normal', 'Climb', 'Dash', 'Swim', 'Boost', 'RedDash', 'HitSquash', 'Launch', 'Pickup', 'DreamDash', 'SummitLaunch', 'Dummy', 'IntroWalk', 'IntroJump', 'IntroRespawn', 'IntroWakeUp', 'BirdDashTutorial', 'Frozen', 'ReflectionFall', 'StarFly', 'TempleFall', 'CassetteFly', 'Attract', 'IntroMoonJump', 'FlingBird', 'IntroThinkForABit']
  const index = known.indexOf(value)
  return index >= 0 ? index : value
}
