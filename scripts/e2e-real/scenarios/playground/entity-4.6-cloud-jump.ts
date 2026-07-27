import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { TECH_ENTITY_4_6_CLOUD_JUMP } from '../common-parts.js'

export const mapParts = [TECH_ENTITY_4_6_CLOUD_JUMP] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ['feature:cloud'],
  techniqueIds: ['4.6'],
  mapParts,
  name: 'entity-4.6-cloud-jump',
  recording: { primaryFor: ['4.6'], startFrame: 0, endFrame: 70 },
  initial: { pos: [616, 440], speed: [0, 0] },
  inputs: Array.from({ length: 70 }, () => input()),
  verify(states) {
    const launched = states.find((state) => Math.abs(state.speed[1] + 200) <= 0.01)
    if (states.some((state) => state.dead) || !launched || launched.state !== 0
      || Math.min(...states.map((state) => state.pos[1])) >= 440) {
      throw new Error(`entity-4.6-cloud-jump: cloud launch was ${JSON.stringify(launched)}`)
    }
  },
})
