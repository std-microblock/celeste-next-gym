import {
  ACTIONS,
  ACTION_LABELS,
  createInitialState,
  makeEmptyButtons,
  type Action,
  type FrameButtons,
  type SimState,
} from "../model";
import type { TrainingInput } from "./session";
import type { TrainingProject } from "./editorProject";
import { triggerContainsPlayer } from "./course";

const DIRECTION_ACTIONS = new Set<Action>(["up", "down", "left", "right"]);
export const RECORDED_ACTIONS = ACTIONS.filter(
  (action) => !DIRECTION_ACTIONS.has(action),
);

function heldFrames(
  frames: readonly FrameButtons[],
  start: number,
  action: Action,
): number {
  let end = start;
  while (end < frames.length && frames[end][action]) end += 1;
  return Math.max(1, end - start);
}

function inputId(keys: readonly Action[], occurrence: number): string {
  const base = keys.join("-");
  return occurrence === 1 ? base : `${base}-${occurrence}`;
}

/** Converts raw controller frames into fixed Fuzz inputs, ignoring WASD by default. */
export function recordedInputsFromFrames(
  frames: readonly FrameButtons[],
): TrainingInput[] {
  const inputs: TrainingInput[] = [];
  const occurrences = new Map<string, number>();
  let previous = makeEmptyButtons();
  for (const [frame, current] of frames.entries()) {
    const keys = RECORDED_ACTIONS.filter(
      (action) => current[action] && !previous[action],
    );
    if (keys.length) {
      const base = keys.join("-");
      const occurrence = (occurrences.get(base) ?? 0) + 1;
      occurrences.set(base, occurrence);
      const input: TrainingInput = {
        id: inputId(keys, occurrence),
        keys,
        at: frame,
        verify: true,
      };
      if (keys.length === 1 && (keys[0] === "jump" || keys[0] === "grab")) {
        const duration = heldFrames(frames, frame, keys[0]);
        if (duration > 1) input.held_time = duration;
      }
      inputs.push(input);
    }
    previous = current;
  }
  return inputs;
}

function recordedDirectionInputsFromFrames(
  frames: readonly FrameButtons[],
): TrainingInput[] {
  const inputs: TrainingInput[] = [];
  for (const action of DIRECTION_ACTIONS) {
    let occurrence = 0;
    for (let frame = 0; frame < frames.length; frame += 1) {
      if (!frames[frame][action] || (frame > 0 && frames[frame - 1][action]))
        continue;
      occurrence += 1;
      inputs.push({
        id: `hold-${action}${occurrence === 1 ? "" : `-${occurrence}`}`,
        keys: [action],
        at: frame,
        held_time: heldFrames(frames, frame, action),
        verify: false,
      });
    }
  }
  return inputs.sort((left, right) => Number(left.at) - Number(right.at));
}

function actionText(keys: readonly string[]): string {
  return keys
    .map((key) => ACTION_LABELS[key as Action] ?? key)
    .join(" + ");
}

function endRegionSuccess(
  bounds: TrainingProject["training"]["modules"][number]["end_trigger"]["bounds"],
): string[] {
  return [
    "!final.dead",
    `final.pos.x >= ${bounds.x - 4}`,
    `final.pos.x <= ${bounds.x + bounds.width + 4}`,
    `final.pos.y >= ${bounds.y}`,
    `final.pos.y <= ${bounds.y + bounds.height + 11}`,
  ];
}

/** Applies one completed play-through to the editable tutorial JSON document. */
export function applyTutorialRecording(
  project: TrainingProject,
  moduleIndex: number,
  initialState: SimState,
  frames: readonly FrameButtons[],
): TrainingProject {
  const inputs = recordedInputsFromFrames(frames);
  if (!inputs.length) throw new Error("录制中没有检测到非方向动作");
  const next = structuredClone(project);
  const module = next.training.modules[moduleIndex];
  if (!module) throw new Error(`教程模块 ${moduleIndex + 1} 不存在`);
  const entry = inputs[0];
  const entryText = actionText(entry.keys);

  module.tutorial.entry.input_id = entry.id;
  module.tutorial.entry.hint = `开始区已激活：按 ${entryText} 开始。`;
  module.tutorial.entry.check = ["!current.dead"];
  module.tutorial.entry.failure = {
    title: `需要 ${entryText}`,
    body: `进入开始区后，第一个教程动作应为 ${entryText}。`,
  };
  module.tutorial.teaching.steps = inputs.map((input) => {
    const text = actionText(input.keys);
    const hold =
      typeof input.held_time === "number" && input.held_time > 1
        ? `并保持 ${input.held_time} 帧`
        : "";
    return {
      prompt: `按 ${text}${hold}。`,
      order_error: {
        title: "动作顺序不正确",
        body: `这里需要 ${text}。`,
      },
      window_error: {
        title: "错过输入窗口",
        body: `请在录制的 F${input.at} 附近输入 ${text}。`,
      },
    };
  });
  module.tutorial.fuzz.inputs = [
    ...recordedDirectionInputsFromFrames(frames),
    ...inputs,
  ].sort((left, right) => Number(left.at) - Number(right.at));
  module.tutorial.fuzz.variables = [];
  module.tutorial.fuzz.observe_until = Math.max(1, frames.length);
  module.tutorial.fuzz.success = endRegionSuccess(module.end_trigger.bounds);
  module.tutorial.fuzz.objectives = [
    {
      type: "approach",
      expression: "final.pos.x",
      target: module.end_trigger.bounds.x + module.end_trigger.bounds.width / 2,
    },
    {
      type: "approach",
      expression: "final.pos.y",
      target: module.end_trigger.bounds.y + module.end_trigger.bounds.height,
    },
  ];
  module.tutorial.fuzz.search.bindings = {};
  module.validation.initial_state = structuredClone(initialState);
  if (module.validation.fuzz)
    module.validation.fuzz = structuredClone(module.tutorial.fuzz);
  return next;
}

export function hasRecordedAction(
  current: FrameButtons,
  previous: FrameButtons,
): boolean {
  return RECORDED_ACTIONS.some(
    (action) => current[action] && !previous[action],
  );
}

/** Record-all follows document order even when training regions overlap. */
export function nextSequentialModuleAtPlayer(
  project: Pick<TrainingProject, "training">,
  state: SimState,
  completed: ReadonlySet<number>,
): number | null {
  const nextIndex = project.training.modules.findIndex(
    (_, index) => !completed.has(index),
  );
  if (nextIndex < 0) return null;
  return triggerContainsPlayer(
    project.training.modules[nextIndex].trigger,
    state,
  )
    ? nextIndex
    : null;
}

/** Current-region recording needs no manually maintained validation position. */
export function recordingStartState(
  project: TrainingProject,
  moduleIndex: number,
): SimState {
  const module = project.training.modules[moduleIndex];
  if (!module) return createInitialState(project.map);
  const existing = structuredClone(module.validation.initial_state);
  if (triggerContainsPlayer(module.trigger, existing)) return existing;
  const bounds = module.trigger.bounds;
  const state = createInitialState(project.map);
  state.pos = {
    x: bounds.x + bounds.width / 2,
    y: bounds.y + bounds.height,
  };
  state.facing =
    module.end_trigger.bounds.x + module.end_trigger.bounds.width / 2 >=
    state.pos.x;
  return state;
}
