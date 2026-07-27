import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_ZIP_MOVER } from '../common-parts.js'
import { field, near, pickCore } from '../../verify.js'

export const mapParts = [PLAYGROUND_ZIP_MOVER] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ["feature:zip-mover"],
  techniqueIds: ['4.8'],
  recording: { primaryFor: ['4.8'], startFrame: 0, endFrame: 25 },
  mapParts,
  name: 'entity-4.8-delayed-blockboost',
    initial: { pos: [92, 440], speed: [0, 0] },
    inputs: Array.from({ length: 25 }, (_, frame) => input({
      move_x: frame >= 8 ? 1 : 0,
      jump_pressed: frame === 24,
      jump_held: frame === 24,
    })),
    verify(states) {
      const before = states[24]
      const jumped = states[25]
      if (!before || before.on_ground || Math.abs(before.pos[0] - 108) > 0.01
        || jumped?.state !== 0 || Math.abs(jumped.speed[0] + 130) > 0.01
        || Math.abs(jumped.speed[1] + 230.828) > 0.01) {
        throw new Error(`entity-4.8-delayed-blockboost: did not apply retained ZipMover lift to the later static-wall jump: ${JSON.stringify({
          before: before && pickCore(before),
          jumped: jumped && pickCore(jumped),
          timeline: states.slice(14, 26).map((state) => ({
            ...pickCore(state),
            jump_grace: state._everest_fields?.jumpGraceTimer,
            current_lift: state._everest_fields?.currentLiftSpeed,
            last_lift: state._everest_fields?.lastLiftSpeed,
            lift_timer: state._everest_fields?.liftSpeedTimer,
          })),
        })}`)
      }
    },
})
