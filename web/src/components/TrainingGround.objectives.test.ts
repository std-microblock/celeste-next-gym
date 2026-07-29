import { describe, expect, it } from "vitest";
import { PLAYGROUND } from "../model";
import { createTrainingProject } from "../training/editorProject";
import {
  trainingRetryTarget,
  tutorialObjectivesForInput,
} from "./TrainingGround";

describe("training timeline checkpoint objectives", () => {
  it("uses only the current step objectives with their Rust result indices", () => {
    const tutorial =
      createTrainingProject(PLAYGROUND).training.modules[0].tutorial;
    tutorial.fuzz.inputs = [
      { id: "hold-right", keys: ["right"], at: 0, verify: false },
      { id: "dash", keys: ["dash"], at: 0, verify: true },
      { id: "jump", keys: ["jump"], at: "jump_frame", verify: true },
    ];
    tutorial.fuzz.checkpoints = [
      {
        id: "dash-target",
        at: 0,
        description: "dash",
        objectives: [{ type: "maximize", expression: "after.speed.x" }],
      },
      {
        id: "jump-target",
        at: "jump_frame",
        description: "jump",
        objectives: [
          { type: "maximize", expression: "after.speed.y" },
          {
            type: "maximize",
            expression:
              "sqrt(after.speed.x * after.speed.x + after.speed.y * after.speed.y)",
          },
        ],
      },
    ];
    tutorial.fuzz.objectives = [
      { type: "maximize", expression: "final.stamina" },
    ];

    expect(tutorialObjectivesForInput(tutorial, 1)).toEqual([
      {
        objective: { type: "maximize", expression: "after.speed.x" },
        resultIndex: 0,
      },
      {
        objective: { type: "maximize", expression: "final.stamina" },
        resultIndex: 3,
      },
    ]);
    expect(tutorialObjectivesForInput(tutorial, 2)).toEqual([
      {
        objective: { type: "maximize", expression: "after.speed.y" },
        resultIndex: 1,
      },
      {
        objective: {
          type: "maximize",
          expression:
            "sqrt(after.speed.x * after.speed.x + after.speed.y * after.speed.y)",
        },
        resultIndex: 2,
      },
      {
        objective: { type: "maximize", expression: "final.stamina" },
        resultIndex: 3,
      },
    ]);
  });

  it("lets R retry the current or previous module start", () => {
    const completed = [
      { moduleId: "lesson-1", triggerFrame: 12 },
      { moduleId: "lesson-2", triggerFrame: 48 },
    ];
    expect(trainingRetryTarget("current", 80, "lesson-3", completed)).toEqual(
      { frame: 80, moduleId: "lesson-3" },
    );
    expect(trainingRetryTarget("previous", 80, "lesson-3", completed)).toEqual(
      { frame: 48, moduleId: "lesson-2" },
    );
    expect(trainingRetryTarget("previous", 48, "lesson-2", completed)).toEqual(
      { frame: 12, moduleId: "lesson-1" },
    );
    expect(trainingRetryTarget("previous", 20, "lesson-1", [])).toEqual({
      frame: 20,
      moduleId: "lesson-1",
    });
  });
});
