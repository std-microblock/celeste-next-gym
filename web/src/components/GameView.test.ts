import { describe, expect, it } from 'vitest'
import atlas from '../../public/assets/original/gameplay/gameplay-selected.json'
import { createInitialState, PLAYGROUND } from '../model'
import type { MapEntity } from '../model'
import { activeBoosterCenter, runtimeEntityBounds, strawberryIsPicked } from './GameView'

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
      'objects/puffer/idle00',
      'objects/puffer/explode00',
      'characters/oshiro/boss34',
      'characters/monsters/predator00',
      'danger/snowball00',
      'objects/clouds/cloud00',
      'objects/glider/idle0',
      'objects/glider/held0',
      'objects/moveBlock/base',
      'objects/moveBlock/arrow00',
      'objects/moveBlock/x',
      'collectables/heartGem/orb',
      'objects/cassetteblock/solid',
      'objects/cassetteblock/pressed00',
      'danger/crystal/bg_blue00',
      'danger/crystal/fg_blue00',
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

  it('uses Cloud and MoveBlock runtime positions from WASM snapshots', () => {
    const state = createInitialState(PLAYGROUND)
    const cloud: MapEntity = {
      kind: 'cloud',
      bounds: { x: 504, y: 440, width: 32, height: 5 },
      direction: { x: 0, y: 0 },
      name: 'cloud',
    }
    const moveBlock: MapEntity = {
      kind: 'move_block',
      bounds: { x: 600, y: 320, width: 32, height: 16 },
      direction: { x: 1, y: 0 },
      name: 'moveBlock',
    }
    state.clouds = [{
      phase: 1,
      speed: -120,
      position: { x: 504, y: 414 },
      remainder_y: .25,
      start: { x: 504, y: 440 },
    }]
    state.move_blocks = [{
      phase: 2,
      wait_timer: 0,
      speed: 60,
      angle: 0,
      crash_timer: .15,
      crash_reset_timer: .1,
      no_steer_timer: 0,
      position: { x: 628, y: 312 },
      remainder: { x: 0, y: 0 },
      lift_speed: { x: 60, y: 0 },
      start: { x: 600, y: 320 },
      visible: true,
      static_movers_enabled: true,
    }]

    expect(runtimeEntityBounds(cloud, state, 0)).toEqual({ x: 504, y: 414, width: 32, height: 5 })
    expect(runtimeEntityBounds(moveBlock, state, 0)).toEqual({ x: 628, y: 312, width: 32, height: 16 })
  })

  it('hides collected strawberries by their map entity bit', () => {
    const state = createInitialState(PLAYGROUND)
    state.strawberry_picked_mask = (1n << 3n) | (1n << 63n)

    expect(strawberryIsPicked(state, 2)).toBe(false)
    expect(strawberryIsPicked(state, 3)).toBe(true)
    expect(strawberryIsPicked(state, 63)).toBe(true)
  })
})
