import { describe, expect, it } from 'vitest'
import { snapshot } from './helpers.ts'
import { allModulesCompleted, average, moduleAtPlayer, timingAccuracy, triggerContainsPlayer } from './course.ts'
import type { TrainingMapDocument } from './catalog.ts'

const trigger = { id: 'lesson', bounds: { x: 40, y: 200, width: 40, height: 40 } }

describe('map-driven training helpers', () => {
  it('detects an invisible trigger against the player collider', () => {
    expect(triggerContainsPlayer(trigger, snapshot({ x: 40, y: 220 }))).toBe(true)
    expect(triggerContainsPlayer(trigger, snapshot({ x: 84.01, y: 220 }))).toBe(false)
    expect(triggerContainsPlayer(trigger, snapshot({ x: 60, y: 199 }))).toBe(false)
  })

  it('scores timing and aggregates course statistics', () => {
    expect(timingAccuracy(12, 12)).toBe(100)
    expect(timingAccuracy(15, 12)).toBe(76)
    expect(timingAccuracy(99, 12)).toBe(0)
    expect(average([100, 76])).toBe(88)
  })

  it('selects only unfinished modules and unlocks the finish after all modules', () => {
    const training = {
      modules: [
        { id: 'first', trigger },
        { id: 'second', trigger: { id: 'second-trigger', bounds: { x: 100, y: 200, width: 40, height: 40 } } },
      ],
    } as unknown as TrainingMapDocument
    const player = snapshot({ x: 60, y: 220 })
    expect(moduleAtPlayer(training, player, new Set())?.id).toBe('first')
    expect(moduleAtPlayer(training, player, new Set(['first']))).toBeUndefined()
    expect(allModulesCompleted(training, new Set(['first']))).toBe(false)
    expect(allModulesCompleted(training, new Set(['first', 'second']))).toBe(true)
  })
})
