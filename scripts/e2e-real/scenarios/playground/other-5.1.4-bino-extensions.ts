import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { field, semanticAssert } from '../../verify.js'
import { TECH_OTHER_5_1_4_BINO_EXTENSIONS } from '../lookout-parts.js'

export const mapParts = [TECH_OTHER_5_1_4_BINO_EXTENSIONS] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:lookout', 'feature:camera-node', 'feature:exit-wipe'],
  techniqueIds: ['5.1.4'],
  mapParts,
  name: 'other-5.1.4-bino-extensions',
  initial: { pos: [512, 496], speed: [0, 0], on_ground: true },
  // Summit LookRoutine needs the node travel, HUD close, and its one-second
  // top-end FadeWipe.  560 frames stops during that source coroutine; 720
  // leaves a full post-wipe observation window.
  inputs: Array.from({ length: 720 }, (_, frame) => input({ talk_pressed: frame === 0, move_y: frame >= 55 ? -1 : 0 })),
  verify(states) {
    const cameras = states.map((state) => field<readonly number[]>(state, 'levelCamera'))
    semanticAssert(states.some((state) => (field<number>(state, 'lookoutNode') ?? 0) >= 2), scenario.name,
      'Lookout did not traverse the independent node chain')
    semanticAssert(cameras.some((camera) => Math.hypot((camera?.[0] ?? 0) - 352, (camera?.[1] ?? 0) - 364) > 600), scenario.name,
      'node extension never exceeded the 600px FadeWipe threshold')
    semanticAssert(field<boolean>(states.at(-1), 'lookoutInteracting') === false, scenario.name,
      'summit node endpoint did not complete the Lookout exit')
  },
})
