import { buttonsToInput, makeEmptyButtons, type Action, type FrameButtons, type GymMap, type SimState } from '../model'
import type { SimulationRunner } from './wasmClient'

type Listener = () => void

export class FrameCache {
  private buttons: FrameButtons[]
  private states: Array<SimState | undefined>
  private revision = 0
  private queue: Promise<void> = Promise.resolve()
  private readonly listeners = new Set<Listener>()

  constructor(
    private readonly runner: SimulationRunner,
    private map: GymMap,
    initial: SimState,
    frameCount = 360,
  ) {
    this.buttons = Array.from({ length: frameCount }, makeEmptyButtons)
    this.states = Array.from({ length: frameCount + 1 })
    this.states[0] = initial
  }

  subscribe(listener: Listener): () => void {
    this.listeners.add(listener)
    return () => this.listeners.delete(listener)
  }

  private emit(): void {
    for (const listener of this.listeners) listener()
  }

  get frameCount(): number {
    return this.buttons.length
  }

  getInputs(): readonly FrameButtons[] {
    return this.buttons
  }

  getStates(): readonly (SimState | undefined)[] {
    return this.states
  }

  getState(frame: number): SimState | undefined {
    return this.states[frame]
  }

  getNearestState(frame: number): { frame: number; state: SimState } {
    for (let cursor = Math.min(frame, this.states.length - 1); cursor >= 0; cursor -= 1) {
      const state = this.states[cursor]
      if (state) return { frame: cursor, state }
    }
    throw new Error('初始帧丢失')
  }

  get computedThrough(): number {
    let frame = 0
    while (frame + 1 < this.states.length && this.states[frame + 1]) frame += 1
    return frame
  }

  ensureCapacity(requiredFrame: number): number {
    const required = Math.max(0, Math.round(requiredFrame))
    if (required <= this.buttons.length) return this.buttons.length
    let size = Math.max(1, this.buttons.length)
    while (size < required) size *= 2
    const added = size - this.buttons.length
    this.buttons = [...this.buttons, ...Array.from({ length: added }, makeEmptyButtons)]
    this.states = [...this.states, ...Array.from<SimState | undefined>({ length: added }).fill(undefined)]
    this.emit()
    return size
  }

  setFrame(frame: number, action: Action, value: boolean): void {
    this.ensureCapacity(frame + 1)
    const current = this.buttons[frame]
    if (!current || current[action] === value) return
    const next = this.buttons.slice()
    next[frame] = { ...current, [action]: value }
    this.buttons = next
    this.invalidateAfterInput(frame)
  }

  setButtons(frame: number, buttons: FrameButtons): void {
    if (frame < 0) return
    this.ensureCapacity(frame + 1)
    const next = this.buttons.slice()
    next[frame] = { ...buttons }
    this.buttons = next
    this.invalidateAfterInput(frame)
  }

  paint(action: Action, from: number, to: number, value: boolean): void {
    const start = Math.max(0, Math.min(from, to))
    const end = Math.min(this.buttons.length - 1, Math.max(from, to))
    let changed = false
    const next = this.buttons.slice()
    for (let frame = start; frame <= end; frame += 1) {
      if (this.buttons[frame][action] !== value) {
        next[frame] = { ...this.buttons[frame], [action]: value }
        changed = true
      }
    }
    if (changed) {
      this.buttons = next
      this.invalidateAfterInput(start)
    }
  }

  moveRun(action: Action, start: number, end: number, delta: number): { start: number; end: number } {
    const runStart = Math.max(0, Math.min(this.buttons.length, Math.round(start)))
    const runEnd = Math.max(runStart, Math.min(this.buttons.length, Math.round(end)))
    const length = runEnd - runStart
    if (length === 0) return { start: runStart, end: runEnd }
    const targetStart = Math.max(0, Math.min(this.buttons.length - length, runStart + Math.round(delta)))
    const targetEnd = targetStart + length
    if (targetStart === runStart) return { start: runStart, end: runEnd }
    const next = this.buttons.slice()
    for (let frame = runStart; frame < runEnd; frame += 1) next[frame] = { ...next[frame], [action]: false }
    for (let frame = targetStart; frame < targetEnd; frame += 1) next[frame] = { ...next[frame], [action]: true }
    this.buttons = next
    this.invalidateAfterInput(Math.min(runStart, targetStart))
    return { start: targetStart, end: targetEnd }
  }

  resize(frameCount: number): void {
    const size = Math.max(1, Math.min(1_048_576, Math.round(frameCount)))
    if (size === this.buttons.length) return
    if (size > this.buttons.length) {
      const added = size - this.buttons.length
      this.buttons = [...this.buttons, ...Array.from({ length: added }, makeEmptyButtons)]
      this.states = [...this.states, ...Array.from<SimState | undefined>({ length: added }).fill(undefined)]
    } else {
      this.buttons = this.buttons.slice(0, size)
      this.states = this.states.slice(0, size + 1)
      this.revision += 1
    }
    this.emit()
  }

  replace(map: GymMap, initial: SimState, inputs: FrameButtons[]): void {
    this.map = map
    this.buttons = inputs.length ? inputs.map((input) => ({ ...makeEmptyButtons(), ...input })) : [makeEmptyButtons()]
    this.states = Array.from({ length: this.buttons.length + 1 })
    this.states[0] = initial
    this.revision += 1
    this.emit()
  }

  reset(initial: SimState): void {
    this.buttons = this.buttons.map(makeEmptyButtons)
    this.states = Array.from({ length: this.buttons.length + 1 })
    this.states[0] = initial
    this.revision += 1
    this.emit()
  }

  private invalidateAfterInput(frame: number): void {
    this.states = this.states.map((state, stateFrame) => stateFrame > frame ? undefined : state)
    this.revision += 1
    this.emit()
  }

  ensureFrame(target: number): Promise<SimState | undefined> {
    const requested = Math.max(0, Math.round(target))
    this.ensureCapacity(requested)
    const clamped = Math.min(this.buttons.length, requested)
    let result: SimState | undefined
    const task = this.queue.then(async () => {
      const cached = this.states[clamped]
      if (cached) {
        result = cached
        return
      }
      let start = clamped - 1
      while (start > 0 && !this.states[start]) start -= 1
      const checkpoint = this.states[start]
      if (!checkpoint) throw new Error(`找不到帧 ${clamped} 的有效检查点`)
      const revision = this.revision
      const inputs = this.buttons.slice(start, clamped).map((buttons, offset) => {
        const absolute = start + offset
        return buttonsToInput(buttons, absolute > 0 ? this.buttons[absolute - 1] : undefined)
      })
      const trace = await this.runner.simulate(checkpoint, inputs, this.map)
      if (revision !== this.revision) return
      if (trace.length !== inputs.length + 1) throw new Error(`WASM 轨迹长度错误：期望 ${inputs.length + 1}，实际 ${trace.length}`)
      const states = this.states.slice()
      for (let offset = 1; offset < trace.length; offset += 1) states[start + offset] = trace[offset]
      this.states = states
      result = this.states[clamped]
      this.emit()
    })
    this.queue = task.then(() => undefined, () => undefined)
    return task.then(() => result)
  }
}
