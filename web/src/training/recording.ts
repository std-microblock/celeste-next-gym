import {
  ACTIONS,
  ACTION_LABELS,
  createInitialState,
  makeEmptyButtons,
  type Action,
  type FrameButtons,
  type SimState,
} from "../model";
import type {
  TrainingCheckpoint,
  TrainingInput,
  TrainingObjective,
} from "./session";
import type { TrainingProject } from "./editorProject";
import { triggerContainsPlayer } from "./course";

const DIRECTION_ACTIONS = ["up", "down", "left", "right"] as const;
const DIRECTION_ACTION_SET = new Set<Action>(DIRECTION_ACTIONS);
export const RECORDED_ACTIONS = ACTIONS.filter(
  (action) => !DIRECTION_ACTION_SET.has(action),
);

export interface RecordedDirectionPlan {
  inputs: TrainingInput[];
  variables: Array<{
    name: string;
    range: { from: number; to: number };
  }>;
  /** Keeps the recorded tail length when the final direction change is fuzzed. */
  observeUntil: number | string;
  changes: Array<{ frame: number; at: string }>;
}

export type RecordingTargetKind =
  | "speed_x"
  | "speed_y"
  | "speed_total"
  | "dashes"
  | "coordinate_crossing"
  | "stamina";

export type RecordingTargetMode =
  | "match_and_maximize"
  | "match"
  | "maximize";

export const RECORDING_TARGET_MODE_OPTIONS: ReadonlyArray<{
  id: RecordingTargetMode;
  label: string;
}> = [
  { id: "match_and_maximize", label: "达到并最大化" },
  { id: "match", label: "必须达到" },
  { id: "maximize", label: "仅最大化" },
];

export function defaultRecordingTargetMode(
  kind: RecordingTargetKind,
): RecordingTargetMode {
  if (kind === "coordinate_crossing") return "match";
  if (kind === "dashes" || kind === "stamina") return "maximize";
  return "match_and_maximize";
}

export const RECORDING_TARGET_GROUPS: ReadonlyArray<{
  id: string;
  label: string;
  options: ReadonlyArray<{ id: RecordingTargetKind; label: string }>;
}> = [
  {
    id: "speed",
    label: "速度",
    options: [
      { id: "speed_x", label: "X 速度" },
      { id: "speed_y", label: "Y 速度" },
      { id: "speed_total", label: "总速度" },
    ],
  },
  {
    id: "resources",
    label: "资源",
    options: [
      { id: "dashes", label: "冲刺" },
      { id: "stamina", label: "体力" },
    ],
  },
  {
    id: "position",
    label: "位置",
    options: [{ id: "coordinate_crossing", label: "坐标越过" }],
  },
];

export interface RecordedCriticalNode {
  id: string;
  frame: number;
  at: number | string;
  label: string;
}

interface RecordedEvent {
  frame: number;
  actions: Action[];
  directions: Action[];
  previousDirections: Action[];
  directionChanged: boolean;
  label: string;
}

export type RecordingTargetSelections = Record<
  string,
  Partial<Record<RecordingTargetKind, RecordingTargetMode>>
>;

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

function directionKeys(frame: FrameButtons): Action[] {
  return DIRECTION_ACTIONS.filter((action) => frame[action]);
}

function sameDirections(left: readonly Action[], right: readonly Action[]) {
  return (
    left.length === right.length &&
    left.every((direction, index) => direction === right[index])
  );
}

function recordedEvents(frames: readonly FrameButtons[]): RecordedEvent[] {
  const events: RecordedEvent[] = [];
  let previous = makeEmptyButtons();
  let previousDirections: Action[] = [];
  for (const [frame, current] of frames.entries()) {
    const actions = RECORDED_ACTIONS.filter(
      (action) => current[action] && !previous[action],
    );
    const directions = directionKeys(current);
    const directionChanged =
      frame > 0 && !sameDirections(previousDirections, directions);
    if (actions.length || directionChanged) {
      const labels: string[] = [];
      if (actions.length) labels.push(`按 ${actionText(actions)}`);
      if (directionChanged)
        labels.push(
          directions.length
            ? `方向切换为 ${actionText(directions)}`
            : "松开方向",
        );
      else if (frame === 0 && directions.length)
        labels.push(`保持 ${actionText(directions)}`);
      events.push({
        frame,
        actions,
        directions,
        previousDirections,
        directionChanged,
        label: `F${frame} · ${labels.join("；")}`,
      });
    }
    previous = current;
    previousDirections = directions;
  }
  return events;
}

