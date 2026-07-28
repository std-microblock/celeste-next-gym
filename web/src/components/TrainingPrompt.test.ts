import { describe, expect, it } from 'vitest'
import type { GymMap, SimState } from '../model'
import { promptTargetPercent } from './TrainingPrompt'

describe('training prompt target', () => {
  it('maps the player position into the game viewport', () => {
    const map = { bounds: { x: 10, y: 20, width: 200, height: 100 } } as GymMap
    const state = { pos: { x: 110, y: 70 } } as SimState
    expect(promptTargetPercent(map, state, { width: 400, height: 400 })).toEqual({ x: 50, y: 42.5 })
  })

  it('accounts for letterboxing around the fixed game aspect ratio', () => {
    const map = { bounds: { x: 0, y: 0, width: 200, height: 100 } } as GymMap
    expect(promptTargetPercent(map, { pos: { x: 0, y: 0 } } as SimState, { width: 400, height: 400 })).toEqual({ x: 0, y: 17.5 })
    expect(promptTargetPercent(map, { pos: { x: 200, y: 100 } } as SimState, { width: 400, height: 400 })).toEqual({ x: 100, y: 67.5 })
  })
})
