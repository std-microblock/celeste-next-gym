import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_BOUNCE_BLOCK } from '../common-parts.js'
import { field, near, pickCore } from '../../verify.js'

export const mapParts = [PLAYGROUND_BOUNCE_BLOCK] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ["feature:bounce-block"],
  techniqueIds: [],
  mapParts,
  name: 'entity-4.7-core-hyper',
    initial: { pos: [384, 360], speed: [0, 0] },
    inputs: Array.from({ length: 38 }, (_, frame) => input({
      move_x: frame >= 32 ? 1 : 0,
      crouch_dash_pressed: frame === 32,
      jump_pressed: frame === 36,
      jump_held: frame === 36,
    })),
    verify(states) {
      const launch = states.find((state) => {
        const lastLiftSpeed = field<readonly number[]>(state, 'lastLiftSpeed')
        return near(state.speed[1], -200)
          && near(field<number>(state, 'jumpGraceTimer'), 0.1, 0.001)
          && near(lastLiftSpeed?.[1], -200)
      })
      const hyper = states[37]
      if (!launch || hyper?.state !== 0 || hyper.ducking
        || Math.abs(hyper.speed[0] - 325) > 0.01
        || Math.abs(hyper.speed[1] + 117.5) > 0.01) {
        throw new Error(`entity-4.7-core-hyper: missing BounceBlock launch grace/lift or 325/-117.5 Core Hyper: ${JSON.stringify({
          launch: launch && pickCore(launch),
          result: hyper && pickCore(hyper),
        })}`)
      }
    },
})
