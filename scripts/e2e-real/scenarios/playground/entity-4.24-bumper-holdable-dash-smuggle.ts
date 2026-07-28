import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { semanticAssert } from '../../verify.js'
import { ENTITY_4_24_BUMPER_SMUGGLE_PART } from '../entity-tail-parts.js'

export const mapParts = [ENTITY_4_24_BUMPER_SMUGGLE_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET, status: 'candidate', tags: ['feature:bumper'], techniqueIds: ['4.24'], mapParts,
  name: 'entity-4.24-bumper-holdable-dash-smuggle',
  initial: { pos: [100, 496], on_ground: true },
  inputs: Array.from({ length: 120 }, (_, frame) => input({
    move_x: frame >= 13 && frame < 45 ? 1 : frame >= 45 ? 1 : 0,
    move_y: frame === 27 || frame >= 45 ? 1 : 0,
    dash_pressed: frame === 45,
    grab_held: frame <= 26 || frame >= 45,
  })),
  verify(states) {
    const firstPickup = states.findIndex((state) => state.state === 8 && state.holding_theo)
    const launch = states.findIndex((state, frame) => frame > firstPickup && state.state === 7 && !state.holding_theo)
    const dash = states.findIndex((state, frame) => frame > launch && state.state === 2 && !state.holding_theo)
    const regrab = states.findIndex((state, frame) => frame > dash && state.state === 8 && state.holding_theo)
    semanticAssert(firstPickup >= 0 && launch > firstPickup, scenario.name, `pickup=${firstPickup}, launch=${launch}`)
    semanticAssert(dash > launch, scenario.name, `dash=${dash}, launch=${launch}`)
    semanticAssert(regrab > dash, scenario.name, `regrab=${regrab}, dash=${dash}`)
  },
})