function recordedTimingPlan(frames: readonly FrameButtons[]) {
  const events = recordedEvents(frames);
  const fuzzed = events.filter((event) => event.frame > 0);
  const occurrences = new Map<string, number>();
  let directionOccurrence = 0;
  const named = fuzzed.map((event) => {
    let base: string;
    if (event.actions.length) base = `${event.actions.join("_")}_frame`;
    else {
      directionOccurrence += 1;
      base = `direction_change_${directionOccurrence}`;
    }
    const occurrence = (occurrences.get(base) ?? 0) + 1;
    occurrences.set(base, occurrence);
    return {
      event,
      name: occurrence === 1 ? base : `${base}_${occurrence}`,
    };
  });
  const valuesPerVariable = named.length
    ? Math.max(1, Math.floor(750_000 ** (1 / named.length)))
    : 1;
  const radius = Math.min(6, Math.floor((valuesPerVariable - 1) / 2));
  const variables = named.map(({ event, name }, index) => {
    const previous = named[index - 1]?.event.frame;
    const following = named[index + 1]?.event.frame;
    const closesDirection =
      event.directionChanged && event.previousDirections.length > 0;
    const orderFrom =
      previous === undefined
        ? closesDirection
          ? 1
          : 0
        : Math.floor((previous + event.frame) / 2) + 1;
    const orderTo =
      following === undefined
        ? Number.POSITIVE_INFINITY
        : Math.floor((event.frame + following) / 2);
    return {
      name,
      range: {
        from: Math.max(orderFrom, event.frame - radius),
        to: Math.min(orderTo, event.frame + radius),
      },
    };
  });
  const atByFrame = new Map<number, number | string>([[0, 0]]);
  for (const [index, { event }] of named.entries())
    atByFrame.set(event.frame, variables[index].name);
  const last = named.at(-1);
  return {
    events,
    variables,
    atByFrame,
    observeUntil:
      last === undefined
        ? Math.max(1, frames.length)
        : `max(${Math.max(1, frames.length)}, ${last.name} + ${Math.max(1, frames.length - last.event.frame)})`,
  };
}

/**
 * Records WASD as direction-combination segments. A segment that reaches the
 * end region is infinite. Every event after F0 gets an ordered Fuzz variable;
 * a same-frame action and direction transition deliberately share it.
 */
export function recordedDirectionPlanFromFrames(
  frames: readonly FrameButtons[],
): RecordedDirectionPlan {
  if (!frames.length)
    return { inputs: [], variables: [], observeUntil: 1, changes: [] };

  const timing = recordedTimingPlan(frames);
  const transitions: number[] = [];
  const segments: Array<{
    keys: Action[];
    start: number;
    end?: number;
  }> = [];
  let current = directionKeys(frames[0]);
  let start = 0;
  for (let frame = 1; frame < frames.length; frame += 1) {
    const next = directionKeys(frames[frame]);
    if (sameDirections(current, next)) continue;
    transitions.push(frame);
    if (current.length) segments.push({ keys: current, start, end: frame });
    current = next;
    start = frame;
  }
  if (current.length) segments.push({ keys: current, start });

  const occurrences = new Map<string, number>();
  const inputs = segments.map((segment) => {
    const base = segment.keys.join("-");
    const occurrence = (occurrences.get(base) ?? 0) + 1;
    occurrences.set(base, occurrence);
    const startAt = timing.atByFrame.get(segment.start)!;
    const endAt =
      segment.end === undefined
        ? undefined
        : timing.atByFrame.get(segment.end)!;
    return {
      id: occurrence === 1 ? `hold-${base}` : `hold-${base}-${occurrence}`,
      keys: segment.keys,
      at: startAt,
      held_time:
        endAt === undefined
          ? "hold::inf"
          : startAt === 0
            ? endAt
            : `${endAt} - ${startAt}`,
      verify: false,
    } satisfies TrainingInput;
  });
  return {
    inputs,
    variables: timing.variables,
    observeUntil: timing.observeUntil,
    changes: transitions.map((frame) => ({
      frame,
      at: String(timing.atByFrame.get(frame)),
    })),
  };
}

