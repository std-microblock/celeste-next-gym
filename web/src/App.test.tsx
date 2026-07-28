import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import App from './App'
import type { GymMap, SimInput, SimState } from './model'

const testMap = vi.hoisted((): GymMap => ({
  name: 'Test room',
  bounds: { x: 0, y: 0, width: 320, height: 180 },
  spawn: { x: 16, y: 16 },
  solids: [],
  entities: [],
  source_package: null,
}))

const wasm = vi.hoisted(() => ({
  simulate: vi.fn<(state: SimState, inputs: SimInput[], map: GymMap) => Promise<SimState[]>>(),
}))

vi.mock('./simulator/wasmClient', () => ({
  WasmClient: class {
    ready = async () => undefined
    loadMap = async () => structuredClone(testMap)
    simulate = wasm.simulate
    dispose = () => undefined
  },
}))

vi.mock('./components/GameView', () => ({
  GameView: ({ frame, states }: { frame: number; states: readonly (SimState | undefined)[] }) => (
    <canvas aria-label="游戏画布" data-frame={frame} data-state-count={states.length} />
  ),
}))
vi.mock('./components/InputTimeline', () => ({ InputTimeline: () => <div>时间线编辑器</div> }))
vi.mock('./components/KeyBindings', () => ({ KeyBindings: () => <div>键位编辑器</div> }))
vi.mock('./components/StateInspector', () => ({ StateInspector: () => <div>状态检查器</div> }))

describe('App modes', () => {
  let nextAnimationFrame: FrameRequestCallback | undefined

  beforeEach(() => {
    nextAnimationFrame = undefined
    vi.stubGlobal('requestAnimationFrame', vi.fn((callback: FrameRequestCallback) => {
      nextAnimationFrame = callback
      return 1
    }))
    vi.stubGlobal('cancelAnimationFrame', vi.fn())
    wasm.simulate.mockImplementation(async (state, inputs) => [
      state,
      ...inputs.map((_, index) => ({ ...state, pos: { ...state.pos, x: state.pos.x + index + 1 } })),
    ])
  })

  afterEach(() => {
    cleanup()
    vi.unstubAllGlobals()
    vi.clearAllMocks()
  })

  it('opens in the canvas-only play mode and reveals the editor from the advanced tab', async () => {
    render(<App />)

    expect(screen.getByRole('tab', { name: '游玩' })).toHaveAttribute('aria-selected', 'true')
    expect(screen.getByLabelText('游戏画布')).toHaveAttribute('data-state-count', '0')
    expect(screen.queryByText('CELESTE')).not.toBeInTheDocument()
    expect(screen.queryByText('录制输入')).not.toBeInTheDocument()
    expect(screen.queryByText('时间线编辑器')).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('tab', { name: '高级' }))

    expect(screen.getByRole('tab', { name: '高级' })).toHaveAttribute('aria-selected', 'true')
    expect(screen.getByText('录制输入')).toBeInTheDocument()
    expect(screen.getByText('时间线编辑器')).toBeInTheDocument()
  })

  it('plays live input without exposing or appending frame history', async () => {
    render(<App />)
    await waitFor(() => expect(nextAnimationFrame).toBeDefined())
    fireEvent.keyDown(window, { code: 'KeyD' })

    await act(async () => {
      nextAnimationFrame?.(performance.now() + 20)
      await Promise.resolve()
    })

    await waitFor(() => expect(wasm.simulate).toHaveBeenCalled())
    const [, inputs] = wasm.simulate.mock.calls.at(-1)!
    expect(inputs[0].move_x).toBe(1)
    expect(screen.getByLabelText('游戏画布')).toHaveAttribute('data-state-count', '0')
    expect(Number(screen.getByLabelText('游戏画布').getAttribute('data-frame'))).toBeGreaterThan(0)
    expect(screen.queryByText('时间线编辑器')).not.toBeInTheDocument()
  })
})
