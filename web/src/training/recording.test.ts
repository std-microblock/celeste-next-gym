import { describe, expect, it } from "vitest";
import { makeEmptyButtons, PLAYGROUND } from "../model";
import { createTrainingProject } from "./editorProject";
import {
  applyTutorialRecording,
  hasRecordedAction,
  nextSequentialModuleAtPlayer,
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
      ["dash"],
      ["jump"],
    ]);
    expect(module.tutorial.teaching.steps).toHaveLength(2);
    expect(module.tutorial.fuzz.observe_until).toBe(3);
    expect(module.tutorial.fuzz.success).toContain("!final.dead");
    expect(module.validation.initial_state.pos).toEqual({ x: 111, y: 222 });
    expect(project.training.modules[0].tutorial.fuzz.inputs).not.toEqual(
      module.tutorial.fuzz.inputs,
    );
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
});
