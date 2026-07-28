import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { field, semanticAssert } from '../../verify.js'
import { TECH_OTHER_5_1_3_BINO_INTERACTION_STORAGE } from '../lookout-parts.js'

export const mapParts = [TECH_OTHER_5_1_3_BINO_INTERACTION_STORAGE] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:lookout', 'feature:booster', 'feature:transition', 'storage:interaction-removal'],
  techniqueIds: ['5.1.3'],
  mapParts,
  name: 'other-5.1.3-bino-interaction-storage',
  initial: { pos: [916, 496], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 300 }, (_, frame) => input({
    talk_pressed: frame === 0,
    // Movement begins after the native Booster has interrupted the Lookout
    // Dummy state and LookRoutine owns the camera independently of Player.
    move_x: frame >= 120 ? 1 : 0,
  })),
  verify(states) {
    semanticAssert(states.some((state) => field<boolean>(state, 'lookoutInteracting') === true), scenario.name,
      'Talk did not reach Lookout.interacting before the native interruption')
    semanticAssert(states.some((state) => field<boolean>(state, 'boosterBoostingPlayer') === true || state.state === 4), scenario.name,
      'the native Booster never interrupted the Lookout Dummy state')
    const removedAt = states.findIndex((state) => field<boolean>(state, 'lookoutRemovalObserved') === true)
    semanticAssert(removedAt >= 0, scenario.name,
      'DummyWalkToExact did not trigger the native Lookout.Removed transition path')
    semanticAssert(field<boolean>(states[removedAt], 'lookoutRemovedWhileInteracting') === true, scenario.name,
      'Lookout.Removed did not observe its entity-owned interacting flag')
    semanticAssert(field<number>(states[removedAt], 'lookoutRemovalPlayerState') === 0 && states[removedAt]?.state === 0, scenario.name,
      'room removal did not restore Player.StNormal while retaining the Lookout interaction')
    semanticAssert(states.slice(removedAt).some((state) => state.pos[0] >= 960), scenario.name,
      'side transition never completed into the second room')
  },
})
