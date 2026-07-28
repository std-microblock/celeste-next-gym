import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { semanticAssert } from '../../verify.js'
import { ENTITY_4_25_THROWABLE_BACKBOOST_PART } from '../entity-tail-parts.js'

export const mapParts = [ENTITY_4_25_THROWABLE_BACKBOOST_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET, status: 'active', tags: [], techniqueIds: ['4.25'], mapParts,
  name: 'entity-4.25-throwable-backboost',
  recording: { primaryFor: ['4.25'], startFrame: 0, endFrame: 42, posterFrame: 24 },
  initial: { pos: [100, 496], on_ground: true },
  inputs: Array.from({ length: 42 }, (_, frame) => input({
    move_x: frame < 23 ? 1 : -1,
    grab_held: frame <= 22,
  })),
  verify(states) {
    const pickup = states.findIndex((state) => state.state === 8 && state.holding_theo)
    const released = states.find((state, frame) => frame > pickup && !state.holding_theo && !state.facing)
    semanticAssert(pickup >= 0 && Boolean(released), scenario.name, `pickup=${pickup}, released=${JSON.stringify(released)}`)
    semanticAssert((released?.speed[0] ?? 0) > 120, scenario.name, `backboost speed=${released?.speed[0]}`)
  },
})
