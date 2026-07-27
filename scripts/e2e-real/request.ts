import type { PlayerSnapshot, ScenarioDefinition, SimulateRequest } from './types.js'

export const DEFAULT_INITIAL_SNAPSHOT = Object.freeze<PlayerSnapshot>({
  pos: [19, 144],
  speed: [0, 0],
  state: 'Normal',
  facing: 'Right',
  dashes: 1,
  stamina: 110,
  on_ground: false,
  ducking: false,
})

export function createRequest(options: {
  scenario: ScenarioDefinition
  map: Uint8Array
  room?: string
  skipTransitions: boolean
}): SimulateRequest {
  const initialSnapshot: PlayerSnapshot = {
    ...DEFAULT_INITIAL_SNAPSHOT,
    can_dream_dash: options.scenario.target.areaId === 2,
    ...options.scenario.initial,
  }
  return {
    map: options.map,
    ...(options.room === undefined ? {} : { room: options.room }),
    dream_dash: options.scenario.target.areaId === 2,
    inputs: options.scenario.inputs,
    initial_snapshot: initialSnapshot,
    frames: options.scenario.inputs.length,
    skip_transitions: options.skipTransitions,
  }
}
