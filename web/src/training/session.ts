import {
  ACTIONS,
  makeEmptyButtons,
  type Action,
  type FrameButtons,
  type SimState,
} from "../model.ts";

export type TrainingPhase = "pre_fuzz" | "fuzz" | "failed" | "success";
export type TrainingFailure =
  | "entry_check_failed"
  | "input_order_mismatch"
  | "timing_window_miss";

export interface TrainingInput {
  id: string;
  keys: string[];
  at: number | string;
  verify?: boolean;
  held_time?: number | string;
  before_input?: string | string[];
  after_input?: string | string[];
}

export type TrainingObjective =
  | { type: "maximize" | "minimize"; expression: string }
  | { type: "approach"; expression: string; target: number };

export interface TrainingCheckpoint {
  id: string;
  at: number | string;
  description: string;
  success?: string[];
  objectives: TrainingObjective[];
}

export interface TrainingCandidateInput {
  input_index: number;
  frame: number;
  keys: string[];
}

export interface TrainingCandidate {
  bindings: Record<string, number>;
  verified_inputs: TrainingCandidateInput[];
  objective_values: number[];
  successful: boolean;
  final_state?: { speed?: { x: number; y: number } };
}

export interface TrainingObjectivePoint {
  frame: number;
  values: number[];
  successful: boolean;
}

export interface TrainingDefinition {
  id: string;
  title: string;
  entry: { input_id: string; hint: string };
  fuzz: { inputs: TrainingInput[]; checkpoints?: TrainingCheckpoint[] };
}

export interface TrainingSession {
  phase: TrainingPhase;
  entryFrame: number | null;
  nextVerifiedInput: number;
  candidates: TrainingCandidate[];
  allCandidates: TrainingCandidate[];
  actualInputs: Array<{ frame: number; keys: string[] }>;
  failure?: {
    kind: TrainingFailure;
    frame: number;
    expectedWindow: FrameWindow[];
  };
}

export interface FrameWindow {
  from: number;
  to: number;
}

export type IndexedTrainingInput = TrainingInput & { fuzzInputIndex: number };

const DIRECTION_KEYS = ["up", "down", "left", "right"] as const;
const PRESS_KEYS = ACTIONS.filter(
  (key) => !DIRECTION_KEYS.includes(key as (typeof DIRECTION_KEYS)[number]),
);

function isAction(key: string): key is Action {
  return ACTIONS.includes(key as Action);
}

export function createTrainingSession(
  candidates: TrainingCandidate[],
  definition?: TrainingDefinition,
): TrainingSession {
  const entryIndex =
    definition === undefined
      ? 0
      : verifiedInputs(definition).findIndex(
          (input) => input.id === definition.entry.input_id,
        );
  return {
    phase: "pre_fuzz",
    entryFrame: null,
    nextVerifiedInput: Math.max(0, entryIndex),
    candidates: [...candidates],
    allCandidates: [...candidates],
    actualInputs: [],
  };
}

export function keySemantics(buttons: FrameButtons): string[] {
  return ACTIONS.filter((key) => buttons[key]);
}

export function sameKeySemantics(
  actual: readonly string[],
  expected: readonly string[],
): boolean {
  const normalizedActual = [...new Set(actual)].sort();
  const normalizedExpected = [...new Set(expected)].sort();
  return (
    normalizedActual.length === normalizedExpected.length &&
    normalizedActual.every((key, index) => key === normalizedExpected[index])
  );
}

export function verifiedInputs(
  definition: TrainingDefinition,
): IndexedTrainingInput[] {
  return definition.fuzz.inputs.flatMap((input, fuzzInputIndex) =>
    input.verify === false ? [] : [{ ...input, fuzzInputIndex }],
  );
}

export function trainingEntryInput(
  definition: TrainingDefinition,
): IndexedTrainingInput | undefined {
  return verifiedInputs(definition).find(
    (input) => input.id === definition.entry.input_id,
  );
}

export function currentTrainingInput(
  session: TrainingSession,
  definition: TrainingDefinition,
): IndexedTrainingInput | undefined {
  return session.phase === "pre_fuzz"
    ? trainingEntryInput(definition)
    : verifiedInputs(definition)[session.nextVerifiedInput];
}

