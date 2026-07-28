import { describe, expect, it } from 'vitest'
import { buttonsFromGamepad, latchNewButtons, mergeButtons, type GamepadSnapshot } from './gamepad'
import { makeEmptyButtons } from '../model'

function snapshot(axes: number[] = [0, 0], pressed: number[] = [], values: Record<number, number> = {}): GamepadSnapshot {
  return {
    axes,
    buttons: Array.from({ length: 16 }, (_, index) => ({
      pressed: pressed.includes(index),
      value: values[index] ?? (pressed.includes(index) ? 1 : 0),
    })),
  }
}

describe('buttonsFromGamepad', () => {
  it('uses the left stick with a deadzone', () => {
    const buttons = buttonsFromGamepad(snapshot([-.7, .8]), 'stick')
    expect(buttons).toMatchObject({ left: true, down: true, right: false, up: false })
    expect(buttonsFromGamepad(snapshot([.34, -.34]), 'stick')).toMatchObject({ left: false, right: false, up: false, down: false })
  })

  it('uses only the d-pad when selected', () => {
    const gamepad = snapshot([-.9, .9], [12, 15])
    expect(buttonsFromGamepad(gamepad, 'dpad')).toMatchObject({ up: true, right: true, down: false, left: false })
  })

  it('maps standard face, shoulder, and trigger buttons', () => {
    expect(buttonsFromGamepad(snapshot([0, 0], [0, 2, 5]), 'stick')).toMatchObject({ jump: true, dash: true, grab: true })
    expect(buttonsFromGamepad(snapshot([0, 0], [], { 7: .75 }), 'stick').grab).toBe(true)
  })
})

describe('gamepad button composition', () => {
  it('merges simultaneous keyboard and gamepad actions', () => {
    const keyboard = { ...makeEmptyButtons(), left: true }
    const gamepad = { ...makeEmptyButtons(), jump: true }
    expect(mergeButtons(keyboard, gamepad)).toMatchObject({ left: true, jump: true })
  })

  it('latches a new press until the recording frame consumes it', () => {
    const previous = makeEmptyButtons()
    const pressed = { ...makeEmptyButtons(), dash: true }
    const latched = latchNewButtons(previous, pressed, makeEmptyButtons())
    expect(mergeButtons(makeEmptyButtons(), latched).dash).toBe(true)
    expect(latchNewButtons(pressed, pressed, makeEmptyButtons()).dash).toBe(false)
  })
})
