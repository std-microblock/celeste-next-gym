import { ACTIONS, type Action, type FrameButtons, type SimState } from '../model.ts'

export type TrainingPhase = 'pre_fuzz' | 'fuzz' | 'failed' | 'success'
export type TrainingFailure = 'entry_check_failed' | 'input_order_mismatch' | 'timing_window_miss'

export interface TrainingInput {
  id: string
  keys: string[]
  at: number | string
  verify?: boolean
  held_time?: number | string
  before_input?: string | string[]
  after_input?: string | string[]
}

export interface TrainingCandidateInput {
  input_index: number
  frame: number
  keys: string[]
}

export interface TrainingCandidate {
  bindings: Record<string, number>
  verified_inputs: TrainingCandidateInput[]
  final_state?: { speed?: { x: number; y: number } }
}

export interface TrainingDefinition {
  id: string
  title: string
  entry: { input_id: string; hint: string }
  fuzz: { inputs: TrainingInput[] }
}

export interface TrainingSession {
  phase: TrainingPhase
  entryFrame: number | null
  nextVerifiedInput: number
  candidates: TrainingCandidate[]
  allCandidates: TrainingCandidate[]
  actualInputs: Array<{ frame: number; keys: string[] }>
  failure?: { kind: TrainingFailure; frame: number; expectedWindow: FrameWindow[] }
}

export interface FrameWindow { from: number; to: number }

export type IndexedTrainingInput = TrainingInput & { fuzzInputIndex: number }

const DIRECTION_KEYS = ['up', 'down', 'left', 'right'] as const
const PRESS_KEYS = ACTIONS.filter((key) => !DIRECTION_KEYS.includes(key as typeof DIRECTION_KEYS[number]))

function isAction(key: string): key is Action {
  return ACTIONS.includes(key as Action)
}

export function createTrainingSession(candidates: TrainingCandidate[], definition?: TrainingDefinition): TrainingSession {
  const entryIndex = definition === undefined
    ? 0
    : verifiedInputs(definition).findIndex((input) => input.id === definition.entry.input_id)
  return {
    phase: 'pre_fuzz',
    entryFrame: null,
    nextVerifiedInput: Math.max(0, entryIndex),
    candidates: [...candidates],
    allCandidates: [...candidates],
    actualInputs: [],
  }
}

export function keySemantics(buttons: FrameButtons): string[] {
  return ACTIONS.filter((key) => buttons[key])
}

export function sameKeySemantics(actual: readonly string[], expected: readonly string[]): boolean {
  const normalizedActual = [...new Set(actual)].sort()
  const normalizedExpected = [...new Set(expected)].sort()
  return normalizedActual.length === normalizedExpected.length
    && normalizedActual.every((key, index) => key === normalizedExpected[index])
}

export function verifiedInputs(definition: TrainingDefinition): IndexedTrainingInput[] {
  return definition.fuzz.inputs.flatMap((input, fuzzInputIndex) => input.verify === false ? [] : [{ ...input, fuzzInputIndex }])
}

export function trainingEntryInput(definition: TrainingDefinition): IndexedTrainingInput | undefined {
  return verifiedInputs(definition).find((input) => input.id === definition.entry.input_id)
}

export function currentTrainingInput(session: TrainingSession, definition: TrainingDefinition): IndexedTrainingInput | undefined {
  return session.phase === 'pre_fuzz'
    ? trainingEntryInput(definition)
    : verifiedInputs(definition)[session.nextVerifiedInput]
}

/** True when the expected combination has just become active, or another action was attempted. */
export function trainingVerificationTriggered(current: FrameButtons, previous: FrameButtons, input: TrainingInput | undefined): boolean {
  if (!input) return false
  const expected = input.keys.filter(isAction)
  const expectedTriggered = expected.length > 0
    && expected.every((key) => current[key])
    && expected.some((key) => !previous[key])
  const actionTriggered = PRESS_KEYS.some((key) => current[key] && !previous[key])
  return expectedTriggered || actionTriggered
}

/** Checks author-defined F0 holds while keeping directional matching exact. */
export function trainingEntryContextPassed(buttons: FrameButtons, definition: TrainingDefinition): boolean {
  const entry = trainingEntryInput(definition)
  if (!entry) return false
  const heldKeys = definition.fuzz.inputs
    .filter((input) => input.verify === false && input.at === entry.at)
    .flatMap((input) => input.keys)
    .filter(isAction)
  const expectedDirections = new Set([...heldKeys, ...entry.keys].filter((key): key is typeof DIRECTION_KEYS[number] => DIRECTION_KEYS.includes(key as typeof DIRECTION_KEYS[number])))
  return DIRECTION_KEYS.every((direction) => buttons[direction] === expectedDirections.has(direction))
    && heldKeys.filter((key) => !DIRECTION_KEYS.includes(key as typeof DIRECTION_KEYS[number])).every((key) => buttons[key])
}

