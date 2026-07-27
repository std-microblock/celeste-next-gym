import { MIN_REFLECTED_FIELDS } from './constants.js'
import type { E2EState, SimulateRequest } from './types.js'

export function validateCollectedStates(states: unknown, request: SimulateRequest): asserts states is E2EState[] {
  if (!Array.isArray(states)) throw new Error('collector states are not an array')
  if (states.length !== request.frames + 1) throw new Error(`invalid state count: ${states.length}`)
  for (const [frame, value] of states.entries()) validateState(value, frame)
  const first = states[0] as Partial<E2EState> | undefined
  if (!first?._everest_fields || Object.keys(first._everest_fields).length < MIN_REFLECTED_FIELDS) {
    throw new Error('real reflected Everest fields are missing')
  }
}

function validateState(value: unknown, frame: number): asserts value is E2EState {
  if (!value || typeof value !== 'object') throw new Error(`collector state ${frame} is not an object`)
  const state = value as Partial<E2EState>
  if (!isVector(state.pos) || !isVector(state.speed)) throw new Error(`collector state ${frame} has invalid position or speed`)
  if (typeof state.state !== 'string' && typeof state.state !== 'number') throw new Error(`collector state ${frame} has invalid state`)
  if (typeof state.facing !== 'boolean' && state.facing !== 'Left' && state.facing !== 'Right') throw new Error(`collector state ${frame} has invalid facing`)
  if (!Number.isFinite(state.dashes) || !Number.isFinite(state.stamina)) throw new Error(`collector state ${frame} has invalid resources`)
  if (typeof state.on_ground !== 'boolean' || typeof state.ducking !== 'boolean' || typeof state.dead !== 'boolean') {
    throw new Error(`collector state ${frame} has invalid boolean core fields`)
  }
  if (!Number.isSafeInteger(state._frame) || state._frame !== frame) throw new Error(`collector state ${frame} has invalid frame index`)
  if (!state._everest_fields || typeof state._everest_fields !== 'object' || Array.isArray(state._everest_fields)) {
    throw new Error(`collector state ${frame} has invalid reflected fields`)
  }
}

function isVector(value: unknown): value is readonly [number, number] {
  return Array.isArray(value) && value.length === 2 && value.every((item) => typeof item === 'number' && Number.isFinite(item))
}