/** True when the expected combination has just become active, or another action was attempted. */
export function trainingVerificationTriggered(
  current: FrameButtons,
  previous: FrameButtons,
  input: TrainingInput | undefined,
): boolean {
  if (!input) return false;
  const expected = input.keys.filter(isAction);
  const expectedTriggered =
    expected.length > 0 &&
    expected.every((key) => current[key]) &&
    expected.some((key) => !previous[key]);
  const actionTriggered = PRESS_KEYS.some(
    (key) => current[key] && !previous[key],
  );
  return expectedTriggered || actionTriggered;
}

/** Checks author-defined F0 holds while keeping directional matching exact. */
export function trainingEntryContextPassed(
  buttons: FrameButtons,
  definition: TrainingDefinition,
): boolean {
  const entry = trainingEntryInput(definition);
  if (!entry) return false;
  const heldKeys = definition.fuzz.inputs
    .filter((input) => input.verify === false && input.at === entry.at)
    .flatMap((input) => input.keys)
    .filter(isAction);
  const expectedDirections = new Set(
    [...heldKeys, ...entry.keys].filter(
      (key): key is (typeof DIRECTION_KEYS)[number] =>
        DIRECTION_KEYS.includes(key as (typeof DIRECTION_KEYS)[number]),
    ),
  );
  return (
    DIRECTION_KEYS.every(
      (direction) => buttons[direction] === expectedDirections.has(direction),
    ) &&
    heldKeys
      .filter(
        (key) =>
          !DIRECTION_KEYS.includes(key as (typeof DIRECTION_KEYS)[number]),
      )
      .every((key) => buttons[key])
  );
}

export function verificationKeys(
  buttons: FrameButtons,
  previous: FrameButtons,
  input: TrainingInput | undefined,
): string[] {
  const expected = input?.keys ?? [];
  const actual = keySemantics(buttons);
  const newlyPressed = actual.filter((key) => !previous[key as Action]);
  // With verify:false direction holds, only the new action belongs to the
  // teaching input. A definition that includes a direction in verify:true
  // deliberately opts into strict directional matching.
  return expected.some((key) =>
    DIRECTION_KEYS.includes(key as (typeof DIRECTION_KEYS)[number]),
  )
    ? actual.filter(
        (key) =>
          DIRECTION_KEYS.includes(key as (typeof DIRECTION_KEYS)[number]) ||
          newlyPressed.includes(key),
      )
    : newlyPressed.filter(
        (key) =>
          !DIRECTION_KEYS.includes(key as (typeof DIRECTION_KEYS)[number]),
      );
}

function candidateInput(
  candidate: TrainingCandidate,
  inputIndex: number,
): TrainingCandidateInput | undefined {
  return candidate.verified_inputs.find(
    (input) => input.input_index === inputIndex,
  );
}

export function candidateWindow(
  candidates: readonly TrainingCandidate[],
  inputIndex: number,
): FrameWindow[] {
  const frames = [
    ...new Set(
      candidates
        .map((candidate) => candidateInput(candidate, inputIndex)?.frame)
        .filter((frame): frame is number => frame !== undefined),
    ),
  ].sort((left, right) => left - right);
  const windows: FrameWindow[] = [];
  for (const frame of frames) {
    const current = windows.at(-1);
    if (current && frame <= current.to + 1) current.to = frame;
    else windows.push({ from: frame, to: frame });
  }
  return windows;
}

export function nextTargetFrame(
  candidates: readonly TrainingCandidate[],
  inputIndex: number,
): number | undefined {
  return candidates.length === 0
    ? undefined
    : candidateInput(candidates[0], inputIndex)?.frame;
}

/** True only when the currently requested input has just become active. */
export function expectedTrainingInputTriggered(
  current: FrameButtons,
  previous: FrameButtons,
  input: TrainingInput | undefined,
): boolean {
  if (!input) return false;
  const expected = input.keys.filter(isAction);
  return (
    expected.length > 0 &&
    expected.every((key) => current[key]) &&
    expected.some((key) => !previous[key])
  );
}

