import { fireEvent, render } from '@testing-library/react'
import { createElement } from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { makeEmptyButtons, type SimState } from '../model'
import { InputTimeline, cachedRuns, heldRuns } from './InputTimeline'

beforeEach(() => {
  vi.stubGlobal('ResizeObserver', class {
    observe(): void {}
    disconnect(): void {}
  })
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(null)
  Object.defineProperty(HTMLCanvasElement.prototype, 'setPointerCapture', { configurable: true, value: vi.fn() })
  Object.defineProperty(HTMLCanvasElement.prototype, 'releasePointerCapture', { configurable: true, value: vi.fn() })
  Object.defineProperty(HTMLCanvasElement.prototype, 'hasPointerCapture', { configurable: true, value: vi.fn(() => false) })
})

afterEach(() => {
  vi.restoreAllMocks()
  vi.unstubAllGlobals()
})

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

  it('uses arrow keys for the playhead until an operation is selected', () => {
    const inputs = Array.from({ length: 12 }, makeEmptyButtons)
    for (const frame of [2, 3, 4]) inputs[frame].right = true
    const onSeek = vi.fn()
    const onMove = vi.fn((_action, _targetAction, start: number, end: number, delta: number) => ({ start: start + delta, end: end + delta }))
    const view = render(createElement(InputTimeline, {
      frame: 1,
      inputs,
      states: Array.from<SimState | undefined>({ length: 13 }),
      onSeek,
      onPaint: vi.fn(),
      onMove,
      onEditComplete: vi.fn(),
      onResize: vi.fn(),
    }))
    const canvas = view.getByLabelText(/Canvas 时间线/) as HTMLCanvasElement
    vi.spyOn(canvas, 'getBoundingClientRect').mockReturnValue({ x: 0, y: 0, left: 0, top: 0, right: 900, bottom: 354, width: 900, height: 354, toJSON: () => ({}) })

    fireEvent.keyDown(canvas, { code: 'ArrowRight' })
    expect(onSeek).toHaveBeenCalledWith(2)

    fireEvent.pointerDown(canvas, { pointerId: 1, clientX: 108 + 2 * 26 + 4, clientY: 66 + 3 * 34 + 10, button: 0 })
    fireEvent.keyDown(canvas, { code: 'ArrowRight' })
    expect(onMove).toHaveBeenCalledWith('right', 'right', 2, 5, 1)
  })
})
