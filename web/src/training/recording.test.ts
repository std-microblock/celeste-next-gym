import { describe, expect, it } from "vitest";
import { makeEmptyButtons, PLAYGROUND } from "../model";
import {
  createTrainingProject,
  validateTrainingProject,
} from "./editorProject";
import {
  applyTutorialRecording,
  hasRecordedAction,
  nextSequentialModuleAtPlayer,
  recordingStartState,
  recordedDirectionPlanFromFrames,
  recordedInputsFromFrames,
} from "./recording";

function buttons(...pressed: Array<keyof ReturnType<typeof makeEmptyButtons>>) {
  const result = makeEmptyButtons();
  for (const action of pressed) result[action] = true;
  return result;
}

describe("tutorial editor recording", () => {
  it("ignores WASD and records non-direction actions relative to F0", () => {
    const frames = [
      buttons("right", "dash"),
      buttons("right"),
      buttons("right", "jump"),
      buttons("right", "jump"),
      buttons("right"),
    ];
    expect(recordedInputsFromFrames(frames)).toEqual([
      { id: "dash", keys: ["dash"], at: 0, verify: true },
      {
        id: "jump",
        keys: ["jump"],
        at: 2,
        verify: true,
        held_time: 2,
      },
    ]);
    expect(hasRecordedAction(buttons("right"), buttons())).toBe(false);
    expect(hasRecordedAction(buttons("dash"), buttons())).toBe(true);
  });

  it("writes inputs, teaching steps, success region and initial state", () => {
    const project = createTrainingProject(PLAYGROUND);
    const initial = structuredClone(project.training.modules[0].validation.initial_state);
    initial.pos = { x: 111, y: 222 };
    const next = applyTutorialRecording(project, 0, initial, [
      buttons("down", "right", "dash"),
      buttons("down", "right"),
      buttons("right", "jump"),
    ]);
    const module = next.training.modules[0];
    expect(module.tutorial.entry.input_id).toBe("dash");
    expect(module.tutorial.fuzz.inputs.map((input) => input.keys)).toEqual([
      ["down", "right"],
      ["right"],
      ["dash"],
      ["jump"],
    ]);
    expect(module.tutorial.fuzz.inputs.slice(0, 2)).toEqual([
      {
        id: "hold-down-right",
        keys: ["down", "right"],
        at: 0,
        held_time: "direction_change_1",
        verify: false,
      },
      {
        id: "hold-right",
        keys: ["right"],
        at: "direction_change_1",
        held_time: "hold::inf",
        verify: false,
      },
    ]);
    expect(module.tutorial.fuzz.variables).toEqual([
      {
        name: "direction_change_1",
        range: { from: 1, to: 8 },
      },
    ]);
    expect(module.tutorial.teaching.steps).toHaveLength(2);
    expect(module.tutorial.fuzz.observe_until).toBe(
      "max(3, direction_change_1 + 1)",
    );
    expect(module.tutorial.fuzz.success).toContain("!final.dead");
    expect(module.tutorial.fuzz.objectives).toEqual([
      {
        type: "approach",
        expression: "final.pos.x",
        target:
          module.end_trigger.bounds.x + module.end_trigger.bounds.width / 2,
      },
      {
        type: "approach",
        expression: "final.pos.y",
        target:
          module.end_trigger.bounds.y + module.end_trigger.bounds.height,
      },
    ]);
    expect(module.validation.initial_state.pos).toEqual({ x: 111, y: 222 });
    expect(validateTrainingProject(next)).toEqual([]);
    expect(project.training.modules[0].tutorial.fuzz.inputs).not.toEqual(
      module.tutorial.fuzz.inputs,
    );
  });

  it("keeps unchanged WASD at F0 forever", () => {
    expect(
      recordedDirectionPlanFromFrames([
        buttons("down", "right", "dash"),
        buttons("down", "right"),
        buttons("down", "right", "jump"),
      ]),
    ).toEqual({
      inputs: [
        {
          id: "hold-down-right",
          keys: ["down", "right"],
          at: 0,
          held_time: "hold::inf",
          verify: false,
        },
      ],
      variables: [],
      observeUntil: 3,
      changes: [],
    });
  });

  it("shares one fuzzed time between direction release and replacement", () => {
    const plan = recordedDirectionPlanFromFrames([
      buttons("right", "dash"),
      buttons("right"),
      buttons(),
      buttons("left"),
      buttons("left"),
    ]);
    expect(plan.inputs).toEqual([
      {
        id: "hold-right",
        keys: ["right"],
        at: 0,
        held_time: "direction_change_1",
        verify: false,
      },
      {
        id: "hold-left",
        keys: ["left"],
        at: "direction_change_2",
        held_time: "hold::inf",
        verify: false,
      },
    ]);
    expect(plan.variables).toEqual([
      {
        name: "direction_change_1",
        range: { from: 1, to: 2 },
      },
      {
        name: "direction_change_2",
        range: { from: 3, to: 9 },
      },
    ]);
    expect(plan.changes).toEqual([
      { frame: 2, at: "direction_change_1" },
      { frame: 3, at: "direction_change_2" },
    ]);
  });

  it("arms record-all modules in document order", () => {
    const project = createTrainingProject(PLAYGROUND);
    const second = structuredClone(project.training.modules[0]);
    second.id = "lesson-2";
    second.trigger.id = "lesson-2-start";
    second.end_trigger.id = "lesson-2-end";
    second.trigger.bounds = { x: 400, y: 430, width: 100, height: 80 };
    project.training.modules.push(second);
    const atSecond = structuredClone(second.validation.initial_state);
    atSecond.pos = { x: 450, y: 496 };
    expect(nextSequentialModuleAtPlayer(project, atSecond, new Set())).toBeNull();
    expect(nextSequentialModuleAtPlayer(project, atSecond, new Set([0]))).toBe(1);
  });

  it("places current-region recording inside a moved start region", () => {
    const project = createTrainingProject(PLAYGROUND);
    const module = project.training.modules[0];
    module.trigger.bounds = { x: 400, y: 400, width: 80, height: 80 };
    module.end_trigger.bounds = { x: 200, y: 400, width: 40, height: 80 };
    const state = recordingStartState(project, 0);
    expect(state.pos).toEqual({ x: 440, y: 480 });
    expect(state.facing).toBe(false);
  });
});