/** Every action edge and direction transition becomes one editor review node. */
export function recordedCriticalNodesFromFrames(
  frames: readonly FrameButtons[],
): RecordedCriticalNode[] {
  const timing = recordedTimingPlan(frames);
  return timing.events.map((event) => ({
    id: `recorded-node-${event.frame}`,
    frame: event.frame,
    at: timing.atByFrame.get(event.frame)!,
    label: event.label,
  }));
}

function conciseNumber(value: number): string {
  return Number(value.toFixed(2)).toString();
}

function measuredTarget(
  expression: string,
  value: number,
  name: string,
  unit: string,
  mode: RecordingTargetMode,
): {
  objective: TrainingObjective;
  success?: string[];
  description: string;
} {
  const target = Number(value.toFixed(2));
  const recorded = `${conciseNumber(target)}${unit}`;
  const objective: TrainingObjective =
    mode === "match"
      ? { type: "approach", expression, target }
      : { type: "maximize", expression };
  if (mode === "maximize") {
    return {
      objective,
      description: name,
    };
  }
  return {
    objective,
    success: [
      `${expression} >= ${conciseNumber(target - 0.01)}`,
      `${expression} <= ${conciseNumber(target + 0.01)}`,
    ],
    description: `${name} ${recorded}`,
  };
}

function targetObjective(
  kind: RecordingTargetKind,
  state: SimState,
  initial: SimState,
  mode: RecordingTargetMode,
): {
  objective?: TrainingObjective;
  success?: string[];
  description: string;
} {
  switch (kind) {
    case "speed_x":
      return measuredTarget(
        "after.speed.x",
        state.speed.x,
        "X 速度",
        " px/s",
        mode,
      );
    case "speed_y":
      return measuredTarget(
        "after.speed.y",
        state.speed.y,
        "Y 速度",
        " px/s",
        mode,
      );
    case "speed_total": {
      const speed = Math.hypot(state.speed.x, state.speed.y);
      return measuredTarget(
        "sqrt(after.speed.x * after.speed.x + after.speed.y * after.speed.y)",
        speed,
        "总速度",
        " px/s",
        mode,
      );
    }
    case "dashes":
      return measuredTarget(
        "after.dashes",
        state.dashes,
        "剩余冲刺",
        "",
        mode,
      );
    case "coordinate_crossing": {
      const useX =
        Math.abs(state.pos.x - initial.pos.x) >=
        Math.abs(state.pos.y - initial.pos.y);
      const axis = useX ? "x" : "y";
      const target = state.pos[axis];
      const operator = target < initial.pos[axis] ? "<=" : ">=";
      return {
        success: [
          `after.pos.${axis} ${operator} ${conciseNumber(target)}`,
        ],
        description: `${axis.toUpperCase()} ${operator === ">=" ? "≥" : "≤"} ${conciseNumber(target)}`,
      };
    }
    case "stamina":
      return measuredTarget(
        "after.stamina",
        state.stamina,
        "体力",
        "",
        mode,
      );
  }
}

export function recordingTargetCondition(
  kind: RecordingTargetKind,
  state: SimState,
  initial: SimState,
): string {
  switch (kind) {
    case "speed_x":
      return `${conciseNumber(state.speed.x)} px/s`;
    case "speed_y":
      return `${conciseNumber(state.speed.y)} px/s`;
    case "speed_total":
      return `${conciseNumber(Math.hypot(state.speed.x, state.speed.y))} px/s`;
    case "dashes":
      return `${state.dashes}`;
    case "coordinate_crossing": {
      const useX =
        Math.abs(state.pos.x - initial.pos.x) >=
        Math.abs(state.pos.y - initial.pos.y);
      const axis = useX ? "x" : "y";
      const operator = state.pos[axis] < initial.pos[axis] ? "≤" : "≥";
      return `${axis.toUpperCase()} ${operator} ${conciseNumber(state.pos[axis])}`;
    }
    case "stamina":
      return conciseNumber(state.stamina);
  }
}

