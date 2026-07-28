import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { field, semanticAssert } from '../../verify.js'
import { TECH_OTHER_5_1_BINO_TECH } from '../lookout-parts.js'

export const mapParts = [TECH_OTHER_5_1_BINO_TECH] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ['feature:lookout'],
  techniqueIds: ['5.1'],
  mapParts,
  recording: { primaryFor: ['5.1'], startFrame: 0, endFrame: 150, posterFrame: 110 },
  name: 'other-5.1-bino-tech',
  initial: { pos: [512, 496], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 150 }, (_, frame) => input({
    talk_pressed: frame === 0,
    move_x: frame >= 55 && frame < 110 ? 1 : 0,
    jump_pressed: frame === 110,
    jump_held: frame === 110,
  })),
  verify(states) {
    const cameras = states.map((state) => field<readonly number[]>(state, 'levelCamera'))
    semanticAssert(field<boolean>(states[1], 'lookoutInteracting') === true, scenario.name,
      'Talk did not start the entity-owned Lookout interaction on frame one')
    semanticAssert(states[2]?.state === 11 || states[2]?.state === 'Dummy', scenario.name,
      'Lookout did not force Player.StDummy on frame two')
    semanticAssert(cameras.some((camera) => (camera?.[0] ?? 0) >= (cameras[0]?.[0] ?? 0) + 186), scenario.name,
      'Aim input did not move Level.Camera through the certified 186px range')
    semanticAssert((states[129]?.state === 11 || states[129]?.state === 'Dummy')
      && field<boolean>(states[129], 'lookoutInteracting') === true, scenario.name,
    'LookRoutine did not retain Dummy interaction through frame 129')
    semanticAssert(states[130]?.state === 0 && field<boolean>(states[130], 'lookoutInteracting') === false, scenario.name,
      'LookRoutine did not restore Normal and clear interaction on frame 130')
  },
})
