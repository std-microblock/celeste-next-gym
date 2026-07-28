import { input, inputFrames } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { field, semanticAssert } from '../../verify.js'
import { TECH_ENTITY_4_18_1_REFORM_KICK } from '../reform-parts.js'

export const mapParts = [TECH_ENTITY_4_18_1_REFORM_KICK] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:move-block'],
  techniqueIds: ['4.18.1'],
  mapParts,
  name: 'entity-4.18.1-reform-kick',
  initial: { pos: [80, 400], speed: [0, 0] },
  inputs: inputFrames(360, (frame) => input({
    move_x: frame >= 20 && frame < 44 ? -1 : frame === 310 ? -1 : 0,
    jump_pressed: frame === 310,
    jump_held: frame === 310,
  })),
  verify(states) {
    const body = states.findIndex((state) => field(state, 'reformBlockCollidable') === true
      && field(state, 'reformBlockVisible') === false)
    const kicked = states.find((state, frame) => frame > body
      && field(state, 'reformBlockCollidable') === true
      && field(state, 'reformBlockVisible') === false
      && state.state === 0
      && state.speed[0] < -129.99
      && state.speed[1] <= -104.99)

    semanticAssert(body >= 0, scenario.name, 'newly collidable invisible MoveBlock body was not observed')
    semanticAssert(kicked, scenario.name, 'player did not wall-jump left from the invisible reformed body')
    semanticAssert(!kicked.dead, scenario.name, 'reform kick killed the player')
  },
})
