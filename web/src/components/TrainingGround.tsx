import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
} from "react";
import {
  ACTION_LABELS,
  ACTIONS,
  buttonsToInput,
  makeEmptyButtons,
  type FrameButtons,
  type GymMap,
  type KeyBindings,
  type SimState,
} from "../model";
import { WasmClient } from "../simulator/wasmClient";
import {
  allModulesCompleted,
  average,
  formatObjectiveOutput,
  moduleAtPlayer,
  objectiveOutputName,
  outputAccuracy,
  triggerContainsPlayer,
  type TrainingCompletion,
} from "../training/course";
import {
  assistedWindowBrake,
  candidateOperationObjectivePoints,
  candidateWindow,
  createTrainingSession,
  currentTrainingInput,
  expectedTrainingInputTriggered,
  matchingTrainingCandidate,
  nextTargetFrame,
  rebuildTrainingSession,
  trainingEntryContextPassed,
  trainingEntryInput,
  trainingVerificationTriggered,
  trainingReferenceButtons,
  trainingReferenceSteps,
  referenceStepBrake,
  verificationKeys,
  verifyTrainingInput,
  type TrainingCandidate,
  type TrainingSession,
  type TrainingReferenceStep,
} from "../training/session";
import {
  trainingCatalog,
  type TrainingDocument,
  type TrainingModule,
  type TrainingVariant,
} from "../training/catalog";
import { GameView } from "./GameView";
import { GameplayStrawberry } from "./GameplaySprite";
import {
  TrainingCatalogSidebar,
  TrainingVariantThumbnail,
} from "./TrainingCatalogSidebar";
import { TrainingPrompt } from "./TrainingPrompt";
import {
  TrainingResultTimeline,
  TrainingTimeline,
  type TrainingObjectiveSeries,
} from "./TrainingTimeline";
import type { VisualTheme } from "../visualThemes";

interface Attempt {
  frame: number;
  inputId: string;
  keys: string[];
  entryCheckPassed?: boolean;
  entryAccepted?: boolean;
}
interface PredictionPreview {
  targetFrame?: number;
  windows: Array<{ from: number; to: number }>;
  objectives: TrainingObjectiveSeries[];
  objectiveResultIndices?: number[];
  bestObjectiveValues?: number[];
}
export interface OutcomeAnimation {
  phase: "failed";
  startedAt: number;
  durationMs: number;
  objectiveValues: number[];
  timelineFrame: number;
}

const FAILURE_SLOWDOWN_MS = 1_000;
const GUIDED_RESULT_HOLD_MS = 1_000;
const MOUSE_ACTIVITY_VISIBLE_MS = 1_200;
export type TrainingResetMode = "current" | "start";

export type TrainingLessonStage = "demo" | "assisted" | "free";

const LESSON_STAGES: Array<{ id: TrainingLessonStage; label: string }> = [
  { id: "demo", label: "演示" },
  { id: "assisted", label: "辅助实操" },
  { id: "free", label: "自由练习" },
];

function completionOutputSummary(completion: TrainingCompletion): string {
  if (
    completion.objectives.length === 0 ||
    !Number.isFinite(completion.objectiveValues[0]) ||
    !Number.isFinite(completion.bestObjectiveValues[0])
  )
    return "已通过当前步骤条件";
  const expression = completion.objectives[0]?.expression ?? "objective";
  return `${objectiveOutputName(expression)}：实际 ${formatObjectiveOutput(expression, completion.objectiveValues[0])} / 最佳 ${formatObjectiveOutput(expression, completion.bestObjectiveValues[0])}`;
}

/** Rust returns checkpoint objectives first, followed by final objectives. */
export function tutorialObjectivesForInput(
  tutorial: TrainingDocument,
  inputIndex: number,
) {
  const inputAt = tutorial.fuzz.inputs[inputIndex]?.at;
  const indexed: Array<{
    objective: TrainingDocument["fuzz"]["objectives"][number];
    resultIndex: number;
  }> = [];
  let resultIndex = 0;
  for (const checkpoint of tutorial.fuzz.checkpoints ?? []) {
    for (const objective of checkpoint.objectives) {
      if (checkpoint.at === inputAt) indexed.push({ objective, resultIndex });
      resultIndex += 1;
    }
  }
  for (const objective of tutorial.fuzz.objectives) {
    indexed.push({ objective, resultIndex });
    resultIndex += 1;
  }
  return indexed;
}

function buttonsFromKeyboard(
  keys: ReadonlySet<string>,
  bindings: KeyBindings,
): FrameButtons {
  const buttons = makeEmptyButtons();
  for (const action of ACTIONS) buttons[action] = keys.has(bindings[action]);
  return buttons;
}

export function timingAssessment(
  actualFrame: number | undefined,
  targetFrame: number | undefined,
): string {
  if (actualFrame === undefined || targetFrame === undefined)
    return "无可比较的最佳点";
  const difference = actualFrame - targetFrame;
  if (difference === 0) return "正中最佳点";
  return difference < 0
    ? `早了 ${Math.abs(difference)} 帧`
    : `晚了 ${difference} 帧`;
}

export function trainingInputLocked(
  outcome: OutcomeAnimation | null,
  settlement = false,
): boolean {
  return outcome !== null || settlement;
}

export function timingAssessmentToWindow(
  actualFrame: number | undefined,
  windows: readonly { from: number; to: number }[],
): string {
  if (actualFrame === undefined || windows.length === 0)
    return "无可比较的可行窗口";
  const nearest = windows
    .map((window) => {
      const frame = Math.max(window.from, Math.min(window.to, actualFrame));
      return { frame, distance: Math.abs(actualFrame - frame) };
    })
    .sort((left, right) => left.distance - right.distance)[0];
  if (!nearest || nearest.distance === 0) return "命中可行窗口";
  return actualFrame < nearest.frame
    ? `早于最近窗口 ${nearest.distance} 帧`
    : `晚于最近窗口 ${nearest.distance} 帧`;
}

export function trainingRetryTarget(
  mode: TrainingResetMode,
  resetFrame: number,
  resetModuleId: string | null,
  completed: readonly Pick<
    TrainingCompletion,
    "moduleId" | "triggerFrame"
  >[],
): { frame: number; moduleId: string | null } {
  const current = { frame: resetFrame, moduleId: resetModuleId };
  if (mode === "current") return current;
  void completed;
  return { frame: 0, moduleId: null };
}

