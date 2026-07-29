import { spawnSync } from "node:child_process";
import { isDeepStrictEqual } from "node:util";
import { readFileSync, readdirSync } from "node:fs";
import { basename, join } from "node:path";

type JsonPrimitive = boolean | number | string | null;
type JsonValue =
  | JsonPrimitive
  | JsonValue[]
  | { readonly [key: string]: JsonValue };
type AssertionOperator = "eq" | "near" | "gt" | "gte" | "lt" | "lte";

export interface TimelineAssertion {
  readonly frame: number;
  readonly path: string;
  readonly op: AssertionOperator;
  readonly value: JsonValue;
  readonly tolerance?: number;
}

export interface TimelineFixture {
  readonly name: string;
  readonly path: string;
  readonly document: Record<string, unknown>;
  readonly inputCount: number;
  readonly e2eScenario: string;
  readonly assertions: readonly TimelineAssertion[];
}

interface ReplayResponse {
  readonly input_count: number;
  readonly state_count: number;
  readonly states: Readonly<Record<string, unknown>>;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function invalid(path: string, detail: string): never {
  throw new Error(`${path}: ${detail}`);
}

function parseAssertion(
  value: unknown,
  fixturePath: string,
  index: number,
  inputCount: number,
): TimelineAssertion {
  const label = `regression.assertions[${index}]`;
  if (!isRecord(value)) invalid(fixturePath, `${label} must be an object`);
  const { frame, path, op, value: expected, tolerance } = value;
  if (
    !Number.isInteger(frame) ||
    Number(frame) < 0 ||
    Number(frame) > inputCount
  ) {
    invalid(
      fixturePath,
      `${label}.frame must be an integer from 0 through ${inputCount}`,
    );
  }
  if (
    typeof path !== "string" ||
    path.length === 0 ||
    path.split(".").some((part) => part.length === 0)
  ) {
    invalid(fixturePath, `${label}.path must be a dot-separated state path`);
  }
  if (!["eq", "near", "gt", "gte", "lt", "lte"].includes(String(op))) {
    invalid(fixturePath, `${label}.op is not supported`);
  }
  if (expected === undefined)
    invalid(fixturePath, `${label}.value is required`);
  if (op === "near") {
    if (
      typeof expected !== "number" ||
      typeof tolerance !== "number" ||
      !Number.isFinite(tolerance) ||
      tolerance <= 0
    ) {
      invalid(
        fixturePath,
        `${label} near assertions require a numeric value and positive tolerance`,
      );
    }
  } else if (op !== "eq" && typeof expected !== "number") {
    invalid(
      fixturePath,
      `${label} ${String(op)} assertions require a numeric value`,
    );
  }
  return {
    frame: Number(frame),
    path,
    op: op as AssertionOperator,
    value: expected as JsonValue,
    ...(tolerance === undefined ? {} : { tolerance: Number(tolerance) }),
  };
}

export function loadTimelineFixture(path: string): TimelineFixture {
  let document: unknown;
  try {
    document = JSON.parse(readFileSync(path, "utf8")) as unknown;
  } catch (error) {
    invalid(path, `cannot parse JSON: ${String(error)}`);
  }
  if (!isRecord(document)) invalid(path, "timeline root must be an object");
  if (document.version !== 2) invalid(path, "timeline version must be 2");
  if (!isRecord(document.map)) invalid(path, "map must be an object");
  if (!isRecord(document.initial_state))
    invalid(path, "initial_state must be an object");
  if (!Array.isArray(document.inputs) || document.inputs.length === 0)
    invalid(path, "inputs must be a non-empty array");
  if (
    !isRecord(document.regression) ||
    !Array.isArray(document.regression.assertions) ||
    document.regression.assertions.length === 0
  ) {
    invalid(path, "regression.assertions must be a non-empty array");
  }
  if (
    typeof document.regression.e2e_scenario !== "string" ||
    !/^[a-z0-9]+(?:-[a-z0-9.]+)*$/.test(document.regression.e2e_scenario)
  ) {
    invalid(
      path,
      "regression.e2e_scenario must name its real Everest E2E scenario",
    );
  }
  const inputs = document.inputs;
  const assertions = document.regression.assertions.map((assertion, index) =>
    parseAssertion(assertion, path, index, inputs.length),
  );
  return {
    name: basename(path, ".json"),
    path,
    document,
    inputCount: inputs.length,
    e2eScenario: document.regression.e2e_scenario,
    assertions,
  };
}

export function discoverTimelineFixtures(
  directory: string,
): readonly TimelineFixture[] {
  const paths = readdirSync(directory, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".json"))
    .map((entry) => join(directory, entry.name))
    .sort((left, right) => left.localeCompare(right));
  if (paths.length === 0)
    throw new Error(`${directory}: no timeline JSON fixtures found`);
  return paths.map(loadTimelineFixture);
}

