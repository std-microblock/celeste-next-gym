import { describe, expect, it } from 'vitest'
import type { GymMap, SimState } from '../model'
import { promptTargetPercent } from './TrainingPrompt'

describe('training prompt target', () => {
  it('maps the player position into the game viewport', () => {
    const map = { bounds: { x: 10, y: 20, width: 200, height: 100 } } as GymMap
    const state = { pos: { x: 110, y: 70 } } as SimState
    expect(promptTargetPercent(map, state)).toEqual({ x: 50, y: 50 })
  })
})