/** Map-driven lesson runner: triggers arm tutorial modules; the next action is local F0. */
export function TrainingGround({
  techniqueId,
  variantId,
  bindings,
  theme,
  onSelectTraining,
  variantOverride,
  editorPreview = false,
}: {
  techniqueId: string;
  variantId: string;
  bindings: KeyBindings;
  theme: VisualTheme;
  onSelectTraining(techniqueId: string, variantId: string): void;
  variantOverride?: TrainingVariant;
  editorPreview?: boolean;
}) {
  const client = useMemo(() => new WasmClient(), []);
  const technique =
    trainingCatalog.find((item) => item.id === techniqueId) ??
    trainingCatalog[0];
  const variantIndex = Math.max(
    0,
    technique.variants.findIndex((variant) => variant.id === variantId),
  );
  const selectedVariant =
    variantOverride ??
    technique.variants[variantIndex] ??
    technique.variants[0];
  const [map, setMap] = useState<GymMap | null>(null);
  const [initial, setInitial] = useState<SimState | null>(null);
  const [activeModuleId, setActiveModuleId] = useState<string | null>(null);
  const activeModuleRef = useRef<TrainingModule | null>(null);
  const [candidates, setCandidates] = useState<TrainingCandidate[]>([]);
  const candidatesRef = useRef<TrainingCandidate[]>([]);
  const [evaluations, setEvaluations] = useState<TrainingCandidate[]>([]);
  const evaluationsRef = useRef<TrainingCandidate[]>([]);
  const [session, setSession] = useState<TrainingSession>(() =>
    createTrainingSession([]),
  );
  const sessionRef = useRef(session);
  const [snapshots, setSnapshots] = useState<SimState[]>([]);
  const snapshotsRef = useRef<SimState[]>([]);
  const [frame, setFrame] = useState(0);
  const frameRef = useRef(0);
  const [playing, setPlaying] = useState(false);
  const [baseRate, setBaseRate] = useState(1);
  const [autoSlowdown, setAutoSlowdown] = useState(true);
  const [lessonStage, setLessonStage] = useState<TrainingLessonStage>("free");
  const lessonStageRef = useRef<TrainingLessonStage>("free");
  const [lessonMode, setLessonMode] = useState(false);
  const lessonStartFrameRef = useRef(0);
  const [demoStepIndex, setDemoStepIndex] = useState(0);
  const demoStepIndexRef = useRef(0);
  const demoAdvanceRef = useRef(false);
  const demoInputsFinishedRef = useRef(false);
  const stageInputSuccessRef = useRef(false);
  const pendingCompletionRef = useRef<TrainingCompletion | null>(null);
  const [guidedResultStage, setGuidedResultStage] =
    useState<"demo" | "assisted" | null>(null);
  const guidedResultTimerRef = useRef<ReturnType<typeof setTimeout> | null>(
    null,
  );
  const [mouseActive, setMouseActive] = useState(false);
  const mouseActivityTimerRef = useRef<ReturnType<typeof setTimeout> | null>(
    null,
  );
  const referenceCandidatesRef = useRef<TrainingCandidate[]>([]);
  const referenceEvaluationsRef = useRef<TrainingCandidate[]>([]);
  const referenceModuleIdRef = useRef<string | null>(null);
  const forceAdvanceRef = useRef(false);
  const [timelineOpen, setTimelineOpen] = useState(true);
  const [resetFrame, setResetFrame] = useState(0);
  const [triggerFrame, setTriggerFrame] = useState<number | null>(null);
  const triggerFrameRef = useRef<number | null>(null);
  const [fuzzStartFrame, setFuzzStartFrame] = useState<number | null>(null);
  const fuzzStartRef = useRef<number | null>(null);
  const [prediction, setPrediction] = useState<PredictionPreview>({
    windows: [],
    objectives: [],
  });
  const predictionRef = useRef<PredictionPreview>({
    windows: [],
    objectives: [],
  });
  const predictionDirty = useRef(false);
  const [followReference, setFollowReference] = useState(true);
  const [outcome, setOutcome] = useState<OutcomeAnimation | null>(null);
  const outcomeRef = useRef<OutcomeAnimation | null>(null);
  const [outcomeProgress, setOutcomeProgress] = useState(0);
  const [completions, setCompletions] = useState<TrainingCompletion[]>([]);
  const completionsRef = useRef<TrainingCompletion[]>([]);
  const [settlement, setSettlement] = useState(false);
  const settlementRef = useRef(false);
  const occupiedTriggers = useRef(new Set<string>());
  const [notice, setNotice] = useState("正在加载训练地图…");
  const keys = useRef(new Set<string>());
  const previousButtons = useRef<FrameButtons>(makeEmptyButtons());
  const outcomeButtons = useRef<FrameButtons>(makeEmptyButtons());
  const attempts = useRef<Attempt[]>([]);
  const simulating = useRef(false);
  const simulationEpoch = useRef(0);
  const resetModuleRef = useRef<TrainingModule | null>(null);

  const document = activeModuleRef.current?.tutorial ?? null;

  const applySession = (next: TrainingSession) => {
    sessionRef.current = next;
    setSession(next);
  };
  const applyPrediction = (next: PredictionPreview) => {
    predictionRef.current = next;
    setPrediction(next);
  };
  const clearOutcome = () => {
    outcomeRef.current = null;
    outcomeButtons.current = makeEmptyButtons();
    setOutcome(null);
    setOutcomeProgress(0);
  };
  const clearGuidedResult = () => {
    if (guidedResultTimerRef.current !== null) {
      clearTimeout(guidedResultTimerRef.current);
      guidedResultTimerRef.current = null;
    }
    setGuidedResultStage(null);
  };
  const beginFailure = (
    timelineFrame: number,
    candidate?: TrainingCandidate,
  ) => {
    const next: OutcomeAnimation = {
      phase: "failed",
      startedAt: performance.now(),
      durationMs: FAILURE_SLOWDOWN_MS,
      objectiveValues: (predictionRef.current.objectiveResultIndices ?? []).map(
        (index) => candidate?.objective_values[index] ?? Number.NaN,
      ),
      timelineFrame,
    };
    outcomeRef.current = next;
    outcomeButtons.current = { ...previousButtons.current };
    setOutcome(next);
    setOutcomeProgress(0);
  };
  const setActiveModule = (
    module: TrainingModule | null,
    atFrame: number | null,
    stage: TrainingLessonStage = module ? "free" : lessonStageRef.current,
  ) => {
    clearGuidedResult();
    activeModuleRef.current = module;
    lessonStageRef.current = stage;
    setLessonStage(stage);
    setLessonMode(module !== null && stage !== "free");
    lessonStartFrameRef.current = atFrame ?? frameRef.current;
    demoStepIndexRef.current = 0;
    setDemoStepIndex(0);
    demoAdvanceRef.current = false;
    demoInputsFinishedRef.current = false;
    stageInputSuccessRef.current = false;
    pendingCompletionRef.current = null;
    setActiveModuleId(module?.id ?? null);
    triggerFrameRef.current = atFrame;
    setTriggerFrame(atFrame);
    fuzzStartRef.current = null;
    setFuzzStartFrame(null);
    const cachedCandidates =
      module && referenceModuleIdRef.current === module.id
        && referenceCandidatesRef.current.length > 0
        ? referenceCandidatesRef.current
        : [];
    candidatesRef.current = cachedCandidates;
    evaluationsRef.current =
      module && referenceModuleIdRef.current === module.id
        && referenceEvaluationsRef.current.length > 0
        ? referenceEvaluationsRef.current
        : [];
    setCandidates(cachedCandidates);
    setEvaluations(evaluationsRef.current);
    applySession(createTrainingSession(cachedCandidates, module?.tutorial));
    attempts.current = [];
    applyPrediction({ windows: [], objectives: [] });
    if (module) {
      resetModuleRef.current = module;
      setResetFrame(atFrame ?? frameRef.current);
      setAutoSlowdown(module.tutorial.assist.auto_slowdown.enabled_by_default);
      setNotice(
        stage === "demo"
          ? `${module.tutorial.title}：系统先演示一次完整按法。`
          : stage === "assisted"
            ? `${module.tutorial.title}：轮到你操作，接近窗口末段会渐慢并停在提示点。`
            : `${module.tutorial.title}：自由练习，提示与慢放已关闭。`,
      );
      if (stage === "demo") setPlaying(cachedCandidates.length === 0);
    }
  };
  const activateModule = (module: TrainingModule, atFrame: number) => {
    if (
      activeModuleRef.current ||
      completionsRef.current.some(
        (completion) => completion.moduleId === module.id,
      )
    )
      return;
    setActiveModule(module, atFrame, "free");
  };

  const idsInside = (state: SimState, variant: TrainingVariant) =>
    new Set([
      ...variant.training.modules
        .filter((module) => triggerContainsPlayer(module.trigger, state))
        .map((module) => module.trigger.id),
      ...(triggerContainsPlayer(variant.training.finish.trigger, state)
        ? [variant.training.finish.trigger.id]
        : []),
    ]);

  const scanTriggers = (
    state: SimState,
    atFrame: number,
    completed = completionsRef.current,
  ) => {
    const inside = idsInside(state, selectedVariant);
    const entered = [...inside].filter(
      (id) => !occupiedTriggers.current.has(id),
    );
    occupiedTriggers.current = inside;
    const completedIds = new Set(
      completed.map((completion) => completion.moduleId),
    );
    // If another trigger is crossed while a module is still active, keep it
    // eligible for as long as the player remains inside. This avoids losing a
    // tightly spaced second module merely because the first one completed a
    // frame or two later.
    const module = activeModuleRef.current
      ? undefined
      : selectedVariant.training.modules.find(
          (candidate) =>
            inside.has(candidate.trigger.id) && !completedIds.has(candidate.id),
        );
    if (module) activateModule(module, atFrame);
    if (entered.includes(selectedVariant.training.finish.trigger.id)) {
      const ready =
        !selectedVariant.training.finish.require_all_modules ||
        allModulesCompleted(selectedVariant.training, completedIds);
      if (ready) {
        settlementRef.current = true;
        setSettlement(true);
        setPlaying(false);
        keys.current.clear();
        setNotice("训练地图全部完成。");
      } else {
        const remaining =
          selectedVariant.training.modules.length - completedIds.size;
        setNotice(`终点还未解锁：还有 ${remaining} 个训练模块未完成。`);
      }
    }
  };

  const installVariant = async (variant: TrainingVariant) => {
    simulationEpoch.current += 1;
    setPlaying(false);
    clearOutcome();
    settlementRef.current = false;
    setSettlement(false);
    setNotice(`正在加载 ${technique.title} · ${variant.title}…`);
    for (const module of variant.training.modules) {
      const entry = trainingEntryInput(module.tutorial);
      if (!entry)
        throw new Error(
          `训练模块 ${module.id} 的 entry.input_id 未指向可验证输入`,
        );
      if (entry.at !== 0)
        throw new Error(`训练模块 ${module.id} 的入口输入必须位于本地 F0`);
    }
    setMap(variant.map);
    setInitial(variant.initial);
    snapshotsRef.current = [variant.initial];
    setSnapshots([variant.initial]);
    frameRef.current = 0;
    setFrame(0);
    setResetFrame(0);
    completionsRef.current = [];
    setCompletions([]);
    referenceModuleIdRef.current = null;
    referenceCandidatesRef.current = [];
    referenceEvaluationsRef.current = [];
    previousButtons.current = makeEmptyButtons();
    forceAdvanceRef.current = false;
    keys.current.clear();
    resetModuleRef.current = null;
    setActiveModule(null, null);
    occupiedTriggers.current = idsInside(variant.initial, variant);
    const initialModule = moduleAtPlayer(
      variant.training,
      variant.initial,
      new Set(),
    );
    if (initialModule) activateModule(initialModule, 0);
    setFollowReference(true);
    setNotice(
      initialModule
        ? `${initialModule.tutorial.title} 已触发；下一个动作将作为 F0。`
        : `地图已加载；前往不可见训练触发区。`,
    );
  };

  const capturePrediction = (
    source: TrainingSession,
    start: number | null,
    tutorial: TrainingDocument,
    sourceEvaluations = evaluationsRef.current,
  ) => {
    const inputIndex = currentTrainingInput(source, tutorial)?.fuzzInputIndex;
    if (inputIndex === undefined || start === null) return;
    const indexedObjectives = tutorialObjectivesForInput(tutorial, inputIndex);
    applyPrediction({
      targetFrame: (() => {
        const target = nextTargetFrame(source.candidates, inputIndex);
        return target === undefined ? undefined : start + target;
      })(),
      windows: candidateWindow(source.candidates, inputIndex).map((window) => ({
        from: start + window.from,
        to: start + window.to,
      })),
      objectives: indexedObjectives.map(({ objective, resultIndex }) => ({
        expression: objective.expression,
        points: candidateOperationObjectivePoints(
          source.candidates,
          sourceEvaluations,
          tutorial,
          inputIndex,
          source.actualInputs,
        )
          .map((point) => ({
            frame: start + point.frame,
            value: point.values[resultIndex],
            successful: point.successful,
          }))
          .filter((point) => Number.isFinite(point.value)),
      })),
      objectiveResultIndices: indexedObjectives.map(
        ({ resultIndex }) => resultIndex,
      ),
      bestObjectiveValues: indexedObjectives.map(
        ({ resultIndex }) =>
          source.candidates[0]?.objective_values[resultIndex] ?? Number.NaN,
      ),
    });
  };

  useEffect(() => () => client.dispose(), [client]);
  useEffect(() => {
    const revealTutorialButton = () => {
      setMouseActive(true);
      if (mouseActivityTimerRef.current !== null)
        clearTimeout(mouseActivityTimerRef.current);
      mouseActivityTimerRef.current = setTimeout(() => {
        mouseActivityTimerRef.current = null;
        setMouseActive(false);
      }, MOUSE_ACTIVITY_VISIBLE_MS);
    };
    window.addEventListener("pointermove", revealTutorialButton, {
      passive: true,
    });
    return () => {
      window.removeEventListener("pointermove", revealTutorialButton);
      if (mouseActivityTimerRef.current !== null)
        clearTimeout(mouseActivityTimerRef.current);
      if (guidedResultTimerRef.current !== null)
        clearTimeout(guidedResultTimerRef.current);
    };
  }, []);
  useEffect(() => {
    let active = true;
    void (async () => {
      try {
        await client.ready();
        if (active) await installVariant(selectedVariant);
      } catch (error) {
        if (active)
          setNotice(
            error instanceof Error ? error.message : "训练地图加载失败",
          );
      }
    })();
    return () => {
      active = false;
    };
    // installVariant reads the selected catalog entry for this render.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [client, selectedVariant]);

  const seek = (requested: number) => {
    simulationEpoch.current += 1;
    const next = Math.max(
      0,
      Math.min(snapshotsRef.current.length - 1, Math.round(requested)),
    );
    setPlaying(false);
    clearOutcome();
    predictionDirty.current = true;
    frameRef.current = next;
    setFrame(next);
    const tutorial = activeModuleRef.current?.tutorial;
    const start = fuzzStartRef.current;
    if (
      !tutorial ||
      start === null ||
      next < start ||
      candidatesRef.current.length === 0
    )
      return;
    const replay = attempts.current
      .filter((attempt) => attempt.frame >= start && attempt.frame <= next)
      .map((attempt) => ({ ...attempt, frame: attempt.frame - start }));
    applySession(
      rebuildTrainingSession(tutorial, candidatesRef.current, replay),
    );
  };

  const resetTo = (
    target = resetFrame,
    moduleOverride?: TrainingModule | null,
    stageOverride?: TrainingLessonStage,
    resume = false,
  ) => {
    const snapshot = snapshotsRef.current[target] ?? initial;
    if (!snapshot) return;
    simulationEpoch.current += 1;
    if (!resume) setPlaying(false);
    snapshotsRef.current = snapshotsRef.current.slice(0, target + 1);
    setSnapshots([...snapshotsRef.current]);
    frameRef.current = target;
    setFrame(target);
    previousButtons.current = resume
      ? buttonsFromKeyboard(keys.current, bindings)
      : makeEmptyButtons();
    forceAdvanceRef.current = false;
    if (!resume) keys.current.clear();
    clearOutcome();
    settlementRef.current = false;
    setSettlement(false);
    const module =
      moduleOverride === undefined
        ? (activeModuleRef.current ?? resetModuleRef.current)
        : moduleOverride;
    const keptCompletions = completionsRef.current.filter(
      (completion) =>
        completion.completedFrame <= target &&
        completion.moduleId !== module?.id,
    );
    completionsRef.current = keptCompletions;
    setCompletions(keptCompletions);
    setActiveModule(
      module,
      module ? target : null,
      stageOverride ?? (module ? lessonStageRef.current : lessonStageRef.current),
    );
    occupiedTriggers.current = idsInside(snapshot, selectedVariant);
    setNotice(
      module
        ? `${module.tutorial.title} 已重置；${module.tutorial.entry.hint}`
        : "训练地图已重置；前往不可见训练触发区。",
    );
    if (resume) setPlaying(true);
  };

  const holdGuidedResult = (
    stage: "demo" | "assisted",
    module: TrainingModule,
  ) => {
    if (guidedResultTimerRef.current !== null) return;
    const epoch = simulationEpoch.current;
    const startFrame = lessonStartFrameRef.current;
    setPlaying(false);
    setGuidedResultStage(stage);
    setNotice(
      stage === "demo"
        ? `${module.tutorial.title}：演示完成，停留一秒查看结果。`
        : `${module.tutorial.title}：辅助实操完成，停留一秒查看结果。`,
    );
    guidedResultTimerRef.current = setTimeout(() => {
      guidedResultTimerRef.current = null;
      if (
        simulationEpoch.current !== epoch ||
        activeModuleRef.current?.id !== module.id ||
        lessonStageRef.current !== stage
      )
        return;
      setGuidedResultStage(null);
      resetTo(
        startFrame,
        module,
        stage === "demo" ? "assisted" : "free",
        true,
      );
    }, GUIDED_RESULT_HOLD_MS);
  };

  const retry = () => {
    const resetModule = activeModuleRef.current ?? resetModuleRef.current;
    const target = trainingRetryTarget(
      "current",
      resetFrame,
      resetModule?.id ?? null,
      completionsRef.current,
    );
    const module = target.moduleId
      ? (selectedVariant.training.modules.find(
          (candidate) => candidate.id === target.moduleId,
        ) ?? resetModule)
      : resetModule;
    resetTo(target.frame, module);
  };

  const selectLessonStage = (stage: TrainingLessonStage) => {
    const module = activeModuleRef.current ?? resetModuleRef.current;
    if (!module) return;
    keys.current.clear();
    resetTo(lessonStartFrameRef.current, module, stage);
  };

  const startLesson = () => {
    const module = activeModuleRef.current ?? resetModuleRef.current;
    if (!module) return;
    keys.current.clear();
    resetTo(lessonStartFrameRef.current, module, "demo");
  };

  const moveDemoStep = (direction: -1 | 1) => {
    const module = activeModuleRef.current;
    const candidate = candidatesRef.current[0];
    if (!module || !candidate || lessonStageRef.current !== "demo") return;
    const steps = trainingReferenceSteps(candidate, module.tutorial);
    if (direction > 0) {
      demoAdvanceRef.current = true;
      setPlaying(true);
      return;
    }
    const nextIndex = Math.max(0, demoStepIndexRef.current - 1);
    const step = steps[nextIndex];
    const target = lessonStartFrameRef.current + (step?.frame ?? 0);
    simulationEpoch.current += 1;
    clearOutcome();
    frameRef.current = target;
    setFrame(target);
    demoStepIndexRef.current = nextIndex;
    setDemoStepIndex(nextIndex);
    demoAdvanceRef.current = false;
    demoInputsFinishedRef.current = false;
    previousButtons.current =
      (step?.frame ?? 0) > 0
        ? trainingReferenceButtons(
            candidate,
            module.tutorial,
            step!.frame - 1,
          )
        : makeEmptyButtons();
    setPlaying(false);
  };

  useEffect(() => {
    const down = (event: KeyboardEvent) => {
      if (
        event.target instanceof HTMLElement &&
        event.target.matches("input, select, button")
      )
        return;
      const gameInput = Object.values(bindings).includes(event.code);
      const resetInput = event.code === "KeyR" && !gameInput;
      if (gameInput || resetInput) event.preventDefault();
      if (resetInput && !event.repeat) {
        retry();
        return;
      }
      if (lessonStageRef.current === "demo") return;
      if (trainingInputLocked(outcomeRef.current, settlementRef.current))
        return;
      const previous = buttonsFromKeyboard(keys.current, bindings);
      keys.current.add(event.code);
      if (gameInput) {
        const current = buttonsFromKeyboard(keys.current, bindings);
        const expected = activeModuleRef.current
          ? currentTrainingInput(
              sessionRef.current,
              activeModuleRef.current.tutorial,
            )
          : undefined;
        forceAdvanceRef.current =
          lessonStageRef.current === "assisted"
            ? expectedTrainingInputTriggered(current, previous, expected)
            : true;
        setFollowReference(true);
        if (predictionDirty.current) {
          predictionDirty.current = false;
          applyPrediction({ windows: [], objectives: [] });
        }
        setPlaying(true);
      }
    };
    const up = (event: KeyboardEvent) => {
      if (!trainingInputLocked(outcomeRef.current, settlementRef.current))
        keys.current.delete(event.code);
    };
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
    };
    // resetTo intentionally reads current state through React's render closure.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [resetFrame, initial, bindings, selectedVariant]);

  useEffect(() => {
    if (!playing || !map || !initial || settlement) return;
    let active = true;
    let animation = 0;
    let last = performance.now();
    let carry = 0;
    const tick = (now: number) => {
      const tutorial = activeModuleRef.current?.tutorial;
      const sourceSession = sessionRef.current;
      const stage = lessonStageRef.current;
      const fuzzInputIndex = tutorial
        ? currentTrainingInput(sourceSession, tutorial)?.fuzzInputIndex
        : undefined;
      const fuzzFrame =
        stage === "demo"
          ? Math.max(0, frameRef.current - lessonStartFrameRef.current)
          : fuzzStartRef.current === null
            ? 0
            : Math.max(0, frameRef.current - fuzzStartRef.current);
      const demoSteps =
        stage === "demo" && tutorial && sourceSession.candidates[0]
          ? trainingReferenceSteps(sourceSession.candidates[0], tutorial)
          : [];
      const demoStep = demoSteps[demoStepIndexRef.current];
      const demoBrake =
        stage === "demo" && !demoAdvanceRef.current
          ? referenceStepBrake(fuzzFrame, demoStep?.frame)
          : undefined;
      const brake =
        stage === "assisted" &&
        tutorial &&
        autoSlowdown &&
        fuzzInputIndex !== undefined
          ? assistedWindowBrake(
              sourceSession.candidates,
              fuzzInputIndex,
              fuzzFrame,
            )
          : undefined;
      const assisted =
        stage === "assisted" && autoSlowdown && brake
          ? baseRate * brake.multiplier
          : baseRate;
      const activeOutcome = outcomeRef.current;
      const progress =
        activeOutcome === null
          ? 0
          : Math.max(
              0,
              Math.min(
                1,
                (now - activeOutcome.startedAt) / activeOutcome.durationMs,
              ),
            );
      const rate =
        stage === "demo"
          ? baseRate * (demoBrake?.multiplier ?? 1)
          : assisted * (1 - progress);
      if (activeOutcome !== null) {
        setOutcomeProgress(progress);
        if (progress >= 1) {
          setPlaying(false);
          return;
        }
      } else if (tutorial)
        capturePrediction(sourceSession, fuzzStartRef.current, tutorial);
      carry += ((Math.min(250, now - last) * 60) / 1000) * rate;
      last = now;
      const needsReferenceSearch =
        (stage === "demo" || stage === "assisted") &&
        candidatesRef.current.length === 0;
      if (
        (carry >= 1 || forceAdvanceRef.current || needsReferenceSearch) &&
        !simulating.current
      ) {
        if (carry >= 1) carry -= 1;
        forceAdvanceRef.current = false;
        const currentFrame = frameRef.current;
        let current =
          stage === "demo" && candidatesRef.current[0] && tutorial
            ? trainingReferenceButtons(
                candidatesRef.current[0],
                tutorial,
                currentFrame - lessonStartFrameRef.current,
              )
            : activeOutcome === null
              ? buttonsFromKeyboard(keys.current, bindings)
              : outcomeButtons.current;
        let input = buttonsToInput(current, previousButtons.current);
        const previous = previousButtons.current;
        previousButtons.current = current;
        simulating.current = true;
        const epoch = simulationEpoch.current;
        void (async () => {
          let beforeSession = sessionRef.current;
          const activeModule = activeModuleRef.current;
          const activeTutorial = activeModule?.tutorial;
          if (
            (stage === "demo" || stage === "assisted") &&
            activeModule &&
            activeTutorial &&
            candidatesRef.current.length === 0
          ) {
            const result = await client.fuzzSearch(
              snapshotsRef.current[currentFrame]!,
              JSON.stringify(activeTutorial.fuzz),
              map,
            );
            if (!active || epoch !== simulationEpoch.current) return;
            if (result.candidates.length === 0)
              throw new Error(`${activeTutorial.title} 没有可演示的成功轨迹`);
            referenceModuleIdRef.current = activeModule.id;
            referenceCandidatesRef.current = result.candidates;
            referenceEvaluationsRef.current = result.evaluations;
            candidatesRef.current = result.candidates;
            evaluationsRef.current = result.evaluations;
            setCandidates(result.candidates);
            setEvaluations(result.evaluations);
            applySession(createTrainingSession(result.candidates, activeTutorial));
            if (stage === "assisted") {
              previousButtons.current = buttonsFromKeyboard(
                keys.current,
                bindings,
              );
              setPlaying(false);
              return;
            }
            current = trainingReferenceButtons(
              result.candidates[0],
              activeTutorial,
              currentFrame - lessonStartFrameRef.current,
            );
            input = buttonsToInput(current, previous);
            previousButtons.current = current;
            const steps = trainingReferenceSteps(
              result.candidates[0],
              activeTutorial,
            );
            demoStepIndexRef.current = 0;
            setDemoStepIndex(0);
            if (
              steps[0] &&
              currentFrame - lessonStartFrameRef.current >= steps[0].frame
            ) {
              previousButtons.current =
                steps[0].frame > 0
                  ? trainingReferenceButtons(
                      result.candidates[0],
                      activeTutorial,
                      steps[0].frame - 1,
                    )
                  : makeEmptyButtons();
              setPlaying(false);
              return;
            }
          }
          const expectedInput = activeTutorial
            ? currentTrainingInput(beforeSession, activeTutorial)
            : undefined;
          const shouldVerify = activeTutorial
            ? stage !== "demo" &&
              trainingVerificationTriggered(current, previous, expectedInput)
            : false;
          if (
            shouldVerify &&
            beforeSession.phase === "pre_fuzz" &&
            candidatesRef.current.length === 0
          ) {
            const beforeState = snapshotsRef.current[currentFrame]!;
            const result = await client.fuzzSearch(
              beforeState,
              JSON.stringify(activeTutorial!.fuzz),
              map,
            );
            if (!active || epoch !== simulationEpoch.current) return;
            if (result.candidates.length === 0)
              throw new Error(
                `${activeTutorial!.title} 在当前触发状态下没有成功候选`,
              );
            candidatesRef.current = result.candidates;
            evaluationsRef.current = result.evaluations;
            setCandidates(result.candidates);
            setEvaluations(result.evaluations);
            beforeSession = createTrainingSession(
              result.candidates,
              activeTutorial,
            );
            applySession(beforeSession);
          }
          const trace = await client.simulate(
            snapshotsRef.current[currentFrame]!,
            [input],
            map,
          );
          if (!active || epoch !== simulationEpoch.current) return;
          const after = trace.at(-1);
          if (!after) throw new Error("训练模拟未返回状态");
          const nextFrame = currentFrame + 1;
          snapshotsRef.current = [
            ...snapshotsRef.current.slice(0, nextFrame),
            after,
          ];
          setSnapshots(snapshotsRef.current);
          frameRef.current = nextFrame;
          setFrame(nextFrame);
          if (
            stage === "demo" &&
            activeTutorial &&
            candidatesRef.current[0]
          ) {
            const steps = trainingReferenceSteps(
              candidatesRef.current[0],
              activeTutorial,
            );
            const step = steps[demoStepIndexRef.current];
            const localFrame = currentFrame - lessonStartFrameRef.current;
            if (
              demoAdvanceRef.current &&
              step &&
              localFrame >= step.frame
            ) {
              demoAdvanceRef.current = false;
              const nextIndex = demoStepIndexRef.current + 1;
              demoStepIndexRef.current = nextIndex;
              setDemoStepIndex(nextIndex);
              if (nextIndex >= steps.length)
                demoInputsFinishedRef.current = true;
            }
            const nextStep = steps[demoStepIndexRef.current];
            if (
              nextStep &&
              nextFrame - lessonStartFrameRef.current >= nextStep.frame
            ) {
              previousButtons.current =
                nextStep.frame > 0
                  ? trainingReferenceButtons(
                      candidatesRef.current[0],
                      activeTutorial,
                      nextStep.frame - 1,
                    )
                  : makeEmptyButtons();
              setPlaying(false);
              return;
            }
          }
          if (shouldVerify && activeTutorial && activeModule) {
            const localFrame =
              beforeSession.phase === "pre_fuzz"
                ? 0
                : currentFrame - (fuzzStartRef.current ?? currentFrame);
            const semanticKeys = verificationKeys(
              current,
              previous,
              expectedInput,
            );
            let entryPassed = true;
            if (beforeSession.phase === "pre_fuzz") {
              entryPassed = trainingEntryContextPassed(current, activeTutorial);
              if (entryPassed)
                entryPassed = await client.entryCheck(
                  after,
                  activeTutorial.entry.check,
                );
              if (!active || epoch !== simulationEpoch.current) return;
            }
            const nextSession = verifyTrainingInput(
              beforeSession,
              activeTutorial,
              localFrame,
              semanticKeys,
              entryPassed,
            );
            const evaluatedCandidate = matchingTrainingCandidate(
              evaluationsRef.current,
              activeTutorial,
              [
                ...beforeSession.actualInputs,
                { frame: localFrame, keys: semanticKeys },
              ],
            );
            const entryAccepted =
              beforeSession.phase === "pre_fuzz" &&
              (nextSession.phase === "fuzz" || nextSession.phase === "success");
            attempts.current = [
              ...attempts.current,
              {
                frame: currentFrame,
                inputId: expectedInput?.id ?? "",
                keys: semanticKeys,
                entryCheckPassed: entryPassed,
                entryAccepted,
              },
            ];
            if (entryAccepted) {
              fuzzStartRef.current = currentFrame;
              setFuzzStartFrame(currentFrame);
            }
            capturePrediction(
              nextSession.phase === "failed" || nextSession.phase === "success"
                ? beforeSession
                : nextSession,
              fuzzStartRef.current ?? currentFrame,
              activeTutorial,
            );
            applySession(nextSession);
            if (nextSession.phase === "failed") {
              setNotice(
                nextSession.failure?.kind === "input_order_mismatch"
                  ? (activeTutorial.teaching.steps[
                      nextSession.nextVerifiedInput
                    ]?.order_error.body ?? "输入顺序不正确。")
                  : nextSession.failure?.kind === "timing_window_miss"
                    ? (activeTutorial.teaching.steps[
                        nextSession.nextVerifiedInput
                      ]?.window_error.body ?? "错过输入窗口。")
                    : activeTutorial.entry.failure.body,
              );
              beginFailure(nextFrame, evaluatedCandidate);
            } else if (nextSession.phase === "success") {
              const preview = predictionRef.current;
              const start = fuzzStartRef.current ?? currentFrame;
              const absoluteInputs = nextSession.actualInputs.map((actual) => ({
                ...actual,
                frame: start + actual.frame,
              }));
              const actualActionFrame = absoluteInputs.at(-1)?.frame;
              const allObjectiveValues =
                evaluatedCandidate?.objective_values ??
                nextSession.candidates[0]?.objective_values ??
                [];
              const projectedObjectiveValues = (
                preview.objectiveResultIndices ?? []
              ).map((index) => allObjectiveValues[index] ?? Number.NaN);
              const projectedBestValues = preview.bestObjectiveValues ?? [];
              const finiteObjectiveResults = preview.objectives.flatMap(
                (objective, index) => {
                  const actual = projectedObjectiveValues[index];
                  const best = projectedBestValues[index];
                  return Number.isFinite(actual) && Number.isFinite(best)
                    ? [{ objective, actual, best }]
                    : [];
                },
              );
              const objectiveValues = finiteObjectiveResults.map(
                ({ actual }) => actual,
              );
              const bestObjectiveValues = finiteObjectiveResults.map(
                ({ best }) => best,
              );
              const completionObjectives = finiteObjectiveResults.map(
                ({ objective }) => objective,
              );
              const completion: TrainingCompletion = {
                moduleId: activeModule.id,
                title: activeTutorial.title,
                triggerFrame: triggerFrameRef.current ?? start,
                startedFrame: start,
                completedFrame: nextFrame,
                targetFrame: preview.targetFrame,
                actualInputFrame: actualActionFrame,
                accuracy:
                  completionObjectives.length === 0
                    ? 100
                    : outputAccuracy(
                        objectiveValues[0],
                        bestObjectiveValues[0],
                      ),
                reactionFrames: Math.max(
                  0,
                  start - (triggerFrameRef.current ?? start),
                ),
                objectiveValues,
                bestObjectiveValues,
                objectives: completionObjectives,
                windows: preview.windows,
                actualInputs: absoluteInputs,
              };
              stageInputSuccessRef.current = true;
              if (stage === "free") pendingCompletionRef.current = completion;
              setPlaying(true);
              setNotice(
                `${activeTutorial.title} 动作正确；继续前往本段结束触发区。`,
              );
            }
          }
          const reachedModuleEnd =
            activeModule !== null &&
            triggerContainsPlayer(activeModule.end_trigger, after);
          if (
            reachedModuleEnd &&
            activeModule &&
            stage === "demo" &&
            demoInputsFinishedRef.current
          ) {
            holdGuidedResult("demo", activeModule);
            return;
          }
          if (
            reachedModuleEnd &&
            activeModule &&
            stage === "assisted" &&
            stageInputSuccessRef.current
          ) {
            holdGuidedResult("assisted", activeModule);
            return;
          }
          if (
            reachedModuleEnd &&
            activeModule &&
            stage === "free" &&
            pendingCompletionRef.current
          ) {
            const completion = {
              ...pendingCompletionRef.current,
              completedFrame: nextFrame,
            };
            pendingCompletionRef.current = null;
            completionsRef.current = [
              ...completionsRef.current.filter(
                (item) => item.moduleId !== completion.moduleId,
              ),
              completion,
            ];
            setCompletions(completionsRef.current);
            const moduleIndex = selectedVariant.training.modules.findIndex(
              (candidate) => candidate.id === activeModule.id,
            );
            const nextModule =
              selectedVariant.training.modules[moduleIndex + 1];
            if (nextModule) {
              const nextState = structuredClone(
                nextModule.validation.initial_state,
              );
              snapshotsRef.current = [
                ...snapshotsRef.current.slice(0, nextFrame),
                nextState,
              ];
              setSnapshots(snapshotsRef.current);
              frameRef.current = nextFrame;
              setFrame(nextFrame);
              previousButtons.current = buttonsFromKeyboard(
                keys.current,
                bindings,
              );
              occupiedTriggers.current = idsInside(nextState, selectedVariant);
              setActiveModule(nextModule, nextFrame, "free");
              setNotice(
                `${activeModule.tutorial.title} 完成；${nextModule.tutorial.title} 已进入自由练习。`,
              );
              return;
            }
            setActiveModule(null, null);
            setNotice(
              `${activeModule.tutorial.title} 完成；房间内训练已全部通过，继续前往终点。`,
            );
          }
          scanTriggers(after, nextFrame, completionsRef.current);
        })()
          .catch((error: unknown) => {
            if (active && epoch === simulationEpoch.current) {
              setPlaying(false);
              setNotice(
                error instanceof Error ? error.message : "训练模拟失败",
              );
            }
          })
          .finally(() => {
            simulating.current = false;
          });
      }
      if (active) animation = requestAnimationFrame(tick);
    };
    animation = requestAnimationFrame(tick);
    return () => {
      active = false;
      cancelAnimationFrame(animation);
    };
    // Runtime refs intentionally carry module/candidate changes without restarting the clock.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    autoSlowdown,
    baseRate,
    bindings,
    client,
    initial,
    map,
    playing,
    settlement,
  ]);

  if (!map || !initial || snapshots.length === 0)
    return (
      <main
        className={`training-workspace ${editorPreview ? "editor-training-preview" : ""}`}
      >
        {!editorPreview && (
          <TrainingCatalogSidebar
            techniqueId={technique.id}
            variantId={selectedVariant.id}
            onSelectTraining={onSelectTraining}
          />
        )}
        <div className="training-loading notice">
          <i />
          {notice}
        </div>
      </main>
    );

  const activeModule = selectedVariant.training.modules.find(
    (module) => module.id === activeModuleId,
  );
  const tutorial = activeModule?.tutorial ?? null;
  const state = snapshots[frame] ?? snapshots.at(-1) ?? initial;
  const fuzzInputIndex = tutorial
    ? currentTrainingInput(session, tutorial)?.fuzzInputIndex
    : undefined;
  const target =
    fuzzInputIndex === undefined
      ? undefined
      : nextTargetFrame(session.candidates, fuzzInputIndex);
  const demoCandidate = candidates[0];
  const demoSteps =
    tutorial && demoCandidate
      ? trainingReferenceSteps(demoCandidate, tutorial)
      : [];
  const activeDemoStep =
    lessonStage === "demo" ? demoSteps[demoStepIndex] : undefined;
  const demoCue = (activeDemoStep?.keys ?? [])
    .map((key) =>
      ACTIONS.includes(key as (typeof ACTIONS)[number])
        ? ACTION_LABELS[key as (typeof ACTIONS)[number]]
        : key,
    )
    .join(" + ");
  const prompt = (() => {
    if (!tutorial) return "";
    if (lessonStage === "demo") {
      if (!demoCandidate) return "正在准备本段演示…";
      if (!activeDemoStep) return "演示动作完成，正在播放到结束位置…";
      const teachingPrompt = tutorial.teaching.steps[demoStepIndex]?.prompt;
      return `演示 ${demoStepIndex + 1}/${demoSteps.length} · ${teachingPrompt ?? `现在按：${demoCue}`}`;
    }
    if (lessonStage === "free") return "自由练习：自己决定按法与时机。";
    return session.phase === "pre_fuzz"
      ? tutorial.entry.hint
      : (tutorial.teaching.steps[session.nextVerifiedInput]?.prompt ??
          "继续保持。");
  })();
  const renderFuzzFrame =
    lessonStage === "demo"
      ? Math.max(0, frame - lessonStartFrameRef.current)
      : fuzzStartFrame === null
        ? 0
        : Math.max(0, frame - fuzzStartFrame);
  const renderBrake =
    lessonStage === "assisted" &&
    tutorial &&
    autoSlowdown &&
    fuzzInputIndex !== undefined
      ? assistedWindowBrake(
          session.candidates,
          fuzzInputIndex,
          renderFuzzFrame,
        )
      : undefined;
  const renderDemoBrake =
    lessonStage === "demo" && !demoAdvanceRef.current
      ? referenceStepBrake(renderFuzzFrame, activeDemoStep?.frame)
      : undefined;
  const displayedPrompt = guidedResultStage
    ? guidedResultStage === "demo"
      ? "演示完成 · 看一下角色最后的落点和状态"
      : "辅助实操完成 · 看一下这次操作的最终结果"
    : lessonStage === "assisted" && renderBrake?.stopped
      ? `现在按：${(currentTrainingInput(session, tutorial!)?.keys ?? [])
          .map((key) =>
            ACTIONS.includes(key as (typeof ACTIONS)[number])
              ? ACTION_LABELS[key as (typeof ACTIONS)[number]]
              : key,
          )
          .join(" + ")}`
      : prompt;
  const effective =
    (lessonStage === "demo" && renderDemoBrake
      ? baseRate * renderDemoBrake.multiplier
      : lessonStage === "assisted" && renderBrake
        ? baseRate * renderBrake.multiplier
        : baseRate) *
    (1 - outcomeProgress);
  const actualInputs = session.actualInputs.map((input) => ({
    ...input,
    frame: (fuzzStartFrame ?? 0) + input.frame,
  }));
  const actualActionFrame = actualInputs.at(-1)?.frame;
  const failureOriginFrame =
    session.failure === undefined
      ? undefined
      : (fuzzStartFrame ??
        Math.max(
          0,
          (outcome?.timelineFrame ?? frame) - 1 - session.failure.frame,
        ));
  const failureFrame = session.failure
    ? (failureOriginFrame ?? 0) + session.failure.frame
    : undefined;
  const failureTiming = timingAssessmentToWindow(
    failureFrame,
    prediction.windows,
  );
  const timelineFrame = outcome?.timelineFrame ?? frame;
  const timelineFrameCount =
    outcome?.timelineFrame ??
    Math.max(40, snapshots.length - 1, (prediction.targetFrame ?? 0) + 24);
  const recommendations = editorPreview
    ? []
    : technique.variants
        .filter((variant) => variant.id !== selectedVariant.id)
        .slice(0, 2);
  const nextVariant = editorPreview
    ? undefined
    : technique.variants[variantIndex + 1];
  const averageAccuracy = average(
    completions.map((completion) => completion.accuracy),
  );
  const averageReactionFrames = average(
    completions.map((completion) => completion.reactionFrames),
  );

  return (
    <main
      className={`training-workspace ${timelineOpen ? "timeline-open" : ""} ${editorPreview ? "editor-training-preview" : ""} training-stage-${lessonStage} ${(renderBrake?.stopped || renderDemoBrake?.stopped) ? "training-cue-stopped" : (renderBrake?.braking || renderDemoBrake?.braking) ? "training-cue-braking" : ""}`}
    >
      {!editorPreview && (
        <TrainingCatalogSidebar
          techniqueId={technique.id}
          variantId={selectedVariant.id}
          onSelectTraining={onSelectTraining}
        />
      )}
      <section className="training-stage panel-frame">
        <div className="stage-header">
          <div>
            <small>TRAINING MAP / {selectedVariant.training.id}</small>
            <h1>
              {selectedVariant.training.title}{" "}
              <em>
                {completions.length}/{selectedVariant.training.modules.length}{" "}
                模块完成
              </em>
            </h1>
            {tutorial && lessonMode && (
              <div className="training-lesson-stages" aria-label="训练三步">
                {LESSON_STAGES.map((item, index) => (
                  <button
                    type="button"
                    key={item.id}
                    className={
                      item.id === lessonStage
                        ? "active"
                        : LESSON_STAGES.findIndex(
                              (stage) => stage.id === lessonStage,
                            ) > index
                          ? "complete"
                          : ""
                    }
                    onClick={() => selectLessonStage(item.id)}
                  >
                    <b>{index + 1}</b> {item.label}
                  </button>
                ))}
              </div>
            )}
          </div>
          <div className="cache-meter">
            <span>{tutorial ? "当前模块倍率" : "地图进度"}</span>
            <strong>
              {tutorial
                ? `${effective.toFixed(2)}×`
                : `${Math.round((completions.length / selectedVariant.training.modules.length) * 100)}%`}
            </strong>
          </div>
        </div>
        <GameView
          map={map}
          state={state}
          states={snapshots}
          frame={frame}
          stale={false}
          theme={theme}
        >
          {(viewport) =>
            tutorial && (
              <TrainingPrompt
                map={map}
                state={state}
                viewport={viewport}
                text={displayedPrompt}
                hidden={
                   outcome !== null || settlement || lessonStage === "free"
                 }
              />
            )
          }
        </GameView>

        {tutorial && lessonStage === "free" && !lessonMode && !outcome && (
          <button
            type="button"
            className={`training-tutorial-launch ${mouseActive ? "mouse-active" : ""}`}
            onClick={startLesson}
          >
            查看本段教学
          </button>
        )}

        {tutorial &&
          lessonMode &&
          lessonStage === "demo" &&
          !guidedResultStage && (
            <div className="training-demo-controls" aria-label="演示步骤">
              <button
                type="button"
                disabled={!demoCandidate || playing || demoStepIndex <= 0}
                onClick={() => moveDemoStep(-1)}
              >
                上一步
              </button>
              <span>
                {demoCandidate
                  ? `${Math.min(demoStepIndex + 1, Math.max(1, demoSteps.length))}/${Math.max(1, demoSteps.length)}`
                  : "准备中"}
              </span>
              <button
                type="button"
                disabled={!demoCandidate || playing}
                onClick={() => moveDemoStep(1)}
              >
                {!demoCandidate
                  ? "准备演示"
                  : playing
                    ? "播放中"
                    : demoStepIndex >= demoSteps.length - 1
                      ? "完成演示"
                      : "下一步"}
              </button>
            </div>
          )}

        <div
          className={`training-success-toasts ${settlement ? "settling" : ""}`}
          aria-live="polite"
        >
          {completions.map((completion, index) => (
            <article
              className="training-success-toast"
              key={completion.moduleId}
              style={{ "--toast-index": index } as CSSProperties}
            >
              <header>
                <span>✓</span>
                <div>
                  <small>MODULE CLEAR</small>
                  <strong>{completion.title}</strong>
                </div>
                <b>{completion.accuracy.toFixed(0)}%</b>
              </header>
              <TrainingResultTimeline
                frameOrigin={completion.startedFrame}
                targetFrame={completion.targetFrame}
                windows={completion.windows}
                actualInputs={completion.actualInputs}
                objectives={completion.objectives}
              />
              <footer>
                <strong>{completionOutputSummary(completion)}</strong>
                <span>
                  {timingAssessment(
                    completion.actualInputFrame,
                    completion.targetFrame,
                  )}{" "}
                  · 反应 {completion.reactionFrames}F
                </span>
              </footer>
            </article>
          ))}
        </div>

        {outcome && tutorial && (
          <div
            className="training-failure"
            style={{ "--outcome-progress": outcomeProgress } as CSSProperties}
          >
            <div className="training-outcome-layout failed">
              <div className="training-result-card failed">
                <div className="training-result-heading">
                  <div>
                    <small>ATTEMPT RESULT</small>
                    <strong>
                      {session.failure?.kind === "entry_check_failed"
                        ? tutorial.entry.failure.title
                        : session.failure?.kind === "input_order_mismatch"
                          ? tutorial.teaching.steps[session.nextVerifiedInput]
                              ?.order_error.title
                          : tutorial.teaching.steps[session.nextVerifiedInput]
                              ?.window_error.title}
                    </strong>
                  </div>
                  <em>{failureTiming}</em>
                </div>
                <TrainingResultTimeline
                  frameOrigin={failureOriginFrame}
                  targetFrame={prediction.targetFrame}
                  windows={prediction.windows}
                  actualInputs={actualInputs}
                  failureFrame={failureFrame}
                  objectives={prediction.objectives}
                />
                <div className="training-result-stats">
                  {prediction.objectives.map((objective, index) => (
                    <div key={`${objective.expression}-${index}`}>
                      <span>本次 OBJECTIVE · {objective.expression}</span>
                      <b>
                        {!Number.isFinite(outcome.objectiveValues[index])
                          ? "—"
                          : outcome.objectiveValues[index].toFixed(2)}
                      </b>
                    </div>
                  ))}
                  <div>
                    <span>输入时机</span>
                    <b>{failureTiming}</b>
                  </div>
                </div>
                <p>{notice}</p>
                <div className="training-result-actions">
                  <button className="primary" onClick={retry}>
                    R 重试
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}

        {settlement && (
          <div
            className="training-settlement"
            role="dialog"
            aria-modal="true"
            aria-label="训练地图结算"
          >
            <section
              className="training-settlement-history"
              aria-label="已完成模块"
            >
              {completions.map((completion, index) => (
                <article
                  key={completion.moduleId}
                  style={{ "--settlement-index": index } as CSSProperties}
                >
                  <header>
                    <div>
                      <small>0{index + 1} · MODULE CLEAR</small>
                      <strong>{completion.title}</strong>
                    </div>
                    <b>{completion.accuracy.toFixed(0)}%</b>
                  </header>
                  <TrainingResultTimeline
                    frameOrigin={completion.startedFrame}
                    targetFrame={completion.targetFrame}
                    windows={completion.windows}
                    actualInputs={completion.actualInputs}
                    objectives={completion.objectives}
                  />
                  <footer>
                    {completionOutputSummary(completion)} · 反应{" "}
                    {completion.reactionFrames}F
                  </footer>
                </article>
              ))}
            </section>
            <section className="training-settlement-card">
              <div className="training-strawberry">
                <GameplayStrawberry />
              </div>
              <small>MAP COMPLETE</small>
              <h2>通过啦</h2>
              <strong className="training-accuracy">
                {averageAccuracy.toFixed(1)}%
              </strong>
              <span>综合精准度</span>
              <div className="training-summary-stats">
                <div>
                  <span>平均反应速度</span>
                  <b>{((averageReactionFrames * 1000) / 60).toFixed(0)} ms</b>
                </div>
                <div>
                  <span>完成模块</span>
                  <b>
                    {completions.length}/
                    {selectedVariant.training.modules.length}
                  </b>
                </div>
                <div>
                  <span>总用时</span>
                  <b>{(frame / 60).toFixed(2)} s</b>
                </div>
                <div>
                  <span>最佳精准度</span>
                  <b>
                    {Math.max(
                      ...completions.map((completion) => completion.accuracy),
                      0,
                    ).toFixed(0)}
                    %
                  </b>
                </div>
              </div>
              <div className="training-result-actions">
                <button onClick={() => void installVariant(selectedVariant)}>
                  重试
                </button>
                {nextVariant && (
                  <button
                    className="primary"
                    onClick={() =>
                      onSelectTraining(technique.id, nextVariant.id)
                    }
                  >
                    下一关
                  </button>
                )}
              </div>
            </section>
            <aside className="training-recommendations" aria-label="推荐地图">
              <div className="training-recommendations-heading">
                <small>KEEP CLIMBING</small>
                <h2>推荐地图</h2>
                <p>继续挑战其他训练地图。</p>
              </div>
              <div className="training-recommendation-list">
                {recommendations.map((variant) => (
                  <button
                    type="button"
                    key={variant.id}
                    onClick={() => onSelectTraining(technique.id, variant.id)}
                  >
                    <TrainingVariantThumbnail variant={variant} />
                    <span>
                      <small>
                        {variant.training.modules.length} 个训练模块
                      </small>
                      <strong>{variant.title}</strong>
                      <em>{variant.summary}</em>
                      <b>
                        开始训练 <i aria-hidden="true">→</i>
                      </b>
                    </span>
                  </button>
                ))}
              </div>
            </aside>
          </div>
        )}

        <div className="transport">
          <button
            type="button"
            className="timeline-toggle"
            aria-expanded={timelineOpen}
            aria-controls="training-timeline"
            onClick={() => setTimelineOpen((open) => !open)}
          >
            {timelineOpen ? "收起时间线" : "时间线"}
          </button>
          <button aria-label="按 R 设置重试" onClick={retry}>
            R
          </button>
          <button aria-label="上一帧" onClick={() => seek(frame - 1)}>
            ◀
          </button>
          <button
            className="play-button"
            onClick={() => {
              if (!playing && predictionDirty.current) {
                predictionDirty.current = false;
                applyPrediction({ windows: [], objectives: [] });
              }
              setPlaying((value) => !value);
            }}
          >
            {playing ? "Ⅱ" : "▶"}
          </button>
          <button aria-label="下一帧" onClick={() => setPlaying(true)}>
            ▶
          </button>
          <select
            aria-label="训练基础速度"
            value={baseRate}
            onChange={(event) => setBaseRate(Number(event.target.value))}
          >
            <option value={0.25}>0.25×</option>
            <option value={0.5}>0.5×</option>
            <option value={1}>1×</option>
            <option value={2}>2×</option>
          </select>
          <label className="training-assist">
            <input
              type="checkbox"
              checked={lessonStage === "assisted" && autoSlowdown}
              disabled={lessonStage !== "assisted"}
              onChange={(event) => setAutoSlowdown(event.target.checked)}
            />
            {lessonStage === "free" ? "自由阶段" : "自动慢放"}
          </label>
        </div>
      </section>
      {timelineOpen && (
        <div id="training-timeline">
          <TrainingTimeline
            frame={timelineFrame}
            frameCount={timelineFrameCount}
            fuzzStart={fuzzStartFrame}
            targetFrame={
              lessonStage === "free" && !outcome
                ? undefined
                : prediction.targetFrame
            }
            windows={
              lessonStage === "free" && !outcome ? [] : prediction.windows
            }
            actualInputs={actualInputs}
            failureFrame={failureFrame}
            resetFrame={resetFrame}
            objectives={
              lessonStage === "free" && !outcome ? [] : prediction.objectives
            }
            followTarget={
              followReference && !outcome && lessonStage !== "free"
            }
            onSeek={(value, manual) => {
              if (manual && value < timelineFrame) setFollowReference(false);
              seek(value);
            }}
            onSetReset={(value) => {
              setResetFrame(value);
              setNotice(`临时 R 点已设为 F${value}`);
            }}
          />
        </div>
      )}
    </main>
  );
}
