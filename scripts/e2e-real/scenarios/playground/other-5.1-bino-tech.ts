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
    semanticAssert(states.some((state) => field<boolean>(state, 'lookoutInteracting') === true), scenario.name,
      'Talk did not start the entity-owned Lookout interaction')
    semanticAssert(states.some((state) => state.state === 11 || state.state === 'Dummy'), scenario.name,
      'Lookout did not force Player.StDummy before camera control')
    semanticAssert(cameras.some((camera, index) => index > 55 && (camera?.[0] ?? 0) > (cameras[55]?.[0] ?? 0) + 8), scenario.name,
      'Aim input did not move Level.Camera independently')
    semanticAssert(field<boolean>(states.at(-1), 'lookoutInteracting') === false, scenario.name,
      'jump exit did not clear Lookout.interacting')
  },
})
