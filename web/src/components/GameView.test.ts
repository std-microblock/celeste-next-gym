import { describe, expect, it } from 'vitest'
import atlas from '../../public/assets/original/gameplay/gameplay-selected.json'
import { createInitialState, PLAYGROUND } from '../model'
import type { MapEntity } from '../model'
import { activeBoosterCenter, runtimeEntityBounds } from './GameView'

describe('booster rendering', () => {
  it('moves the active booster bubble with the player', () => {
    const booster = PLAYGROUND.entities.find((entity) => entity.kind === 'booster')!
    const state = createInitialState(PLAYGROUND)
    state.pos = { x: 780, y: 420 }
    state.state = 'Dash'
    state.booster_reuse_timer = .4
    state.last_booster_target = {
      x: booster.bounds.x + booster.bounds.width / 2,
      y: booster.bounds.y + booster.bounds.height / 2 + 2,
    }

    expect(activeBoosterCenter(booster, state)).toEqual({ x: 780, y: 412.5 })
  })
})

describe('runtime entity rendering', () => {
  it('ships every source texture used by the added renderers', () => {
    expect(Object.keys(atlas.entries)).toEqual(expect.arrayContaining([
      'objects/spring/00',
      'objects/fireball/fireball08',
      'characters/theoCrystal/idle00',
      'collectables/strawberry/normal00',
      'objects/zipmover/block',
      'objects/zipmover/cog',
      'objects/zipmover/innercog00',
      'objects/zipmover/light01',
      'objects/BumpBlockNew/fire00',
      'objects/BumpBlockNew/fire_center00',
    ]))
  })

  it('uses the simulator positions for moving vanilla entities', () => {
    const state = createInitialState(PLAYGROUND)
    const zipMover: MapEntity = {
      kind: 'zip_mover',
      bounds: { x: 32, y: 440, width: 64, height: 16 },
      direction: { x: 0, y: 0 },
      nodes: [{ x: 32, y: 320 }],
      name: 'zipMover',
    }
    state.zip_movers = [{
      phase: 2,
      wait_timer: 0,
      at: .5,
      position: { x: 32, y: 380 },
      remainder: { x: 0, y: 0 },
      lift_speed: { x: 0, y: -120 },
      start: { x: 32, y: 440 },
    }]

    expect(runtimeEntityBounds(zipMover, state, 0)).toEqual({ x: 32, y: 380, width: 64, height: 16 })
  })

  it('converts Theo actor position back to its body collider', () => {
    const state = createInitialState(PLAYGROUND)
    const theo: MapEntity = {
      kind: 'theo_crystal',
      bounds: { x: 846, y: 486, width: 8, height: 10 },
      direction: { x: 0, y: 0 },
      name: 'theoCrystal',
    }
    state.theo_crystals = [{
      position: { x: 500, y: 300 },
      speed: { x: 0, y: 0 },
      remainder: { x: 0, y: 0 },
      held: true,
      cannot_hold_timer: 0,
      gravity_timer: 0,
    }]

    expect(runtimeEntityBounds(theo, state, 0)).toEqual({ x: 496, y: 290, width: 8, height: 10 })
  })

  it('matches Rust round-ties-even movement for simulator solids', () => {
    const state = createInitialState(PLAYGROUND)
    state.moving_solid_time = .5
    const solid: MapEntity = {
      kind: 'moving_solid',
      bounds: { x: 10, y: 20, width: 32, height: 8 },
      direction: { x: 5, y: -5 },
      name: 'celesteGymMovingSolid',
    }

    expect(runtimeEntityBounds(solid, state, 0)).toEqual({ x: 12, y: 18, width: 32, height: 8 })
  })
})
