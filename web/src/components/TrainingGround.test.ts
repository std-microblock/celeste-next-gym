import { describe, expect, it } from 'vitest'
import { makeEmptyButtons } from '../model'
import { trainingEntryDirectionPassed } from './TrainingGround'

describe('training entry direction', () => {
  it('accepts a held down-right direction independently of the simulated last-aim timing', () => {
    expect(trainingEntryDirectionPassed({ ...makeEmptyButtons(), right: true, down: true, dash: true })).toBe(true)
    expect(trainingEntryDirectionPassed({ ...makeEmptyButtons(), right: true, dash: true })).toBe(false)
    expect(trainingEntryDirectionPassed({ ...makeEmptyButtons(), down: true, dash: true })).toBe(false)
  })
})
