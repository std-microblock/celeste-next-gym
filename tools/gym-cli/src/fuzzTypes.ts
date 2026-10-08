/**
 * TypeScript view of the version-1 Celeste Fuzz specification parsed by
 * `crates/celeste-fuzz/src/model.rs`. Expressions are restricted Rhai:
 * numbers, `+ - * / %`, comparisons, `&& || !`, `abs/min/max/sqrt`, the
 * declared variables, and snapshots `initial`, `before`, `after`, `final`
 * (fields: pos.x/y, speed.x/y, state, facing, dashes, stamina, on_ground,
 * ducking, dead, dash_dir, last_aim, core_mode). Compare states with
 * `state::dash`, `state::normal`, ... (no string literals).
 */
export type FuzzExpr = string;
export type FuzzNumber = number | FuzzExpr;
export type FuzzKey =
  | "left"
  | "right"
  | "up"
  | "down"
  | "dash"
  | "crouch_dash"
  | "jump"
  | "grab";

export interface FuzzVariable {
  name: string;
  range: { from: FuzzNumber; to: FuzzNumber; step?: number };
}

export interface FuzzInput {
  keys: FuzzKey[];
  /** Frame index (0-based, relative to the start state). */
  at: FuzzNumber;
  /** Frames to hold. Required for direction keys; `"hold::inf"` = until the end. Jump defaults to 12 (max var-jump). */
  held_time?: FuzzNumber | "hold::inf";
  /** Expressions over `before` (pre-step) that must hold on frame `at`. */
  before_input?: FuzzExpr | FuzzExpr[];
  /** Expressions over `after` (post-step) that must hold on frame `at`. */
  after_input?: FuzzExpr | FuzzExpr[];
  /** Report this input's frame in `verified_inputs` (default true). */
  verify?: boolean;
}

export interface FuzzObjective {
  type: "maximize" | "minimize" | "approach";
  expression: FuzzExpr;
  target?: number;
}

export interface FuzzCheckpoint {
  at: FuzzNumber;
  success?: FuzzExpr | FuzzExpr[];
  objectives?: FuzzObjective[];
}

export type FuzzOutput =
  | "best"
  | "windows"
  | "coverage"
  | "candidates"
  | "evaluations"
  | `top_${number}`;

export interface FuzzSpec {
  version: 1;
  variables?: FuzzVariable[];
  inputs?: FuzzInput[];
  observe_until: FuzzNumber;
  /** Expressions over `final` that must all be true. */
  success?: FuzzExpr | FuzzExpr[];
  checkpoints?: FuzzCheckpoint[];
  objectives?: FuzzObjective[];
  search?: { bindings?: Record<string, number>; output?: FuzzOutput[] };
  limits?: {
    max_candidates?: number;
    max_input_frames?: number;
    max_trie_nodes?: number;
    max_cache_bytes?: number;
    max_expression_operations?: number;
  };
}

/** Identity helper giving editor type-checking for `.ts` fuzz files. */
export function defineFuzz(spec: FuzzSpec): FuzzSpec {
  return spec;
}

export interface FuzzCandidate {
  bindings: Record<string, number>;
  final_state: Record<string, unknown>;
  objective_values: number[];
  verified_inputs: { input_index: number; frame: number; keys: string[] }[];
  successful: boolean;
}

export interface FuzzResult {
  best: FuzzCandidate | null;
  top: FuzzCandidate[];
  candidates: FuzzCandidate[];
  evaluations: FuzzCandidate[];
  exact_windows: { prefix: Record<string, number>; last_variable: string; intervals: [number, number][] }[];
  connected_regions: {
    bounds: Record<string, [number, number]>;
    successful_count: number;
    density: number;
    best: FuzzCandidate;
  }[];
  coverage_report: unknown;
  stats: {
    candidate_count: number;
    successful_count: number;
    pruned_before: number;
    pruned_after: number;
    naive_frame_count: number;
    unique_simulated_frames: number;
    saved_frames: number;
    trie_nodes: number;
    peak_cache_bytes: number;
    rhai_evaluations: number;
  };
}

export interface FuzzRunResult {
  success: true;
  result: FuzzResult;
  estimated_candidates: number;
  best_inputs: import("./types.ts").SimInput[] | null;
}
