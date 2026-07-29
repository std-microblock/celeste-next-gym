import type { SimState } from "./model";

export interface PlayerTrailSample {
  frame: number;
  age: number;
}

export type PlayerParticleKind = "dash-streak" | "dust" | "slash";

export interface PlayerParticleSample {
  kind: PlayerParticleKind;
  frame: number;
  age: number;
  origin: { x: number; y: number };
  direction: { x: number; y: number };
  count: number;
  color: string;
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

function normalizedDirection(x: number, y: number): { x: number; y: number } {
  const length = Math.hypot(x, y);
  return length > 0
    ? { x: x / length, y: y / length }
    : { x: 0, y: -1 };
}

function particleLifetime(kind: PlayerParticleKind): number {
  if (kind === "slash") return 12;
  if (kind === "dash-streak") return 14;
  return 24;
}

/** Replays the Player.cs Dust, SlashFx and dash-particle calls from snapshots. */
export function playerParticleSamples(
  states: readonly (SimState | undefined)[],
  frame: number,
): PlayerParticleSample[] {
  const samples: PlayerParticleSample[] = [];
  const emit = (
    kind: PlayerParticleKind,
    emittedFrame: number,
    state: SimState,
    origin: { x: number; y: number },
    direction: { x: number; y: number },
    count: number,
    color = "#d8d3c8",
  ) => {
    const age = frame - emittedFrame;
    if (age >= particleLifetime(kind)) return;
    samples.push({
      kind,
      frame: emittedFrame,
      age,
      origin,
      direction: normalizedDirection(direction.x, direction.y),
      count,
      color: kind === "dash-streak" ? playerTrailColor(state) : color,
    });
  };

  for (let index = Math.max(0, frame - 23); index <= frame; index += 1) {
    const state = states[index];
    if (!state || state.dead) continue;
    const previous = states[index - 1];
    const center = { x: state.pos.x, y: state.pos.y - 6 };
    const bottom = { x: state.pos.x, y: state.pos.y - 1 };
    const dashStarted = isDash(state) && !isDash(previous);

    if (isDash(state) && Math.hypot(state.speed.x, state.speed.y) > 0) {
      emit("dash-streak", index, state, center, state.dash_dir, 1);
    }
    if (dashStarted) {
      emit("slash", index, state, center, state.dash_dir, 1, "#ffffff");
      if (state.state === "RedDash") {
        emit(
          "dust",
          index,
          state,
          bottom,
          { x: -state.dash_dir.x, y: -state.dash_dir.y },
          8,
        );
      }
    }

    if (previous?.on_ground && !state.on_ground && state.speed.y < -20) {
      emit("dust", index, state, bottom, { x: 0, y: -1 }, 4);
    } else if (
      previous?.state === "Climb" &&
      state.state !== "Climb" &&
      state.speed.y < -20 &&
      Math.abs(state.speed.x) > 20
    ) {
      const awayFromWall = Math.sign(state.speed.x) || (state.facing ? 1 : -1);
      emit(
        "dust",
        index,
        state,
        { x: center.x - awayFromWall * 2, y: center.y },
        { x: awayFromWall, y: -1 },
        4,
      );
    }

    if (
      previous &&
      !previous.on_ground &&
      state.on_ground &&
      previous.speed.y >= 80
    ) {
      emit("dust", index, state, bottom, { x: 0, y: -1 }, 8);
    }
  }
  return samples;
}
