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
  initial: { pos: [720, 440], speed: [0, 0] },
  inputs: inputFrames(220, (frame) => input({
    // Offset the launch leftward so its descending arc lands on the isolated
    // ledge. The source body is then clear at reform and can receive a
    // grounded rightward re-trigger during the StaticMover alarm.
    move_x: frame >= 130 && frame < 170 ? 1 : 0,
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
