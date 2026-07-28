import { input, inputFrames } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { field, semanticAssert } from '../../verify.js'
import { TECH_ENTITY_4_18_3_CORE_BLOCK_ENTITY_DISPLACEMENT } from '../reform-parts.js'

export const mapParts = [TECH_ENTITY_4_18_3_CORE_BLOCK_ENTITY_DISPLACEMENT] as const

function isPair(value: unknown): value is readonly [number, number] {
  return Array.isArray(value) && value.length === 2
}

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ['feature:bounce-block'],
  techniqueIds: ['4.18.3'],
  mapParts,
  name: 'entity-4.18.3-core-block-entity-displacement',
  initial: { pos: [720, 480], speed: [0, 0] },
  inputs: inputFrames(220, (frame) => input({
    // Clear to the one-pixel-separated right platform first. The leftward
    // crossing begins only after the observed respawn frame, while the spike
    // is still disabled by BounceBlock's 0.35-second reform alarm.
    move_x: frame >= 36 && frame < 108 ? 1 : frame >= 135 && frame < 170 ? -1 : 0,
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
    const brokenBody = field<readonly number[]>(states[broken], 'reformBlockPosition')
    const brokenSpike = field<readonly number[]>(states[broken], 'reformSpikePosition')
    const restored = field<readonly number[]>(states[body], 'reformBlockPosition')
    const reenabledSpike = spike >= 0 && field<readonly number[]>(states[spike], 'reformSpikePosition')
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
    // MoveStaticMovers runs before the alarm is armed.  Verify the re-enabled
    // spike was transported by the actual broken-body -> restored-body delta,
    // rather than merely accepting the fixture's nominal spike offset.
    semanticAssert(isPair(brokenBody) && isPair(brokenSpike) && isPair(reenabledSpike),
      scenario.name, 'collector did not capture the broken and re-enabled StaticMover positions')
    const expectedSpike = isPair(brokenBody) && isPair(brokenSpike) && isPair(restored)
      ? [Number(brokenSpike[0]) + Number(restored[0]) - Number(brokenBody[0]), Number(brokenSpike[1]) + Number(restored[1]) - Number(brokenBody[1])] as const
      : undefined
    semanticAssert(expectedSpike !== undefined && isPair(reenabledSpike)
      && Math.abs(Number(reenabledSpike[0]) - expectedSpike[0]) <= 0.01
      && Math.abs(Number(reenabledSpike[1]) - expectedSpike[1]) <= 0.01,
    scenario.name, `spike re-enabled away from the restored-body displacement ${JSON.stringify({ brokenBody, brokenSpike, restored, reenabledSpike, expectedSpike })}`)
    semanticAssert(expectedSpike !== undefined && isPair(brokenSpike)
      && (Math.abs(expectedSpike[0] - Number(brokenSpike[0])) > 0.01 || Math.abs(expectedSpike[1] - Number(brokenSpike[1])) > 0.01),
    scenario.name, `body did not actually displace its disabled StaticMover ${JSON.stringify({ brokenBody, brokenSpike, restored, expectedSpike })}`)
  },
})
