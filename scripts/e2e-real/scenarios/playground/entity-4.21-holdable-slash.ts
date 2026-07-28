import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { near, semanticAssert } from '../../verify.js'
import { ENTITY_4_21_HOLDABLE_SLASH_PART } from '../holdable-parts.js'

export const mapParts = [ENTITY_4_21_HOLDABLE_SLASH_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['4.21'],
  mapParts,
  name: 'entity-4.21-holdable-slash',
  recording: { primaryFor: ['4.21'], startFrame: 0, endFrame: 70, posterFrame: 38 },
  initial: { pos: [300, 496], on_ground: true },
  inputs: Array.from({ length: 70 }, (_, frame) => input({
    move_x: frame >= 14 && frame < 28 ? -1 : frame >= 28 ? 1 : 0,
    move_y: frame === 23 ? 1 : 0,
    jump_pressed: frame === 14,
    jump_held: frame >= 14 && frame < 23,
    dash_pressed: frame === 28,
    grab_held: frame <= 22 || frame >= 35,
  })),
  verify(states) {
    const firstPickup = states.findIndex((state) => state.state === 8 && state.holding_theo)
    const released = states.findIndex((state, frame) => frame > firstPickup && !state.holding_theo)
    const dash = states.findIndex((state, frame) => frame > released && state.state === 2)
    const regrab = states.findIndex((state, frame) => frame > dash && state.state === 8 && state.holding_theo)
    const straightDash = states.slice(dash, regrab).find((state) => near(state.speed[0], 240) && near(state.speed[1], 0))

    semanticAssert(firstPickup >= 0 && released > firstPickup, scenario.name,
      `pickup=${firstPickup}, release=${released}`)
    semanticAssert(!states[released]?.on_ground && !near(states[released]?.speed[1] ?? 0, 0), scenario.name,
      `release=${JSON.stringify(states[released])}`)
    semanticAssert(dash > released && straightDash !== undefined, scenario.name,
      `dash=${dash}, regrab=${regrab}`)
    semanticAssert(regrab > dash, scenario.name, `regrab=${regrab}, dash=${dash}`)
  },
})
