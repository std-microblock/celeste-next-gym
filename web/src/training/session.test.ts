import { describe, expect, it } from "vitest";
import { makeEmptyButtons } from "../model";
import {
  assistedWindowBrake,
  candidateOperationObjectivePoints,
  candidateObjectivePoints,
  createTrainingSession,
  currentTrainingInput,
  expectedTrainingInputTriggered,
  matchingTrainingCandidate,
  trainingEntryContextPassed,
  trainingEntryInput,
  trainingReferenceButtons,
  trainingReferenceEndFrame,
  trainingReferenceSteps,
  referenceStepBrake,
  trainingVerificationTriggered,
  verificationKeys,
  verifyTrainingInput,
  type TrainingCandidate,
  type TrainingDefinition,
} from "./session";

const definition: TrainingDefinition = {
  id: "jump-entry",
  title: "Jump entry",
  entry: { input_id: "jump_entry", hint: "Jump" },
  fuzz: {
    inputs: [
      { id: "unused_dash", keys: ["dash"], at: -2 },
      { id: "hold_right", keys: ["right"], at: 0, verify: false },
      { id: "jump_entry", keys: ["jump"], at: 0 },
      { id: "grab_later", keys: ["grab"], at: 4 },
    ],
  },
};

const candidates: TrainingCandidate[] = [
  {
    bindings: {},
    objective_values: [325],
    successful: true,
    verified_inputs: [
      { input_index: 0, frame: -2, keys: ["dash"] },
      { input_index: 2, frame: 0, keys: ["jump"] },
      { input_index: 3, frame: 4, keys: ["grab"] },
    ],
  },
];

