import { describe, expect, it } from 'vitest'
import { makeEmptyButtons } from '../model'
import { timingAssessment, trainingEntryDirectionPassed } from './TrainingGround'

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
})
