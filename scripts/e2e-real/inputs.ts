import type { InputState } from "./types.js";

const EMPTY_INPUT: InputState = Object.freeze({
  move_x: 0,
  move_y: 0,
  jump_pressed: false,
  jump_held: false,
  dash_pressed: false,
  crouch_dash_pressed: false,
  grab_held: false,
  talk_pressed: false,
});

export function input(overrides: Partial<InputState> = {}): InputState {
  return Object.freeze({ ...EMPTY_INPUT, ...overrides });
}

export function inputFrames(
  length: number,
  factory: (frame: number) => InputState = () => EMPTY_INPUT,
): readonly InputState[] {
  if (!Number.isSafeInteger(length) || length < 0)
    throw new Error(`invalid input frame count: ${length}`);
  return Object.freeze(Array.from({ length }, (_, frame) => factory(frame)));
}
