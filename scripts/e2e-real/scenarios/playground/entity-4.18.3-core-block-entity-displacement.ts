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
  initial: { pos: [720, 480], speed: [0, 0] },
  inputs: inputFrames(220, (frame) => input({
    // Clear to the one-pixel-separated right platform first.  Begin the
    // leftward return early enough to enter the native 12-frame WindingUp
    // just after reform, while still remaining outside the source body at
    // its real BlockedCheck frame.
    move_x: frame >= 36 && frame < 97 ? 1 : frame >= 97 && frame < 170 ? -1 : 0,
    jump_pressed: frame === 80,
    jump_held: frame >= 80 && frame < 90,
  })),
  verify(states) {
    const broken = states.findIndex((state) => field(state, 'reformBlockCollidable') === false
      && field(state, 'reformSpikeCollidable') === false)
    const body = states.findIndex((state, frame) => frame > broken
      && field(state, 'reformBlockCollidable') === true
      && field(state, 'reformSpikeCollidable') === false)
    const spike = states.findIndex((state, frame) => frame > body && field(state, 'reformSpikeCollidable') === true)
    const secondBounce = states.findIndex((state, frame) => {
      const position = field<readonly number[]>(state, 'reformBlockPosition')
      return frame > body && frame < spike
        && field(state, 'reformBlockCollidable') === true
        && field(state, 'reformSpikeCollidable') === false
        && position?.length === 2
        && (Math.abs(Number(position[0]) - 712) > 0.01 || Math.abs(Number(position[1]) - 480) > 0.01)
    })
    const restored = field<readonly number[]>(states[body], 'reformBlockPosition')
    const displaced = spike >= 0 && field<readonly number[]>(states[spike], 'reformSpikePosition')
    const bodyPlayer = states[body]?.pos
    const playerOverlapsSource = bodyPlayer !== undefined
      && bodyPlayer[0] + 4 > 712 && bodyPlayer[0] - 4 < 776
      && bodyPlayer[1] > 480 && bodyPlayer[1] - 11 < 496

    semanticAssert(field(states[0], 'reformBlockKind') === 'BounceBlock', scenario.name, 'collector did not select the Core/BounceBlock')
    semanticAssert(broken >= 0, scenario.name, 'attached spike was not disabled with the broken block')
    semanticAssert(body > broken, scenario.name, 'block body did not reform while its StaticMover remained disabled')
    semanticAssert(restored?.length === 2 && Math.abs(Number(restored[0]) - 712) <= 0.01
      && Math.abs(Number(restored[1]) - 480) <= 0.01, scenario.name, `block reformed at the wrong reset target ${JSON.stringify(restored)}`)
    semanticAssert(!playerOverlapsSource, scenario.name, `player still overlapped the source body at reform: ${JSON.stringify(bodyPlayer)}`)
    semanticAssert(spike > body, scenario.name, 'attached spike did not re-enable after the 0.35 second alarm')
    semanticAssert(spike - body >= 20 && spike - body <= 24, scenario.name, `StaticMover alarm gap was ${spike - body} frames instead of about 0.35 seconds`)
    semanticAssert(secondBounce > body && secondBounce < spike, scenario.name, 'block did not begin its native second bounce while the StaticMover alarm was active')
    semanticAssert(displaced?.length === 2 && (Math.abs(Number(displaced[0]) - 776) > 0.01
      || Math.abs(Number(displaced[1]) - 440) > 0.01), scenario.name, `spike re-enabled at its undisplaced source position ${JSON.stringify(displaced)}`)
  },
})
