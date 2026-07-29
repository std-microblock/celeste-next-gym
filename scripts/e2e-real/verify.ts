import { CORE_TOLERANCE } from "./constants.js";
import type {
  CoreSnapshot,
  E2EState,
  InputState,
  PlayerSnapshot,
  ScenarioDefinition,
  VerifyContext,
} from "./types.js";

export function near(
  actual: number | undefined,
  expected: number,
  tolerance = CORE_TOLERANCE,
): boolean {
  return actual !== undefined && Math.abs(actual - expected) <= tolerance;
}

export function field<T = unknown>(
  state: E2EState | undefined,
  name: string,
): T | undefined {
  return state?._everest_fields[name] as T | undefined;
}

export function semanticAssert(
  condition: unknown,
  scenarioName: string,
  message: string,
): asserts condition {
  if (!condition)
    throw new Error(
      `${scenarioName}: semantic verification failed: ${message}`,
    );
}

export function pickCore(state: E2EState | undefined): CoreSnapshot | null {
  if (!state) return null;
  return {
    frame: state._frame,
    pos: state.pos,
    speed: state.speed,
    state: state.state,
    facing: state.facing,
    dashes: state.dashes,
    stamina: state.stamina,
    on_ground: state.on_ground,
    ducking: state.ducking,
    dead: state.dead,
  };
}

export function createVerifyContext(options: {
  scenario: ScenarioDefinition;
  inputs: readonly InputState[];
  initialSnapshot: PlayerSnapshot;
  room: string | undefined;
  mapPath: string;
  tracePath: string;
}): VerifyContext {
  const scenarioName = options.scenario.name;
  return {
    scenarioName,
    target: options.scenario.target,
    inputs: options.inputs,
    initialSnapshot: options.initialSnapshot,
    room: options.room,
    mapPath: options.mapPath,
    tracePath: options.tracePath,
    tolerance: CORE_TOLERANCE,
    near,
    field,
    core: pickCore,
    assert(
      condition: unknown,
      message: string,
      details?: unknown,
    ): asserts condition {
      if (condition) return;
      const suffix =
        details === undefined ? "" : `: ${JSON.stringify(details)}`;
      throw new Error(
        `${scenarioName}: semantic verification failed: ${message}${suffix}`,
      );
    },
  };
}
