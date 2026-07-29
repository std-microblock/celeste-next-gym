import type { ScenarioDefinition } from "../types.js";
import { scenarios as area1Scenarios } from "./area-1/index.js";
import { scenarios as area2Scenarios } from "./area-2/index.js";
import { scenarios as area4Scenarios } from "./area-4/index.js";
import { scenarios as playgroundScenarios } from "./playground/index.js";

export const scenarios: readonly ScenarioDefinition[] = Object.freeze([
  ...playgroundScenarios,
  ...area1Scenarios,
  ...area2Scenarios,
  ...area4Scenarios,
]);