describe("training-defined entry input", () => {
  it("starts from entry.input_id instead of the first verified or Dash input", () => {
    const initial = createTrainingSession(candidates, definition);
    expect(trainingEntryInput(definition)).toMatchObject({
      id: "jump_entry",
      fuzzInputIndex: 2,
    });
    expect(currentTrainingInput(initial, definition)?.id).toBe("jump_entry");

    const entered = verifyTrainingInput(initial, definition, 0, ["jump"]);
    expect(entered.phase).toBe("fuzz");
    expect(currentTrainingInput(entered, definition)?.id).toBe("grab_later");
    expect(verifyTrainingInput(entered, definition, 4, ["grab"]).phase).toBe(
      "success",
    );
  });

  it("triggers the declared entry action and declared direction combinations", () => {
    const empty = makeEmptyButtons();
    expect(
      trainingVerificationTriggered(
        { ...empty, jump: true },
        empty,
        trainingEntryInput(definition),
      ),
    ).toBe(true);
    expect(
      trainingVerificationTriggered(
        { ...empty, right: true },
        empty,
        trainingEntryInput(definition),
      ),
    ).toBe(false);

    const directionInput = { id: "diagonal", keys: ["right", "down"], at: 0 };
    expect(
      trainingVerificationTriggered(
        { ...empty, right: true, down: true },
        { ...empty, right: true },
        directionInput,
      ),
    ).toBe(true);
  });

  it("checks verify:false entry holds from the same training definition", () => {
    const empty = makeEmptyButtons();
    const correct = { ...empty, right: true, jump: true };
    expect(trainingEntryContextPassed(correct, definition)).toBe(true);
    expect(
      trainingEntryContextPassed({ ...correct, right: false }, definition),
    ).toBe(false);
    expect(
      trainingEntryContextPassed({ ...correct, down: true }, definition),
    ).toBe(false);
    expect(
      verificationKeys(
        correct,
        { ...empty, right: true },
        trainingEntryInput(definition),
      ),
    ).toEqual(["jump"]);
  });

  it("rejects an entry.input_id that is absent or verify:false", () => {
    const invalid = {
      ...definition,
      entry: { ...definition.entry, input_id: "hold_right" },
    };
    expect(trainingEntryInput(invalid)).toBeUndefined();
    expect(() =>
      verifyTrainingInput(
        createTrainingSession(candidates, invalid),
        invalid,
        0,
        ["right"],
      ),
    ).toThrow(/entry\.input_id/);
  });

  it("exposes the objective values returned for each candidate frame", () => {
    const alternatives: TrainingCandidate[] = [
      {
        ...candidates[0],
        objective_values: [325],
        verified_inputs: candidates[0].verified_inputs.map((input) =>
          input.input_index === 3 ? { ...input, frame: 6 } : input,
        ),
      },
      { ...candidates[0], objective_values: [300] },
    ];
    expect(candidateObjectivePoints(alternatives, 3)).toEqual([
      { frame: 4, values: [300], successful: true },
      { frame: 6, values: [325], successful: true },
    ]);
    expect(
      matchingTrainingCandidate(alternatives, definition, [
        { frame: 0, keys: ["jump"] },
        { frame: 4, keys: ["grab"] },
      ])?.objective_values,
    ).toEqual([300]);
  });

  it("plots current-operation output by operation frame with prior inputs fixed", () => {
    const currentInputIndex = 3;
    const matchingSuccess = {
      ...candidates[0],
      objective_values: [300],
    };
    const evaluations: TrainingCandidate[] = [
      {
        ...candidates[0],
        objective_values: [999],
        successful: false,
        verified_inputs: candidates[0].verified_inputs.map((input) =>
          input.input_index === 2 ? { ...input, frame: 1 } : input,
        ),
      },
      {
        ...candidates[0],
        objective_values: [100],
        successful: false,
      },
      matchingSuccess,
      {
        ...candidates[0],
        objective_values: [250],
        successful: false,
        verified_inputs: candidates[0].verified_inputs.map((input) =>
          input.input_index === currentInputIndex
            ? { ...input, frame: 5 }
            : input,
        ),
      },
    ];

    expect(
      candidateOperationObjectivePoints(
        [matchingSuccess],
        evaluations,
        definition,
        currentInputIndex,
        [{ frame: 0, keys: ["jump"] }],
      ),
    ).toEqual([
      { frame: 4, values: [300], successful: true },
      { frame: 5, values: [250], successful: false },
    ]);
  });

  it("does not release an assisted pause for an unrelated action", () => {
    const empty = makeEmptyButtons();
    const expected = trainingEntryInput(definition);
    expect(
      expectedTrainingInputTriggered(
        { ...empty, dash: true },
        empty,
        expected,
      ),
    ).toBe(false);
    expect(
      expectedTrainingInputTriggered(
        { ...empty, jump: true },
        empty,
        expected,
      ),
    ).toBe(true);
  });

  it("replays the best candidate with its held and momentary controls", () => {
    expect(trainingReferenceButtons(candidates[0], definition, 0)).toMatchObject({
      right: true,
      jump: true,
      grab: false,
    });
    expect(trainingReferenceButtons(candidates[0], definition, 1)).toMatchObject({
      right: false,
      jump: false,
      grab: false,
    });
    expect(trainingReferenceButtons(candidates[0], definition, 4)).toMatchObject({
      right: false,
      jump: false,
      grab: true,
    });
    expect(trainingReferenceEndFrame(candidates[0], definition)).toBe(28);
    expect(trainingReferenceSteps(candidates[0], definition)).toEqual([
      { inputIndex: 2, inputId: "jump_entry", frame: 0, keys: ["jump"] },
      { inputIndex: 3, inputId: "grab_later", frame: 4, keys: ["grab"] },
    ]);
  });

  it("smoothly stops a demonstration on its next input frame", () => {
    expect(referenceStepBrake(9, 20).multiplier).toBe(1);
    expect(referenceStepBrake(10, 20)).toMatchObject({
      multiplier: 1,
      braking: true,
      stopped: false,
    });
    expect(referenceStepBrake(15, 20).multiplier).toBeCloseTo(0.5);
    expect(referenceStepBrake(20, 20)).toMatchObject({
      multiplier: 0,
      braking: true,
      stopped: true,
    });
  });

  it("smoothly brakes without pausing at frame 17 of a 20-frame window", () => {
    const windowCandidates = Array.from({ length: 20 }, (_, frame) => ({
      ...candidates[0],
      verified_inputs: candidates[0].verified_inputs.map((input) =>
        input.input_index === 3 ? { ...input, frame } : input,
      ),
    }));
    expect(assistedWindowBrake(windowCandidates, 3, 9).multiplier).toBe(1);
    expect(assistedWindowBrake(windowCandidates, 3, 10)).toMatchObject({
      multiplier: 1,
      braking: true,
      stopped: false,
      stopFrame: 17,
    });
    const middle = assistedWindowBrake(windowCandidates, 3, 14).multiplier;
    expect(middle).toBeGreaterThan(0);
    expect(middle).toBeLessThan(1);
    expect(assistedWindowBrake(windowCandidates, 3, 17)).toMatchObject({
      multiplier: 0,
      braking: true,
      stopped: true,
      stopFrame: 17,
    });
  });
});
