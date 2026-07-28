import { ACTIONS, makeEmptyButtons, type FrameButtons } from '../model'

export const GAMEPAD_DIRECTION_SOURCES = ['stick', 'dpad'] as const

export type GamepadDirectionSource = (typeof GAMEPAD_DIRECTION_SOURCES)[number]

export const DEFAULT_GAMEPAD_DIRECTION_SOURCE: GamepadDirectionSource = 'stick'

export interface GamepadSnapshot {
  axes: readonly number[]
  buttons: readonly Pick<GamepadButton, 'pressed' | 'value'>[]
}

const STICK_DEADZONE = 0.35
const BUTTON_THRESHOLD = 0.5

function buttonPressed(gamepad: GamepadSnapshot, index: number): boolean {
  const button = gamepad.buttons[index]
  return Boolean(button && (button.pressed || button.value >= BUTTON_THRESHOLD))
}

export function buttonsFromGamepad(gamepad: GamepadSnapshot, directionSource: GamepadDirectionSource): FrameButtons {
  const buttons = makeEmptyButtons()

  if (directionSource === 'dpad') {
    buttons.up = buttonPressed(gamepad, 12)
    buttons.down = buttonPressed(gamepad, 13)
    buttons.left = buttonPressed(gamepad, 14)
    buttons.right = buttonPressed(gamepad, 15)
  } else {
    const horizontal = gamepad.axes[0] ?? 0
    const vertical = gamepad.axes[1] ?? 0
    buttons.up = vertical < -STICK_DEADZONE
    buttons.down = vertical > STICK_DEADZONE
    buttons.left = horizontal < -STICK_DEADZONE
    buttons.right = horizontal > STICK_DEADZONE
  }

  buttons.jump = buttonPressed(gamepad, 0)
  buttons.dash = buttonPressed(gamepad, 2)
  buttons.grab = [4, 5, 6, 7].some((index) => buttonPressed(gamepad, index))
  return buttons
}

export function mergeButtons(...sources: readonly FrameButtons[]): FrameButtons {
  const merged = makeEmptyButtons()
  for (const action of ACTIONS) merged[action] = sources.some((source) => source[action])
  return merged
}

export function buttonsEqual(left: FrameButtons, right: FrameButtons): boolean {
  return ACTIONS.every((action) => left[action] === right[action])
}

export function latchNewButtons(previous: FrameButtons, current: FrameButtons, latched: FrameButtons): FrameButtons {
  const next = { ...latched }
  for (const action of ACTIONS) {
    if (current[action] && !previous[action]) next[action] = true
  }
  return next
}

export function isGamepadDirectionSource(value: unknown): value is GamepadDirectionSource {
  return typeof value === 'string' && GAMEPAD_DIRECTION_SOURCES.includes(value as GamepadDirectionSource)
}