export function verificationKeys(buttons: FrameButtons, previous: FrameButtons, input: TrainingInput | undefined): string[] {
  const expected = input?.keys ?? []
  const actual = keySemantics(buttons)
  const newlyPressed = actual.filter((key) => !previous[key as Action])
  // With verify:false direction holds, only the new action belongs to the
  // teaching input. A definition that includes a direction in verify:true
  // deliberately opts into strict directional matching.
  return expected.some((key) => DIRECTION_KEYS.includes(key as typeof DIRECTION_KEYS[number]))
    ? actual.filter((key) => DIRECTION_KEYS.includes(key as typeof DIRECTION_KEYS[number]) || newlyPressed.includes(key))
    : newlyPressed.filter((key) => !DIRECTION_KEYS.includes(key as typeof DIRECTION_KEYS[number]))
}

function candidateInput(candidate: TrainingCandidate, inputIndex: number): TrainingCandidateInput | undefined {
  return candidate.verified_inputs.find((input) => input.input_index === inputIndex)
}

export function candidateWindow(candidates: readonly TrainingCandidate[], inputIndex: number): FrameWindow[] {
  const frames = [...new Set(candidates
    .map((candidate) => candidateInput(candidate, inputIndex)?.frame)
    .filter((frame): frame is number => frame !== undefined))].sort((left, right) => left - right)
  const windows: FrameWindow[] = []
  for (const frame of frames) {
    const current = windows.at(-1)
    if (current && frame <= current.to + 1) current.to = frame
    else windows.push({ from: frame, to: frame })
  }
  return windows
}

export function nextTargetFrame(candidates: readonly TrainingCandidate[], inputIndex: number): number | undefined {
  return candidates
    .map((candidate) => candidateInput(candidate, inputIndex)?.frame)
    .filter((frame): frame is number => frame !== undefined)
    .sort((left, right) => left - right)[0]
}

function failed(session: TrainingSession, kind: TrainingFailure, frame: number, fuzzInputIndex: number): TrainingSession {
  return {
    ...session,
    phase: 'failed',
    failure: { kind, frame, expectedWindow: candidateWindow(session.candidates, fuzzInputIndex) },
  }
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
  if (session.phase === 'failed' || session.phase === 'success') return session
  const inputs = verifiedInputs(definition)
  const input = currentTrainingInput(session, definition)
  if (session.phase === 'pre_fuzz' && !input) throw new Error(`Training ${definition.id} entry.input_id ${definition.entry.input_id} does not name a verified Fuzz input`)
  if (!input) return { ...session, phase: 'success' }
  if (!sameKeySemantics(keys, input.keys)) {
    return session.phase === 'pre_fuzz'
      ? failed(session, 'entry_check_failed', frame, input.fuzzInputIndex)
      : failed(session, 'input_order_mismatch', frame, input.fuzzInputIndex)
  }
  if (session.phase === 'pre_fuzz' && !entryCheckPassed) return failed(session, 'entry_check_failed', frame, input.fuzzInputIndex)

  const matching = session.candidates.filter((candidate) => {
    const expected = candidateInput(candidate, input.fuzzInputIndex)
    return expected !== undefined && expected.frame === frame && sameKeySemantics(keys, expected.keys)
  })
  if (matching.length === 0) return failed(session, session.phase === 'pre_fuzz' ? 'entry_check_failed' : 'timing_window_miss', frame, input.fuzzInputIndex)

  const next = {
    ...session,
    phase: 'fuzz' as TrainingPhase,
    entryFrame: session.phase === 'pre_fuzz' ? frame : session.entryFrame,
    nextVerifiedInput: inputs.findIndex((candidate) => candidate.fuzzInputIndex === input.fuzzInputIndex) + 1,
    candidates: matching,
    actualInputs: [...session.actualInputs, { frame, keys: [...keys] }],
    failure: undefined,
  }
  return next.nextVerifiedInput >= inputs.length ? { ...next, phase: 'success' } : next
}

export function rebuildTrainingSession(
  definition: TrainingDefinition,
  candidates: TrainingCandidate[],
  inputs: readonly { frame: number; keys: readonly string[]; entryCheckPassed?: boolean }[],
): TrainingSession {
  let session = createTrainingSession(candidates, definition)
  for (const input of inputs) {
    session = verifyTrainingInput(session, definition, input.frame, input.keys, input.entryCheckPassed)
    if (session.phase === 'failed') break
  }
  return session
}

export function assistedRate(baseRate: number, fuzzFrame: number, targetFrame: number | undefined, radiusFrames: number, minimumMultiplier: number): number {
  if (targetFrame === undefined || radiusFrames <= 0) return baseRate
  const distance = Math.max(targetFrame - fuzzFrame, 0)
  const progress = Math.max(0, Math.min(1, 1 - distance / radiusFrames))
  return baseRate * (1 - progress * (1 - minimumMultiplier))
}

export function entryCheckSnapshot(state: SimState): SimState {
  // Kept as a named boundary so the UI never evaluates author expressions in
  // JavaScript. The Rust Fuzz bridge owns the Rhai evaluation.
  return state
}
