import { input, inputFrames } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { field, semanticAssert } from '../../verify.js'
import { TECH_ENTITY_4_18_3_CORE_BLOCK_ENTITY_DISPLACEMENT } from '../reform-parts.js'

export const mapParts = [TECH_ENTITY_4_18_3_CORE_BLOCK_ENTITY_DISPLACEMENT] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:bounce-block'],
  techniqueIds: ['4.18.3'],
  mapParts,
  name: 'entity-4.18.3-core-block-entity-displacement',
  initial: { pos: [736, 440], speed: [0, 0] },
  inputs: inputFrames(220, (frame) => input({
    // First leave the broken block's source footprint, then enter the
    // reformed body from the adjacent ledge while its 0.35s StaticMover
    // alarm is still pending.  The exact source body is empty at reform.
    move_x: frame >= 40 && frame < 72 ? -1 : frame >= 142 && frame < 180 ? 1 : 0,
  })),
  verify(states) {
    const broken = states.findIndex((state) => field(state, 'reformBlockCollidable') === false
      && field(state, 'reformSpikeCollidable') === false)
    const body = states.findIndex((state, frame) => frame > broken
      && field(state, 'reformBlockCollidable') === true
      && field(state, 'reformSpikeCollidable') === false)
    const spike = states.find((state, frame) => frame > body && field(state, 'reformSpikeCollidable') === true)
    const displaced = spike && field<readonly number[]>(spike, 'reformSpikePosition')

    semanticAssert(field(states[0], 'reformBlockKind') === 'BounceBlock', scenario.name, 'collector did not select the Core/BounceBlock')
    semanticAssert(broken >= 0, scenario.name, 'attached spike was not disabled with the broken block')
    semanticAssert(body > broken, scenario.name, 'block body did not reform while its StaticMover remained disabled')
    semanticAssert(spike, scenario.name, 'attached spike did not re-enable after the 0.35 second alarm')
    semanticAssert(displaced?.length === 2 && (Math.abs(Number(displaced[0]) - 768) > 0.01
      || Math.abs(Number(displaced[1]) - 440) > 0.01), scenario.name, `spike re-enabled at its undisplaced source position ${JSON.stringify(displaced)}`)
  },
})
