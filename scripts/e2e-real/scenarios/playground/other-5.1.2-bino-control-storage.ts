import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { field, semanticAssert } from '../../verify.js'
import { TECH_OTHER_5_1_2_BINO_CONTROL_STORAGE } from '../lookout-parts.js'

export const mapParts = [TECH_OTHER_5_1_2_BINO_CONTROL_STORAGE] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ['feature:lookout', 'feature:booster', 'storage:control-interrupt'],
  techniqueIds: ['5.1.2'],
  mapParts,
  name: 'other-5.1.2-bino-control-storage',
  initial: { pos: [496, 496], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 240 }, (_, frame) => input({
    talk_pressed: frame === 0,
    move_x: frame >= 100 && frame < 180 ? 1 : 0,
    jump_pressed: frame === 180,
    jump_held: frame === 180,
  })),
  verify(states) {
    const interruptedAt = states.findIndex((state) => field<boolean>(state, 'boosterBoostingPlayer') === true)
    semanticAssert(interruptedAt >= 0, scenario.name,
      'the native Booster callback never interrupted the Lookout Dummy state')
    semanticAssert(field<boolean>(states[interruptedAt], 'lookoutInteracting') === true, scenario.name,
      'Booster interruption cleared Lookout.interacting instead of storing camera control')
    const storedControl = states.some((state, frame) => {
      if (frame === 0 || state.state !== 0 || field<boolean>(state, 'lookoutInteracting') !== true) return false
      const previous = states[frame - 1]
      const camera = field<readonly number[]>(state, 'levelCamera')
      const previousCamera = field<readonly number[]>(previous, 'levelCamera')
      return Math.abs(state.pos[0] - previous.pos[0]) > 0.01
        && Math.hypot((camera?.[0] ?? 0) - (previousCamera?.[0] ?? 0), (camera?.[1] ?? 0) - (previousCamera?.[1] ?? 0)) > 0.01
    })
    semanticAssert(storedControl, scenario.name,
      'Normal movement and the entity-owned Lookout camera never advanced on the same frame')
    semanticAssert(field<boolean>(states.at(-1), 'lookoutInteracting') === false, scenario.name,
      'jump exit did not clear the stored Lookout interaction')
  },
})
