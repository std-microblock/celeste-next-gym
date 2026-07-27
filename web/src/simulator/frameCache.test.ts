import { describe, expect, it } from 'vitest'
import { createInitialState, PLAYGROUND, type SimInput, type SimState } from '../model'
import { FrameCache } from './frameCache'
import type { SimulationRunner } from './wasmClient'

class FakeRunner implements SimulationRunner {
  calls: { start: number; inputs: SimInput[] }[] = []

  async simulate(state: SimState, inputs: SimInput[]): Promise<SimState[]> {
    this.calls.push({ start: state.pos.x, inputs })
    return [state, ...inputs.map((_, index) => ({ ...state, pos: { ...state.pos, x: state.pos.x + index + 1 } }))]
  }
}

describe('FrameCache', () => {
  it('computes only the requested range and reuses cached checkpoints', async () => {
    const runner = new FakeRunner()
    const initial = createInitialState(PLAYGROUND)
    initial.pos.x = 0
    const cache = new FrameCache(runner, PLAYGROUND, initial, 20)

    await cache.ensureFrame(10)
    expect(runner.calls).toHaveLength(1)
    expect(runner.calls[0].inputs).toHaveLength(10)

    await cache.ensureFrame(7)
    expect(runner.calls).toHaveLength(1)

    cache.setFrame(4, 'jump', true)
    expect(cache.getState(4)).toBeDefined()
    expect(cache.getState(5)).toBeUndefined()

    await cache.ensureFrame(7)
    expect(runner.calls).toHaveLength(2)
    expect(runner.calls[1]).toMatchObject({ start: 4 })
    expect(runner.calls[1].inputs).toHaveLength(3)
    expect(runner.calls[1].inputs[0].jump_pressed).toBe(true)
  })

  it('doubles capacity on demand without invalidating cached states', async () => {
    const runner = new FakeRunner()
    const cache = new FrameCache(runner, PLAYGROUND, createInitialState(PLAYGROUND), 4)

    await cache.ensureFrame(4)
    expect(cache.frameCount).toBe(4)
    expect(cache.getState(4)).toBeDefined()

    expect(cache.ensureCapacity(5)).toBe(8)
    expect(cache.frameCount).toBe(8)
    expect(cache.getState(4)).toBeDefined()

    await cache.ensureFrame(17)
    expect(cache.frameCount).toBe(32)
    expect(cache.getState(17)).toBeDefined()
  })

  it('moves a held input run as one interval', () => {
    const cache = new FrameCache(new FakeRunner(), PLAYGROUND, createInitialState(PLAYGROUND), 12)
    cache.paint('right', 2, 5, true)

    expect(cache.moveRun('right', 2, 6, 3)).toEqual({ start: 5, end: 9 })
    expect(cache.getInputs().map((input, frame) => input.right ? frame : -1).filter((frame) => frame >= 0)).toEqual([5, 6, 7, 8])
  })
})
