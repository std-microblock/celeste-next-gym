import { describe, expect, it } from 'vitest'
import { makeEmptyButtons } from '../model'
import { outcomeSpeedX, timingAssessment, trainingEntryDirectionPassed, trainingInputLocked } from './TrainingGround'
import type { SimState } from '../model'

describe('training entry direction', () => {
  it('accepts a held down-right direction independently of the simulated last-aim timing', () => {
    expect(trainingEntryDirectionPassed({ ...makeEmptyButtons(), right: true, down: true, dash: true })).toBe(true)
    expect(trainingEntryDirectionPassed({ ...makeEmptyButtons(), right: true, dash: true })).toBe(false)
    expect(trainingEntryDirectionPassed({ ...makeEmptyButtons(), down: true, dash: true })).toBe(false)
  })
})

describe('training result timing', () => {
  it('describes whether the action was early, exact, or late', () => {
    expect(timingAssessment(9, 11)).toBe('早了 2 帧')
    expect(timingAssessment(11, 11)).toBe('正中最佳点')
    expect(timingAssessment(14, 11)).toBe('晚了 3 帧')
  })

  it('samples the same post-input X speed used by the outcome state', () => {
    expect(outcomeSpeedX({ speed: { x: 325, y: -80 } } as SimState)).toBe(325)
  })

  it('locks game input whenever a result overlay is active', () => {
    expect(trainingInputLocked(null)).toBe(false)
    expect(trainingInputLocked({ phase: 'success', startedAt: 0, durationMs: 3_000, resultSpeedX: 325, timelineFrame: 12 })).toBe(true)
  })
})
