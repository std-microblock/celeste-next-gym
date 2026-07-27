import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_BERRY } from '../common-parts.js'
import type { E2EState } from '../../types.js'
import { field, near, pickCore, semanticAssert } from '../../verify.js'

export const mapParts = [PLAYGROUND_BERRY] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: [],
  mapParts,
  name: 'mechanics-berry-train',
    initial: { pos: [160, 468], speed: [0, 0] },
    inputs: Array.from({ length: 64 }, () => input()),
    verify: verifyBerryTrain,
})


function verifyBerryTrain(states: readonly E2EState[]): void {
  const first = states.findIndex((state) => Number(field(state, 'StrawberryCollectIndex')) >= 1)
  const second = states.findIndex((state) => Number(field(state, 'StrawberryCollectIndex')) >= 2)
  semanticAssert(first >= 27, 'mechanics-berry-train', `first berry collected too early at frame ${first}; follower delay plus nine safe-ground frames were not observed`)
  semanticAssert(second > first, 'mechanics-berry-train', 'second berry never collected')
  semanticAssert(second - first === 17, 'mechanics-berry-train', `later berry queue offset was ${second - first} frames instead of 17`)
}
