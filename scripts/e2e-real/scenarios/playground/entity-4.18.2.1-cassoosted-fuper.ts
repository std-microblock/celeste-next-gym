import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { near, semanticAssert } from '../../verify.js'
import { cassetteBlock } from '../cassette-observation.js'
import { TECH_ENTITY_4_18_2_1_CASSOOSTED_FUPER } from '../cassette-spinner-parts.js'

export const mapParts = [TECH_ENTITY_4_18_2_1_CASSOOSTED_FUPER] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:cassette-block', 'feature:star-fly'],
  techniqueIds: ['4.18.2.1'],
  mapParts,
  name: 'entity-4.18.2.1-cassoosted-fuper',
  initial: { pos: [350, 496], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 100 }, (_, frame) => input({
    // Keep right aim live through Feather's coroutine. The tempo-three
    // CassetteBlock writes its upward lift in the entity phase; the grounded
    // Feather jump consumes that retained LiftBoost on its next update.
    move_x: 1,
    jump_pressed: frame === 28,
    jump_held: frame >= 28 && frame < 40,
  })),
  verify(states) {
    const fuper = states.findIndex((state) => state.state === 0
      && near(state.speed[0], 273.333_34, 0.01)
      && near(state.speed[1], -165, 0.01))
    const reform = states.findIndex((state) => {
      const block = cassetteBlock(state, 0)
      return block?.position[1] === 493 && block.collidable
    })
    semanticAssert(fuper > 0, scenario.name, `grounded feather exit was not observed: ${fuper}`)
    semanticAssert(reform > 0 && Math.abs(reform - fuper) <= 1, scenario.name,
      `feather exit did not coincide with observed cassette reform: fuper=${fuper}, reform=${reform}`)
    semanticAssert(!states.some((state) => state.dead), scenario.name, 'cassoosted fuper killed the player')
  },
})
