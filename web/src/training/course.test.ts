import { describe, expect, it } from "vitest";
import { snapshot } from "./helpers.ts";
import {
  allModulesCompleted,
  average,
  formatObjectiveOutput,
  moduleAtPlayer,
  objectiveOutputName,
  outputAccuracy,
  triggerContainsPlayer,
} from "./course.ts";
import type { TrainingMapDocument } from "./catalog.ts";
import type { GymMap } from "../model.ts";

const trigger = {
  kind: "training_trigger" as const,
  name: "lesson",
  bounds: { x: 40, y: 200, width: 40, height: 40 },
  direction: { x: 0, y: 0 },
};

describe("map-driven training helpers", () => {
  it("detects an invisible trigger against the player collider", () => {
    expect(triggerContainsPlayer(trigger, snapshot({ x: 40, y: 220 }))).toBe(
      true,
    );
    expect(triggerContainsPlayer(trigger, snapshot({ x: 84.01, y: 220 }))).toBe(
      false,
    );
    expect(triggerContainsPlayer(trigger, snapshot({ x: 60, y: 199 }))).toBe(
      false,
    );
  });

  it("scores actual output against the Fuzz-best output", () => {
    expect(outputAccuracy(325, 325)).toBe(100);
    expect(outputAccuracy(300, 325)).toBeCloseTo(92.3077);
    expect(outputAccuracy(0, 325)).toBe(0);
    expect(outputAccuracy(0, 0)).toBe(100);
    expect(average([100, 80])).toBe(90);
    expect(objectiveOutputName("final.speed.x")).toBe("水平速度");
    expect(formatObjectiveOutput("final.speed.x", 325)).toBe("325 px/s");
  });

  it("selects only unfinished modules and unlocks the finish after all modules", () => {
    const training = {
      modules: [
        { id: "first", trigger: { id: "lesson" } },
        {
          id: "second",
          trigger: { id: "second-trigger" },
        },
      ],
    } as unknown as TrainingMapDocument;
    const map = {
      entities: [
        trigger,
        {
          ...trigger,
          name: "second-trigger",
          bounds: { x: 100, y: 200, width: 40, height: 40 },
        },
      ],
    } as GymMap;
    const player = snapshot({ x: 60, y: 220 });
    expect(moduleAtPlayer(training, map, player, new Set())?.id).toBe("first");
    expect(
      moduleAtPlayer(training, map, player, new Set(["first"])),
    ).toBeUndefined();
    expect(allModulesCompleted(training, new Set(["first"]))).toBe(false);
    expect(allModulesCompleted(training, new Set(["first", "second"]))).toBe(
      true,
    );
  });
});
