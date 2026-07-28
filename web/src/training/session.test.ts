import { describe, expect, it } from 'vitest'
import { assistedRate, candidateWindow, createTrainingSession, rebuildTrainingSession, verifyTrainingInput, type TrainingCandidate, type TrainingDefinition } from './session'

const definition: TrainingDefinition = {
  id: 'hyper-basic', title: 'Hyper', entry: { input_id: 'diagonal_dash', hint: '按住右下并冲刺。' },
  fuzz: { inputs: [
    { id: 'hold_diagonal', keys: ['right', 'down'], at: 0, verify: false },
    { id: 'diagonal_dash', keys: ['dash'], at: 0 },
    { id: 'jump', keys: ['jump'], at: 'jump_frame' },
  ] },
}

const candidates: TrainingCandidate[] = [
  { bindings: { jump_frame: 5 }, verified_inputs: [{ input_index: 1, frame: 0, keys: ['dash'] }, { input_index: 2, frame: 5, keys: ['jump'] }] },
  { bindings: { jump_frame: 6 }, verified_inputs: [{ input_index: 1, frame: 0, keys: ['dash'] }, { input_index: 2, frame: 6, keys: ['jump'] }] },
  { bindings: { jump_frame: 8 }, verified_inputs: [{ input_index: 1, frame: 0, keys: ['dash'] }, { input_index: 2, frame: 8, keys: ['jump'] }] },
]

describe('training session', () => {
  it('uses verify:false holds without adding a teaching step', () => {
    expect(candidateWindow(candidates, 2)).toEqual([{ from: 5, to: 6 }, { from: 8, to: 8 }])
    const entered = verifyTrainingInput(createTrainingSession(candidates), definition, 0, ['dash'])
    const jumped = verifyTrainingInput(entered, definition, 6, ['jump'])
    expect(jumped.phase).toBe('success')
    expect(jumped.candidates.map((candidate) => candidate.bindings.jump_frame)).toEqual([6])
  })

  it('distinguishes key-order and timing failures', () => {
    const entered = verifyTrainingInput(createTrainingSession(candidates), definition, 0, ['dash'])
    expect(verifyTrainingInput(entered, definition, 4, ['dash']).failure?.kind).toBe('input_order_mismatch')
    expect(verifyTrainingInput(entered, definition, 7, ['jump']).failure).toMatchObject({ kind: 'timing_window_miss', expectedWindow: [{ from: 5, to: 6 }, { from: 8, to: 8 }] })
  })

  it('rebuilds filtering deterministically after a timeline seek', () => {
    const rebuilt = rebuildTrainingSession(definition, candidates, [
      { frame: 0, keys: ['dash'] },
      { frame: 5, keys: ['jump'] },
    ])
    expect(rebuilt.phase).toBe('success')
    expect(rebuilt.candidates).toHaveLength(1)
  })

  it('uses the specified linear slowdown curve', () => {
    expect(assistedRate(1, 0, 12, 12, .35)).toBe(1)
    expect(assistedRate(1, 6, 12, 12, .35)).toBeCloseTo(.675)
    expect(assistedRate(1, 12, 12, 12, .35)).toBeCloseTo(.35)
  })
})
