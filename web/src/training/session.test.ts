import { describe, expect, it } from 'vitest'
import { makeEmptyButtons } from '../model'
import {
  createTrainingSession,
  currentTrainingInput,
  trainingEntryContextPassed,
  trainingEntryInput,
  trainingVerificationTriggered,
  verificationKeys,
  verifyTrainingInput,
  type TrainingCandidate,
  type TrainingDefinition,
} from './session'

const definition: TrainingDefinition = {
  id: 'jump-entry',
  title: 'Jump entry',
  entry: { input_id: 'jump_entry', hint: 'Jump' },
  fuzz: { inputs: [
    { id: 'unused_dash', keys: ['dash'], at: -2 },
    { id: 'hold_right', keys: ['right'], at: 0, verify: false },
    { id: 'jump_entry', keys: ['jump'], at: 0 },
    { id: 'grab_later', keys: ['grab'], at: 4 },
  ] },
}

const candidates: TrainingCandidate[] = [{
  bindings: {},
  verified_inputs: [
    { input_index: 0, frame: -2, keys: ['dash'] },
    { input_index: 2, frame: 0, keys: ['jump'] },
    { input_index: 3, frame: 4, keys: ['grab'] },
  ],
}]

describe('training-defined entry input', () => {
  it('starts from entry.input_id instead of the first verified or Dash input', () => {
    const initial = createTrainingSession(candidates, definition)
    expect(trainingEntryInput(definition)).toMatchObject({ id: 'jump_entry', fuzzInputIndex: 2 })
    expect(currentTrainingInput(initial, definition)?.id).toBe('jump_entry')

    const entered = verifyTrainingInput(initial, definition, 0, ['jump'])
    expect(entered.phase).toBe('fuzz')
    expect(currentTrainingInput(entered, definition)?.id).toBe('grab_later')
    expect(verifyTrainingInput(entered, definition, 4, ['grab']).phase).toBe('success')
  })

  it('triggers the declared entry action and declared direction combinations', () => {
    const empty = makeEmptyButtons()
    expect(trainingVerificationTriggered({ ...empty, jump: true }, empty, trainingEntryInput(definition))).toBe(true)
    expect(trainingVerificationTriggered({ ...empty, right: true }, empty, trainingEntryInput(definition))).toBe(false)

    const directionInput = { id: 'diagonal', keys: ['right', 'down'], at: 0 }
    expect(trainingVerificationTriggered(
      { ...empty, right: true, down: true },
      { ...empty, right: true },
      directionInput,
    )).toBe(true)
  })

  it('checks verify:false entry holds from the same training definition', () => {
    const empty = makeEmptyButtons()
    const correct = { ...empty, right: true, jump: true }
    expect(trainingEntryContextPassed(correct, definition)).toBe(true)
    expect(trainingEntryContextPassed({ ...correct, right: false }, definition)).toBe(false)
    expect(trainingEntryContextPassed({ ...correct, down: true }, definition)).toBe(false)
    expect(verificationKeys(correct, { ...empty, right: true }, trainingEntryInput(definition))).toEqual(['jump'])
  })

  it('rejects an entry.input_id that is absent or verify:false', () => {
    const invalid = { ...definition, entry: { ...definition.entry, input_id: 'hold_right' } }
    expect(trainingEntryInput(invalid)).toBeUndefined()
    expect(() => verifyTrainingInput(createTrainingSession(candidates, invalid), invalid, 0, ['right'])).toThrow(/entry\.input_id/)
  })
})
