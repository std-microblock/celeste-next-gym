import { input } from "../inputs.js";
import { AREA_1_TARGET, PLAYGROUND_TARGET } from "../targets.js";
import type { E2EState, ScenarioDefinition } from "../types.js";

export function testScenario(
  name: string,
  overrides: Partial<ScenarioDefinition> = {},
): ScenarioDefinition {
  return {
    name,
    target: AREA_1_TARGET,
    status: "active",
    tags: [],
    techniqueIds: [],
    mapParts: [],
    inputs: [input()],
    ...overrides,
  };
}

export function playgroundScenario(
  name: string,
  overrides: Partial<ScenarioDefinition> = {},
): ScenarioDefinition {
  return testScenario(name, { target: PLAYGROUND_TARGET, ...overrides });
}

export function reflectedState(overrides: Partial<E2EState> = {}): E2EState {
  return {
    _frame: 0,
    _everest_fields: Object.fromEntries(
      Array.from({ length: 101 }, (_, index) => [`field${index}`, index]),
    ),
    pos: [19, 144],
    speed: [0, 0],
    state: "Normal",
    facing: "Right",
    dashes: 1,
    stamina: 110,
    on_ground: false,
    ducking: false,
    dead: false,
    ...overrides,
  };
}
