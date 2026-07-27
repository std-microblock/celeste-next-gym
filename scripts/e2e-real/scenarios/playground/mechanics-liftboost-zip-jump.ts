import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_ZIP_MOVER } from '../common-parts.js'
import type { E2EState } from '../../types.js'
import { field, near, pickCore, semanticAssert } from '../../verify.js'

export const mapParts = [PLAYGROUND_ZIP_MOVER] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'mechanics-liftboost-zip-jump',
    initial: { pos: [64, 440], speed: [0, 0] },
    inputs: Array.from({ length: 24 }, (_, frame) => input({
      jump_pressed: frame === 10,
      jump_held: frame >= 10 && frame < 16,
    })),
    verify: verifyZipMoverLiftboost,
})


function verifyZipMoverLiftboost(states: readonly E2EState[]): void {
  const jumped = states.find((state) => state.speed[1] < -105.01)
  semanticAssert(jumped, 'mechanics-liftboost-zip-jump', 'jump never inherited the upward ZipMover lift speed')
  const retained = field(jumped, 'lastLiftSpeed')
  semanticAssert(Array.isArray(retained) && retained[1] < 0, 'mechanics-liftboost-zip-jump', `jump frame did not retain an upward lastLiftSpeed: ${JSON.stringify(retained)}`)
  semanticAssert(jumped.on_ground === false && jumped.dead === false, 'mechanics-liftboost-zip-jump', 'liftboost jump did not leave the moving platform alive')
}

