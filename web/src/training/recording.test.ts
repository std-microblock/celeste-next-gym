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
  recordedCriticalNodesFromFrames,
  recordedDirectionPlanFromFrames,
  recordedInputsFromFrames,
  recordingCheckpoints,
  type RecordingTargetSelections,
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

  it("writes inputs and initial state without implicit global targets", () => {
    const project = createTrainingProject(PLAYGROUND);
    const initial = structuredClone(
      project.training.modules[0].validation.initial_state,
    );
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
        held_time: "jump_frame",
        verify: false,
      },
      {
        id: "hold-right",
        keys: ["right"],
        at: "jump_frame",
        held_time: "hold::inf",
        verify: false,
      },
    ]);
    expect(module.tutorial.fuzz.variables).toEqual([
      {
        name: "jump_frame",
        range: { from: 1, to: 8 },
      },
    ]);
    expect(module.tutorial.teaching.steps).toHaveLength(2);
    expect(module.tutorial.fuzz.observe_until).toBe("max(3, jump_frame + 1)");
    expect(module.tutorial.fuzz.inputs.slice(2)).toEqual([
      { id: "dash", keys: ["dash"], at: 0, verify: true },
      {
        id: "jump",
        keys: ["jump"],
        at: "jump_frame",
        verify: true,
      },
    ]);
    expect(module.tutorial.fuzz.success).toEqual([]);
    expect(module.tutorial.fuzz.objectives).toEqual([]);
    expect(module.tutorial.fuzz.checkpoints).toEqual([]);
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
      variables: [{ name: "jump_frame", range: { from: 0, to: 8 } }],
      observeUntil: "max(3, jump_frame + 1)",
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

  it("defines every later action frame as a bounded fuzz variable", () => {
    const frames = [buttons("dash")];
    for (let frame = 1; frame <= 20; frame += 1)
      frames.push(frame % 2 ? buttons("jump") : buttons());
    const project = applyTutorialRecording(
      createTrainingProject(PLAYGROUND),
      0,
      createTrainingProject(PLAYGROUND).training.modules[0].validation
        .initial_state,
      frames,
    );
    const fuzz = project.training.modules[0].tutorial.fuzz;
    expect(fuzz.variables).toHaveLength(10);
    expect(fuzz.inputs[0]).toEqual({
      id: "dash",
      keys: ["dash"],
      at: 0,
      verify: true,
    });
    expect(
      fuzz.inputs.slice(1).every((input) => typeof input.at === "string"),
    ).toBe(true);
    const candidateCount = fuzz.variables.reduce(
      (count, variable) =>
        count * (Number(variable.range.to) - Number(variable.range.from) + 1),
      1,
    );
    expect(candidateCount).toBeLessThanOrEqual(750_000);
  });

  it("builds multi-select objectives and descriptions at every critical node", () => {
    const project = createTrainingProject(PLAYGROUND);
    const initial = structuredClone(
      project.training.modules[0].validation.initial_state,
    );
    const frames = [
      buttons("right", "dash"),
      buttons("right"),
      buttons("jump"),
      buttons("left"),
    ];
    const snapshots = Array.from({ length: 5 }, () => structuredClone(initial));
    snapshots[1].speed = { x: 240, y: -60 };
    snapshots[1].pos = { x: initial.pos.x + 12, y: initial.pos.y };
    snapshots[1].dashes = 0;
    snapshots[1].stamina = 87.5;
    snapshots[4].speed.y = 105;
    const nodes = recordedCriticalNodesFromFrames(frames);
    expect(nodes).toEqual([
      {
        id: "recorded-node-0",
        frame: 0,
        at: 0,
        label: "F0 · 按 冲刺；保持 右",
      },
      {
        id: "recorded-node-2",
        frame: 2,
        at: "jump_frame",
        label: "F2 · 按 跳跃；松开方向",
      },
      {
        id: "recorded-node-3",
        frame: 3,
        at: "direction_change_1",
        label: "F3 · 方向切换为 左",
      },
    ]);
    const selections: RecordingTargetSelections = {
      "recorded-node-0": [
        "speed_x",
        "speed_total",
        "dashes",
        "coordinate_crossing",
        "stamina",
      ],
      "recorded-node-3": ["speed_y"],
    };
    const checkpoints = recordingCheckpoints(
      initial,
      frames,
      snapshots,
      selections,
    );
    expect(checkpoints).toHaveLength(2);
    expect(checkpoints[0].objectives.map((item) => item.expression)).toEqual([
      "after.speed.x",
      "sqrt(after.speed.x * after.speed.x + after.speed.y * after.speed.y)",
      "after.dashes",
      "after.stamina",
    ]);
    expect(
      checkpoints[0].objectives.every((item) => item.type === "maximize"),
    ).toBe(true);
    expect(checkpoints[0].success).toEqual([
      `after.pos.x >= ${initial.pos.x + 12}`,
    ]);
    expect(checkpoints[0].description).toBe(
      `X 速度，总速度，剩余冲刺，X ≥ ${initial.pos.x + 12}，体力`,
    );
    expect(checkpoints[1].description).toBe("Y 速度");
    expect(checkpoints[1].at).toBe("direction_change_1");

    const next = applyTutorialRecording(
      project,
      0,
      initial,
      frames,
      snapshots,
      selections,
    );
    expect(next.training.modules[0].tutorial.fuzz.checkpoints).toEqual(
      checkpoints,
    );
    expect(next.training.modules[0].tutorial.fuzz.success).toEqual([]);
    expect(next.training.modules[0].tutorial.fuzz.objectives).toEqual([]);
    const tutorial = next.training.modules[0].tutorial;
    expect(tutorial.summary).toBe("依次完成 冲刺、跳跃。");
    expect(tutorial.teaching.steps[0].prompt).toBe(
      `按 冲刺；X 速度，总速度，剩余冲刺，X ≥ ${initial.pos.x + 12}，体力。`,
    );
    const generatedDescriptions = [
      tutorial.summary,
      tutorial.entry.hint,
      tutorial.entry.failure.title,
      tutorial.entry.failure.body,
      ...tutorial.teaching.steps.flatMap((step) => [
        step.prompt,
        step.order_error.title,
        step.order_error.body,
        step.window_error.title,
        step.window_error.body,
      ]),
      ...(tutorial.fuzz.checkpoints ?? []).map(
        (checkpoint) => checkpoint.description,
      ),
    ].join("\n");
    expect(generatedDescriptions).not.toContain("录制");
    expect(generatedDescriptions).not.toContain("最大化");
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
    expect(
      nextSequentialModuleAtPlayer(project, atSecond, new Set()),
    ).toBeNull();
    expect(nextSequentialModuleAtPlayer(project, atSecond, new Set([0]))).toBe(
      1,
    );
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
