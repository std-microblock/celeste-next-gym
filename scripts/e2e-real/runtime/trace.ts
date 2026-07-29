import { mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

import type { E2EState, InputState } from "../types.js";

export function writeTrace(
  tracePath: string,
  inputs: readonly InputState[],
  states: readonly E2EState[],
): void {
  mkdirSync(dirname(tracePath), { recursive: true });
  writeFileSync(tracePath, JSON.stringify({ inputs, states }), "utf8");
}
