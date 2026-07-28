import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { semanticAssert } from '../../verify.js'
import { ENTITY_4_19_SEEKER_BOUNCE_PART } from '../seeker-parts.js'

export const mapParts = [ENTITY_4_19_SEEKER_BOUNCE_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: [],
  techniqueIds: ['4.19'],
  mapParts,
  name: 'entity-4.19-seeker-bounce',
  initial: { pos: [160, 496], on_ground: true, dashes: 0, stamina: 20 },
  inputs: Array.from({ length: 180 }, (_, frame) => input({
    move_x: frame < 45 ? 1 : frame < 105 ? -1 : 1,
    jump_pressed: frame === 8 || frame === 78,
    jump_held: (frame >= 8 && frame < 20) || (frame >= 78 && frame < 90),
    dash_pressed: frame === 52,
  })),
  verify(states) {
    const pointBounce = states.findIndex((state, frame) => frame > 30
      && !state.dead
      && state.dashes === 1
      && Math.abs(state.speed[0]) >= 100
      && state.speed[1] < 0)
    semanticAssert(pointBounce >= 0, scenario.name,
      `missing Stunned OnAttackPlayer -> PointBounce signature in ${states.length} frames`)
  },
})
