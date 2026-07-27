import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { TECH_ENTITY_4_4_EXPLOSION_BOOST } from '../common-parts.js'

export const mapParts = [TECH_ENTITY_4_4_EXPLOSION_BOOST] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ['feature:bumper'],
  techniqueIds: ['4.4'],
  mapParts,
  name: 'entity-4.4-explosion-boost',
  recording: { primaryFor: ['4.4'], startFrame: 0, endFrame: 30 },
    initial: { pos: [589, 206], speed: [0, 0] },
    inputs: Array.from({ length: 30 }, () => input({ move_x: -1 })),
    verify(states) {
      const launch = states[1]
      if (states.some((state) => state.dead) || launch?.state !== 7
        || Math.abs(launch.speed[0] + 336) > 0.01 || Math.abs(launch.speed[1] + 150) > 0.01) {
        throw new Error(`entity-4.4-explosion-boost: same-direction launch was ${JSON.stringify(launch)}`)
      }
    },
})