function resolvedCandidateInputFrame(
  candidate: TrainingCandidate,
  input: TrainingInput,
  inputIndex: number,
): number | undefined {
  const verifiedFrame = candidateInput(candidate, inputIndex)?.frame;
  if (verifiedFrame !== undefined) return verifiedFrame;
  if (typeof input.at === "number") return input.at;
  const bound = candidate.bindings[input.at];
  return Number.isFinite(bound) ? bound : undefined;
}

function resolvedHoldFrames(
  candidate: TrainingCandidate,
  heldTime: TrainingInput["held_time"],
): number | "infinite" {
  if (heldTime === "hold::inf") return "infinite";
  if (typeof heldTime === "number") return Math.max(1, heldTime);
  if (typeof heldTime === "string") {
    const bound = candidate.bindings[heldTime];
    if (Number.isFinite(bound)) return Math.max(1, bound);
  }
  return 1;
}

/** Builds the frame-exact controls for replaying the best existing candidate. */
export function trainingReferenceButtons(
  candidate: TrainingCandidate,
  definition: TrainingDefinition,
  frame: number,
): FrameButtons {
  const buttons = makeEmptyButtons();
  definition.fuzz.inputs.forEach((input, inputIndex) => {
    const at = resolvedCandidateInputFrame(candidate, input, inputIndex);
    if (at === undefined) return;
    const heldFrames = resolvedHoldFrames(candidate, input.held_time);
    const active =
      heldFrames === "infinite"
        ? frame >= at
        : frame >= at && frame < at + heldFrames;
    if (!active) return;
    for (const key of input.keys) {
      if (isAction(key)) buttons[key] = true;
    }
  });
  return buttons;
}

/** Leaves enough post-input frames for the player to see the demonstrated result. */
export function trainingReferenceEndFrame(
  candidate: TrainingCandidate,
  definition: TrainingDefinition,
  tailFrames = 24,
): number {
  const lastInput = definition.fuzz.inputs.reduce((last, input, inputIndex) => {
    const at = resolvedCandidateInputFrame(candidate, input, inputIndex);
    if (at === undefined) return last;
    const heldFrames = resolvedHoldFrames(candidate, input.held_time);
    return Math.max(last, at + (heldFrames === "infinite" ? 0 : heldFrames - 1));
  }, 0);
  return lastInput + Math.max(1, tailFrames);
}

export interface TrainingReferenceStep {
  inputIndex: number;
  inputId: string;
  frame: number;
  keys: string[];
}

/** Ordered tutorial checkpoints, beginning at the declared entry input. */
export function trainingReferenceSteps(
  candidate: TrainingCandidate,
  definition: TrainingDefinition,
): TrainingReferenceStep[] {
  const inputs = verifiedInputs(definition);
  const entryIndex = inputs.findIndex(
    (input) => input.id === definition.entry.input_id,
  );
  const included = new Map(
    inputs
      .slice(Math.max(0, entryIndex))
      .map((input) => [input.fuzzInputIndex, input]),
  );
  return candidate.verified_inputs
    .flatMap((candidateInput) => {
      const input = included.get(candidateInput.input_index);
      return input
        ? [
            {
              inputIndex: candidateInput.input_index,
              inputId: input.id,
              frame: candidateInput.frame,
              keys: [...candidateInput.keys],
            },
          ]
        : [];
    })
    .sort(
      (left, right) =>
        left.frame - right.frame || left.inputIndex - right.inputIndex,
    );
}

export interface AssistedBrake {
  multiplier: number;
  braking: boolean;
  stopped: boolean;
  stopFrame?: number;
}

/** Smoothly reaches a full stop on a reference input frame. */
export function referenceStepBrake(
  frame: number,
  targetFrame: number | undefined,
  radiusFrames = 10,
): AssistedBrake {
  if (targetFrame === undefined)
    return { multiplier: 1, braking: false, stopped: false };
  const brakeStart = Math.max(0, targetFrame - Math.max(1, radiusFrames));
  if (frame < brakeStart)
    return {
      multiplier: 1,
      braking: false,
      stopped: false,
      stopFrame: targetFrame,
    };
  if (frame >= targetFrame)
    return {
      multiplier: 0,
      braking: true,
      stopped: true,
      stopFrame: targetFrame,
    };
  const span = Math.max(1, targetFrame - brakeStart);
  const progress = Math.max(0, Math.min(1, (frame - brakeStart) / span));
  const smooth = progress * progress * (3 - 2 * progress);
  return {
    multiplier: 1 - smooth,
    braking: true,
    stopped: false,
    stopFrame: targetFrame,
  };
}

