import { describe, expect, it } from "vitest";
import { PLAYGROUND } from "../model";
import { createTrainingProject } from "../training/editorProject";
import { tutorialObjectivesForInput } from "./TrainingGround";

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
});