function replayTimeline(
  fixture: TimelineFixture,
  repoRoot: string,
): ReplayResponse {
  const frames = [
    ...new Set(fixture.assertions.map((assertion) => assertion.frame)),
  ].sort((left, right) => left - right);
  const result = spawnSync(
    "cargo",
    ["run", "--quiet", "-p", "celeste-physics", "--example", "replay_timeline"],
    {
      cwd: repoRoot,
      encoding: "utf8",
      input: JSON.stringify({ timeline: fixture.document, frames }),
      maxBuffer: 16 * 1024 * 1024,
      shell: false,
    },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(
      `${fixture.path}: timeline replay exited with ${String(result.status)}: ${result.stderr.trim()}`,
    );
  }
  try {
    return JSON.parse(result.stdout) as ReplayResponse;
  } catch (error) {
    throw new Error(
      `${fixture.path}: replay returned invalid JSON: ${String(error)}`,
    );
  }
}

function valueAtPath(root: unknown, path: string): unknown {
  let value = root;
  for (const part of path.split(".")) {
    if (!isRecord(value) || !(part in value))
      throw new Error(`state path ${path} does not exist`);
    value = value[part];
  }
  return value;
}

function assertionPasses(
  actual: unknown,
  assertion: TimelineAssertion,
): boolean {
  switch (assertion.op) {
    case "eq":
      return isDeepStrictEqual(actual, assertion.value);
    case "near":
      return (
        typeof actual === "number" &&
        typeof assertion.value === "number" &&
        Math.abs(actual - assertion.value) <= (assertion.tolerance ?? 0)
      );
    case "gt":
      return typeof actual === "number" && actual > Number(assertion.value);
    case "gte":
      return typeof actual === "number" && actual >= Number(assertion.value);
    case "lt":
      return typeof actual === "number" && actual < Number(assertion.value);
    case "lte":
      return typeof actual === "number" && actual <= Number(assertion.value);
  }
}

export function runTimelineRegression(
  fixture: TimelineFixture,
  repoRoot: string,
): void {
  const replay = replayTimeline(fixture, repoRoot);
  if (
    replay.input_count !== fixture.inputCount ||
    replay.state_count !== fixture.inputCount + 1
  ) {
    throw new Error(
      `${fixture.path}: replay returned ${replay.input_count} inputs and ${replay.state_count} states; expected ${fixture.inputCount} and ${fixture.inputCount + 1}`,
    );
  }
  for (const assertion of fixture.assertions) {
    const state = replay.states[String(assertion.frame)];
    if (state === undefined)
      throw new Error(
        `${fixture.path}: replay omitted requested frame ${assertion.frame}`,
      );
    const actual = valueAtPath(state, assertion.path);
    if (!assertionPasses(actual, assertion)) {
      throw new Error(
        `${fixture.name} frame ${assertion.frame} ${assertion.path}: expected ${assertion.op} ${JSON.stringify(assertion.value)}, got ${JSON.stringify(actual)}`,
      );
    }
  }
}