/**
 * Brakes through the latter half of the feasible window and reaches zero at
 * 85% of it (frame 17 in a 20-frame window). The simulation remains
 * frame-exact; only the wall-clock cadence changes.
 */
export function assistedWindowBrake(
  candidates: readonly TrainingCandidate[],
  inputIndex: number,
  frame: number,
): AssistedBrake {
  const windows = candidateWindow(candidates, inputIndex);
  const window = windows.find((candidate) => frame <= candidate.to) ?? windows.at(-1);
  if (!window) return { multiplier: 1, braking: false, stopped: false };
  const length = window.to - window.from + 1;
  const stopFrame = window.from + Math.min(length - 1, Math.floor(length * 0.85));
  const midpoint = window.from + Math.floor(length * 0.5);
  const brakeStart = Math.min(midpoint, Math.max(0, stopFrame - 3));
  if (frame < brakeStart)
    return { multiplier: 1, braking: false, stopped: false, stopFrame };
  if (frame >= stopFrame)
    return { multiplier: 0, braking: true, stopped: true, stopFrame };
  const span = Math.max(1, stopFrame - brakeStart);
  const progress = Math.max(0, Math.min(1, (frame - brakeStart) / span));
  const smooth = progress * progress * (3 - 2 * progress);
  return {
    multiplier: 1 - smooth,
    braking: true,
    stopped: false,
    stopFrame,
  };
}

/** Duplicate frames retain the first candidate supplied by the caller. */
export function candidateObjectivePoints(
  candidates: readonly TrainingCandidate[],
  inputIndex: number,
): TrainingObjectivePoint[] {
  const points = new Map<number, { values: number[]; successful: boolean }>();
  for (const candidate of candidates) {
    const frame = candidateInput(candidate, inputIndex)?.frame;
    if (frame !== undefined && !points.has(frame))
      points.set(frame, {
        values: candidate.objective_values,
        successful: candidate.successful,
      });
  }
  return [...points]
    .sort(([left], [right]) => left - right)
    .map(([frame, point]) => ({
      frame,
      ...point,
    }));
}

function candidatesMatchingTrainingInputs(
  candidates: readonly TrainingCandidate[],
  definition: TrainingDefinition,
  actualInputs: readonly { frame: number; keys: readonly string[] }[],
): TrainingCandidate[] {
  const inputs = verifiedInputs(definition);
  const entryIndex = inputs.findIndex(
    (input) => input.id === definition.entry.input_id,
  );
  return candidates.filter((candidate) =>
    actualInputs.every((actual, index) => {
      const input = inputs[entryIndex + index];
      const expected =
        input === undefined
          ? undefined
          : candidateInput(candidate, input.fuzzInputIndex);
      return (
        expected?.frame === actual.frame &&
        sameKeySemantics(expected.keys, actual.keys)
      );
    }),
  );
}

/**
 * Returns the output of performing the current operation on each candidate
 * frame, while holding every operation the player already performed fixed.
 * Successful candidates are objective-ranked by Rust and therefore take
 * precedence over failed evaluations that share the same operation frame.
 */
export function candidateOperationObjectivePoints(
  candidates: readonly TrainingCandidate[],
  evaluations: readonly TrainingCandidate[],
  definition: TrainingDefinition,
  inputIndex: number,
  actualInputs: readonly { frame: number; keys: readonly string[] }[],
): TrainingObjectivePoint[] {
  const matchingCandidates = candidatesMatchingTrainingInputs(
    candidates,
    definition,
    actualInputs,
  );
  const matchingEvaluations = candidatesMatchingTrainingInputs(
    evaluations,
    definition,
    actualInputs,
  );
  return candidateObjectivePoints(
    [...matchingCandidates, ...matchingEvaluations],
    inputIndex,
  );
}

