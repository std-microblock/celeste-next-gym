import { defineMapPart } from '../map-parts.js'
import type { FixtureEntity, Rect } from '../types.js'

const PACKAGE = 'CelesteGymPlayground'
const SID = 'CelesteGymPlayground/Playground'
const ROOM = 'playground'
const room = (solids: readonly Rect[] = [], entities: readonly FixtureEntity[] = []) => ({
  name: ROOM,
  bounds: [0, 0, 320, 184] as const,
  spawn: [24, 160] as const,
  solids,
  entities,
})

export const SPIKE_CLIMB_PART = defineMapPart({
  id: 'tech.3.8.spike-climb',
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [room([[64, 40, 8, 144]], [{
    id: 'tech-3-8-spikes', kind: 'spikes', bounds: [61, 40, 3, 120], direction: [-1, 0], name: 'spikesLeft',
  }])],
})

export const NARROW_SPIKED_CLIMB_PART = defineMapPart({
  id: 'tech.3.8.1.narrow-spiked-climb',
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [room([[40, 24, 8, 160], [64, 24, 8, 160]], [
    { id: 'tech-3-8-1-left-spikes', kind: 'spikes', bounds: [48, 146, 3, 22], direction: [1, 0], name: 'spikesRight' },
    { id: 'tech-3-8-1-right-spikes', kind: 'spikes', bounds: [61, 140, 3, 28], direction: [-1, 0], name: 'spikesLeft' },
  ])],
})

export const SPIKE_CLIP_PART = defineMapPart({
  id: 'tech.3.9.spike-clip',
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [room([], [{
    id: 'tech-3-9-spikes', kind: 'spikes', bounds: [80, 100, 24, 3], direction: [0, -1], name: 'spikesUp',
  }])],
})

export const SPIKE_JUMP_PART = defineMapPart({
  id: 'tech.3.10.spike-jump',
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [room([], [
    { id: 'tech-3-10-zip', kind: 'zip_mover', bounds: [32, 120, 32, 16], nodes: [[64, 120]], name: 'zipMover' },
    { id: 'tech-3-10-spikes', kind: 'spikes', bounds: [65, 117, 16, 3], direction: [0, -1], name: 'spikesUp' },
  ])],
})

export const COBWOB_PART = defineMapPart({
  id: 'tech.3.12.1.cornerboost-wallboost',
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [room([[40, 40, 8, 64]])],
})

export const CORNERSLIP_PART = defineMapPart({
  id: 'tech.3.13.cornerslip',
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [room([], [{
    id: 'tech-3-13-dream-block', kind: 'dream_block', bounds: [40, 40, 32, 32], name: 'dreamBlock',
  }])],
})
