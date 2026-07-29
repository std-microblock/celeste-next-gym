import type { SimState } from "./model";

export interface PlayerTrailSample {
  frame: number;
  age: number;
}

const TRAIL_LIFETIME_FRAMES = 24;

function isDash(state: SimState | undefined): boolean {
  return state?.state === "Dash" || state?.state === "RedDash";
}

function continuousStateAge(
  states: readonly (SimState | undefined)[],
  frame: number,
  name: string,
): number {
  let age = 0;
  for (let index = frame - 1; index >= 0; index -= 1) {
    if (states[index]?.state !== name) break;
    age += 1;
  }
  return age;
}

/**
 * Rebuilds Celeste's player trail emission frames from deterministic snapshots.
 * Player.cs emits at dash start, 0.08 s later and at dash end; DreamDash and
 * StarFly emit on 0.10 s and 0.05 s intervals respectively.
 */
export function playerTrailSamples(
  states: readonly (SimState | undefined)[],
  frame: number,
): PlayerTrailSample[] {
  const first = Math.max(0, frame - TRAIL_LIFETIME_FRAMES);
  const samples: PlayerTrailSample[] = [];
  for (let index = first; index <= frame; index += 1) {
    const state = states[index];
    if (!state || state.dead) continue;
    const previous = states[index - 1];
    const dashStarted = isDash(state) && !isDash(previous);
    const dashMiddle =
      isDash(state) && isDash(previous) && continuousStateAge(states, index, state.state) === 5;
    const dashEnded = !isDash(state) && isDash(previous);
    const dreamDashInterval =
      state.state === "DreamDash" &&
      continuousStateAge(states, index, "DreamDash") % 6 === 0;
    const starFlyInterval =
      state.state === "StarFly" &&
      continuousStateAge(states, index, "StarFly") % 3 === 0;
    if (
      dashStarted ||
      dashMiddle ||
      dashEnded ||
      dreamDashInterval ||
      starFlyInterval
    ) {
      samples.push({ frame: index, age: frame - index });
    }
  }
  return samples;
}

export function playerTrailOpacity(age: number): number {
  return Math.max(0, 1 - age / TRAIL_LIFETIME_FRAMES) * 0.55;
}

export function playerTrailColor(state: SimState): string {
  return state.dashes >= 2 ? "#ff6def" : state.dashes === 0 ? "#44b7ff" : "#ac3232";
}
