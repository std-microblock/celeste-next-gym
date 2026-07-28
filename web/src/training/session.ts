import type { FrameButtons, SimState } from '../model'

export type TrainingPhase = 'pre_fuzz' | 'fuzz' | 'failed' | 'success'
export type TrainingFailure = 'entry_check_failed' | 'input_order_mismatch' | 'timing_window_miss'

export interface TrainingInput {
  id: string
  keys: string[]
  at: number | string
  verify?: boolean
}

export interface TrainingCandidateInput {
  input_index: number
  frame: number
  keys: string[]
}

export interface TrainingCandidate {
  bindings: Record<string, number>
  verified_inputs: TrainingCandidateInput[]
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

const ACTION_KEYS = ['up', 'down', 'left', 'right', 'jump', 'dash', 'grab'] as const

export function createTrainingSession(candidates: TrainingCandidate[]): TrainingSession {
  return {
    phase: 'pre_fuzz',
    entryFrame: null,
    nextVerifiedInput: 0,
    candidates: [...candidates],
    allCandidates: [...candidates],
    actualInputs: [],
  }
}

export function keySemantics(buttons: FrameButtons): string[] {
  return ACTION_KEYS.filter((key) => buttons[key])
}

export function sameKeySemantics(actual: readonly string[], expected: readonly string[]): boolean {
  const normalizedActual = [...new Set(actual)].sort()
  const normalizedExpected = [...new Set(expected)].sort()
  return normalizedActual.length === normalizedExpected.length
    && normalizedActual.every((key, index) => key === normalizedExpected[index])
}

export function verifiedInputs(definition: TrainingDefinition): TrainingInput[] {
  return definition.fuzz.inputs.filter((input) => input.verify !== false)
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

function failed(session: TrainingSession, kind: TrainingFailure, frame: number): TrainingSession {
  return {
    ...session,
    phase: 'failed',
    failure: { kind, frame, expectedWindow: candidateWindow(session.candidates, session.nextVerifiedInput) },
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
  const input = inputs[session.nextVerifiedInput]
  if (!input) return { ...session, phase: 'success' }
  if (!sameKeySemantics(keys, input.keys)) {
    return session.phase === 'pre_fuzz'
      ? failed(session, 'entry_check_failed', frame)
      : failed(session, 'input_order_mismatch', frame)
  }
  if (session.phase === 'pre_fuzz' && !entryCheckPassed) return failed(session, 'entry_check_failed', frame)

  const matching = session.candidates.filter((candidate) => {
    const expected = candidateInput(candidate, session.nextVerifiedInput)
    return expected !== undefined && expected.frame === frame && sameKeySemantics(keys, expected.keys)
  })
  if (matching.length === 0) return failed(session, session.phase === 'pre_fuzz' ? 'entry_check_failed' : 'timing_window_miss', frame)

  const next = {
    ...session,
    phase: 'fuzz' as TrainingPhase,
    entryFrame: session.phase === 'pre_fuzz' ? frame : session.entryFrame,
    nextVerifiedInput: session.nextVerifiedInput + 1,
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
  let session = createTrainingSession(candidates)
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