export function matchingTrainingCandidate(
  candidates: readonly TrainingCandidate[],
  definition: TrainingDefinition,
  actualInputs: readonly { frame: number; keys: readonly string[] }[],
): TrainingCandidate | undefined {
  return candidatesMatchingTrainingInputs(
    candidates,
    definition,
    actualInputs,
  )[0];
}

function failed(
  session: TrainingSession,
  kind: TrainingFailure,
  frame: number,
  fuzzInputIndex: number,
): TrainingSession {
  return {
    ...session,
    phase: "failed",
    failure: {
      kind,
      frame,
      expectedWindow: candidateWindow(session.candidates, fuzzInputIndex),
    },
  };
}

/**
 * Records a verified key press. `frame` is the training-local Fuzz frame: the
 * accepted entry is always F0, and every later input is relative to it.
 */
export function verifyTrainingInput(
  session: TrainingSession,
  definition: TrainingDefinition,
  frame: number,
  keys: readonly string[],
  entryCheckPassed = true,
): TrainingSession {
  if (session.phase === "failed" || session.phase === "success") return session;
  const inputs = verifiedInputs(definition);
  const input = currentTrainingInput(session, definition);
  if (session.phase === "pre_fuzz" && !input)
    throw new Error(
      `Training ${definition.id} entry.input_id ${definition.entry.input_id} does not name a verified Fuzz input`,
    );
  if (!input) return { ...session, phase: "success" };
  if (!sameKeySemantics(keys, input.keys)) {
    return session.phase === "pre_fuzz"
      ? failed(session, "entry_check_failed", frame, input.fuzzInputIndex)
      : failed(session, "input_order_mismatch", frame, input.fuzzInputIndex);
  }
  if (session.phase === "pre_fuzz" && !entryCheckPassed)
    return failed(session, "entry_check_failed", frame, input.fuzzInputIndex);

  const matching = session.candidates.filter((candidate) => {
    const expected = candidateInput(candidate, input.fuzzInputIndex);
    return (
      expected !== undefined &&
      expected.frame === frame &&
      sameKeySemantics(keys, expected.keys)
    );
  });
  if (matching.length === 0)
    return failed(
      session,
      session.phase === "pre_fuzz"
        ? "entry_check_failed"
        : "timing_window_miss",
      frame,
      input.fuzzInputIndex,
    );

  const next = {
    ...session,
    phase: "fuzz" as TrainingPhase,
    entryFrame: session.phase === "pre_fuzz" ? frame : session.entryFrame,
    nextVerifiedInput:
      inputs.findIndex(
        (candidate) => candidate.fuzzInputIndex === input.fuzzInputIndex,
      ) + 1,
    candidates: matching,
    actualInputs: [...session.actualInputs, { frame, keys: [...keys] }],
    failure: undefined,
  };
  return next.nextVerifiedInput >= inputs.length
    ? { ...next, phase: "success" }
    : next;
}

export function rebuildTrainingSession(
  definition: TrainingDefinition,
  candidates: TrainingCandidate[],
  inputs: readonly {
    frame: number;
    keys: readonly string[];
    entryCheckPassed?: boolean;
  }[],
): TrainingSession {
  let session = createTrainingSession(candidates, definition);
  for (const input of inputs) {
    session = verifyTrainingInput(
      session,
      definition,
      input.frame,
      input.keys,
      input.entryCheckPassed,
    );
    if (session.phase === "failed") break;
  }
  return session;
}

export function assistedRate(
  baseRate: number,
  fuzzFrame: number,
  targetFrame: number | undefined,
  radiusFrames: number,
  minimumMultiplier: number,
): number {
  if (targetFrame === undefined || radiusFrames <= 0) return baseRate;
  const distance = Math.max(targetFrame - fuzzFrame, 0);
  const progress = Math.max(0, Math.min(1, 1 - distance / radiusFrames));
  return baseRate * (1 - progress * (1 - minimumMultiplier));
}

export function entryCheckSnapshot(state: SimState): SimState {
  // Kept as a named boundary so the UI never evaluates author expressions in
  // JavaScript. The Rust Fuzz bridge owns the Rhai evaluation.
  return state;
}
