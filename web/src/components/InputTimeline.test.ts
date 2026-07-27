import { describe, expect, it } from 'vitest'
import { makeEmptyButtons } from '../model'
import { cachedRuns, heldRuns } from './InputTimeline'

describe('heldRuns', () => {
  it('merges pressed frames into press-to-release intervals', () => {
    const inputs = Array.from({ length: 9 }, makeEmptyButtons)
    for (const frame of [1, 2, 3, 6, 7]) inputs[frame].right = true
    expect(heldRuns(inputs, 'right', 0, inputs.length)).toEqual([[1, 4], [6, 8]])
  })

  it('merges computed states into continuous cache intervals', () => {
    const states = [undefined, {} as never, {} as never, undefined, {} as never]
    expect(cachedRuns(states, 0, states.length)).toEqual([[1, 3], [4, 5]])
  })
})