export function recordingCheckpoints(
  initial: SimState,
  frames: readonly FrameButtons[],
  snapshots: readonly SimState[],
  selections: RecordingTargetSelections,
): TrainingCheckpoint[] {
  return recordedCriticalNodesFromFrames(frames).flatMap((node) => {
    const selected = Object.entries(selections[node.id] ?? {}) as Array<
      [RecordingTargetKind, RecordingTargetMode]
    >;
    if (!selected.length) return [];
    const state = snapshots[node.frame + 1] ?? snapshots.at(-1) ?? initial;
    const targets = selected.map(([kind, mode]) =>
      targetObjective(kind, state, initial, mode),
    );
    const success = targets.flatMap((target) =>
      target.success === undefined ? [] : target.success,
    );
    return [
      {
        id: node.id,
        at: node.at,
        description: targets.map((target) => target.description).join("，"),
        ...(success.length ? { success } : {}),
        objectives: targets.flatMap((target) =>
          target.objective === undefined ? [] : [target.objective],
        ),
      },
    ];
  });
}

function actionText(keys: readonly string[]): string {
  return keys.map((key) => ACTION_LABELS[key as Action] ?? key).join(" + ");
}

/** Applies one completed play-through to the editable tutorial JSON document. */
export function applyTutorialRecording(
  project: TrainingProject,
  moduleIndex: number,
  initialState: SimState,
  frames: readonly FrameButtons[],
  snapshots: readonly SimState[] = [initialState],
  targetSelections: RecordingTargetSelections = {},
): TrainingProject {
  const nodes = recordedCriticalNodesFromFrames(frames);
  const atByRecordedFrame = new Map(nodes.map((node) => [node.frame, node.at]));
  const inputs = recordedInputsFromFrames(frames).map((input) => ({
    ...input,
    at: atByRecordedFrame.get(Number(input.at)) ?? input.at,
  }));
  if (!inputs.length) throw new Error("录制中没有检测到非方向动作");
  const next = structuredClone(project);
  const module = next.training.modules[moduleIndex];
  if (!module) throw new Error(`教程模块 ${moduleIndex + 1} 不存在`);
  const entry = inputs[0];
  const entryText = actionText(entry.keys);
  const checkpoints = recordingCheckpoints(
    initialState,
    frames,
    snapshots,
    targetSelections,
  );
  const descriptionAt = new Map(
    checkpoints.map((checkpoint) => [checkpoint.at, checkpoint.description]),
  );

  module.tutorial.entry.input_id = entry.id;
  module.tutorial.entry.hint = `按 ${entryText} 开始。`;
  module.tutorial.entry.check = ["!current.dead"];
  module.tutorial.entry.failure = {
    title: `需要 ${entryText}`,
    body: `先按 ${entryText}。`,
  };
  module.tutorial.teaching.steps = inputs.map((input) => {
    const text = actionText(input.keys);
    const hold =
      typeof input.held_time === "number" && input.held_time > 1
        ? `并保持 ${input.held_time} 帧`
        : "";
    const target = descriptionAt.get(input.at);
    return {
      prompt: target
        ? `按 ${text}${hold}；${target}。`
        : `按 ${text}${hold}。`,
      order_error: {
        title: "动作顺序不正确",
        body: `按 ${text}。`,
      },
      window_error: {
        title: "错过输入窗口",
        body: `稍早或稍晚按 ${text}。`,
      },
    };
  });
  module.tutorial.summary = `依次完成 ${inputs
    .map((input) => actionText(input.keys))
    .join("、")}。`;
  const directionPlan = recordedDirectionPlanFromFrames(frames);
  module.tutorial.fuzz.inputs = [...directionPlan.inputs, ...inputs];
  module.tutorial.fuzz.variables = directionPlan.variables;
  module.tutorial.fuzz.observe_until = directionPlan.observeUntil;
  module.tutorial.fuzz.success = [];
  module.tutorial.fuzz.checkpoints = checkpoints;
  module.tutorial.fuzz.objectives = [];
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
