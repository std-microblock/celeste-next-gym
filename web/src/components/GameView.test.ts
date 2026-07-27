import { describe, expect, it } from 'vitest'
import { createInitialState, PLAYGROUND } from '../model'
import { activeBoosterCenter } from './GameView'

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
