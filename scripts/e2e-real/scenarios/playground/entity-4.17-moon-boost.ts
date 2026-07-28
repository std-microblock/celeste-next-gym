import { input, inputFrames } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { field, semanticAssert } from '../../verify.js'
import { TECH_ENTITY_4_17_MOON_BOOST } from '../reform-parts.js'

export const mapParts = [TECH_ENTITY_4_17_MOON_BOOST] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ['feature:move-block'],
  techniqueIds: ['4.17'],
  recording: { primaryFor: ['4.17'], startFrame: 0, endFrame: 90 },
  mapParts,
  name: 'entity-4.17-moon-boost',
  initial: { pos: [80, 320], speed: [0, 0] },
  inputs: inputFrames(90, (frame) => input({
    move_x: frame >= 38 ? 1 : 0,
    move_y: frame < 55 ? -1 : 0,
    jump_pressed: frame === 38,
    jump_held: frame === 38,
  })),
  verify(states) {
    const diagonal = states.find((state) => {
      const position = field<readonly number[]>(state, 'reformBlockPosition')
      return field(state, 'reformBlockKind') === 'MoveBlock'
        && position?.length === 2
        && Number(position[0]) > 64.01
        && Number(position[1]) < 319.99
        && field(state, 'reformBlockCollidable') === true
    })
    const boosted = states.find((state) => state.state === 0 && state.speed[1] < -105.01)
    semanticAssert(!states.some((state) => state.dead), scenario.name, 'player died during the steered MoveBlock jump')
    semanticAssert(diagonal, scenario.name, 'MoveBlock never exposed an up-right steered position')
    semanticAssert(boosted, scenario.name, 'jump never inherited a negative vertical MoveBlock lift')
  },
})
