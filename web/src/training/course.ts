import type { SimState } from "../model.ts";
import type {
  TrainingMapDocument,
  TrainingModule,
  TrainingTrigger,
} from "./catalog.ts";

export interface TrainingObjectiveSeries {
  expression: string;
  points: Array<{ frame: number; value: number; successful: boolean }>;
}

export interface TrainingCompletion {
  moduleId: string;
  title: string;
  triggerFrame: number;
  startedFrame: number;
  completedFrame: number;
  targetFrame?: number;
  actualInputFrame?: number;
  accuracy: number;
  reactionFrames: number;
  objectiveValues: number[];
  bestObjectiveValues: number[];
  objectives: TrainingObjectiveSeries[];
  windows: Array<{ from: number; to: number }>;
  actualInputs: Array<{ frame: number; keys: string[] }>;
}

const PLAYER_HALF_WIDTH = 4;
const STANDING_HEIGHT = 11;
const DUCKING_HEIGHT = 6;

/** Training triggers use the same bottom-centred player position as SimState. */
export function triggerContainsPlayer(
  trigger: TrainingTrigger,
  state: SimState,
): boolean {
  const player = {
    left: state.pos.x - PLAYER_HALF_WIDTH,
    right: state.pos.x + PLAYER_HALF_WIDTH,
    top: state.pos.y - (state.ducking ? DUCKING_HEIGHT : STANDING_HEIGHT),
    bottom: state.pos.y,
  };
  const bounds = trigger.bounds;
  return (
    player.right >= bounds.x &&
    player.left <= bounds.x + bounds.width &&
    player.bottom >= bounds.y &&
    player.top <= bounds.y + bounds.height
  );
}

export function moduleAtPlayer(
  document: TrainingMapDocument,
  state: SimState,
  completedIds: ReadonlySet<string>,
): TrainingModule | undefined {
  return document.modules.find(
    (module) =>
      !completedIds.has(module.id) &&
      triggerContainsPlayer(module.trigger, state),
  );
}

export function allModulesCompleted(
  document: TrainingMapDocument,
  completedIds: ReadonlySet<string>,
): boolean {
  return document.modules.every((module) => completedIds.has(module.id));
}

/** Relative output quality: exact Fuzz-best output is 100%. */
export function outputAccuracy(
  actual: number | undefined,
  best: number | undefined,
): number {
  if (
    actual === undefined ||
    best === undefined ||
    !Number.isFinite(actual) ||
    !Number.isFinite(best)
  )
    return 0;
  if (Math.abs(best) < 1e-9) return Math.abs(actual - best) < 1e-9 ? 100 : 0;
  return Math.max(
    0,
    Math.min(100, (1 - Math.abs(actual - best) / Math.abs(best)) * 100),
  );
}

export function objectiveOutputName(expression: string): string {
  if (expression === "final.speed.x") return "水平速度";
  if (expression === "final.speed.y") return "垂直速度";
  if (
    expression === "final.speed.x.abs()" ||
    expression === "abs(final.speed.x)"
  )
    return "水平速度绝对值";
  if (
    expression === "final.speed.y.abs()" ||
    expression === "abs(final.speed.y)"
  )
    return "垂直速度绝对值";
  return expression;
}

export function formatObjectiveOutput(
  expression: string,
  value: number | undefined,
): string {
  if (value === undefined || !Number.isFinite(value)) return "—";
  const formatted = Number.isInteger(value)
    ? value.toFixed(0)
    : value.toFixed(2);
  return expression.includes("speed.") ? `${formatted} px/s` : formatted;
}

export function average(values: readonly number[]): number {
  return values.length === 0
    ? 0
    : values.reduce((sum, value) => sum + value, 0) / values.length;
}
