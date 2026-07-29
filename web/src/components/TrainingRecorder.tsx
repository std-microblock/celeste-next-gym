import { useEffect, useMemo, useRef, useState } from "react";
import {
  ACTIONS,
  buttonsToInput,
  createInitialState,
  makeEmptyButtons,
  type FrameButtons,
  type KeyBindings,
  type SimState,
} from "../model";
import { WasmClient } from "../simulator/wasmClient";
import { triggerContainsPlayer } from "../training/course";
import type { TrainingProject } from "../training/editorProject";
import {
  applyTutorialRecording,
  hasRecordedAction,
  nextSequentialModuleAtPlayer,
  RECORDING_TARGET_GROUPS,
  recordedCriticalNodesFromFrames,
  recordingStartState,
  recordingTargetCondition,
  type RecordedCriticalNode,
  type RecordingTargetKind,
  type RecordingTargetSelections,
} from "../training/recording";
import type { VisualTheme } from "../visualThemes";
import { GameView } from "./GameView";

export type TrainingRecordingScope =
  | { type: "module"; index: number }
  | { type: "all" };

type RecorderPhase =
  | "loading"
  | "armed"
  | "recording"
  | "reviewing"
  | "roaming"
  | "done";

interface PendingReview {
  moduleIndex: number;
  title: string;
  initial: SimState;
  frames: FrameButtons[];
  snapshots: SimState[];
  nodes: RecordedCriticalNode[];
  selections: RecordingTargetSelections;
  currentNode: number;
  startGlobalFrame: number;
  after: SimState;
}

function downloadJson(name: string, value: unknown): void {
  const url = URL.createObjectURL(
    new Blob([`${JSON.stringify(value, null, 2)}\n`], {
      type: "application/json",
    }),
  );
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = name;
  anchor.click();
  URL.revokeObjectURL(url);
}

function buttonsFromKeyboard(
  keys: ReadonlySet<string>,
  bindings: KeyBindings,
): FrameButtons {
  const buttons = makeEmptyButtons();
  for (const action of ACTIONS) buttons[action] = keys.has(bindings[action]);
  return buttons;
}

function initialForScope(
  project: TrainingProject,
  scope: TrainingRecordingScope,
): SimState {
  if (scope.type === "module") {
    return recordingStartState(project, scope.index);
  }
  return createInitialState(project.map);
}

export function TrainingRecorder({
  project,
  scope,
  bindings,
  theme,
  onChange,
  onExit,
}: {
  project: TrainingProject;
  scope: TrainingRecordingScope;
  bindings: KeyBindings;
  theme: VisualTheme;
  onChange: (project: TrainingProject) => void;
  onExit: () => void;
}) {
  const client = useMemo(() => new WasmClient(), []);
  const onChangeRef = useRef(onChange);
  const onExitRef = useRef(onExit);
  onChangeRef.current = onChange;
  onExitRef.current = onExit;
  const originalProject = useRef(structuredClone(project));
  const projectRef = useRef(structuredClone(project));
  const initialRef = useRef(initialForScope(project, scope));
  const snapshotsRef = useRef<SimState[]>([initialRef.current]);
  const [snapshots, setSnapshots] = useState<SimState[]>(snapshotsRef.current);
  const frameRef = useRef(0);
  const [frame, setFrame] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [phase, setPhase] = useState<RecorderPhase>("loading");
  const activeIndexRef = useRef<number | null>(
    scope.type === "module" ? scope.index : null,
  );
  const [activeIndex, setActiveIndex] = useState<number | null>(
    activeIndexRef.current,
  );
  const completedRef = useRef(new Set<number>());
  const [completedCount, setCompletedCount] = useState(0);
  const recordingInitial = useRef<SimState | null>(null);
  const recordingFrames = useRef<FrameButtons[]>([]);
  const recordingSnapshots = useRef<SimState[]>([]);
  const recordingStartGlobalFrame = useRef(0);
  const [review, setReview] = useState<PendingReview | null>(null);
  const previousButtons = useRef<FrameButtons>(makeEmptyButtons());
  const keys = useRef(new Set<string>());
  const simulating = useRef(false);
  const simulationEpoch = useRef(0);
  const [notice, setNotice] = useState("正在启动录制器…");

  const setActive = (index: number | null) => {
    activeIndexRef.current = index;
    setActiveIndex(index);
    recordingInitial.current = null;
    recordingFrames.current = [];
    recordingSnapshots.current = [];
    if (index === null) {
      setPhase("roaming");
      setNotice("继续游玩；进入下一个教程开始区后会自动待命。");
    } else {
      const title = projectRef.current.training.modules[index]?.tutorial.title;
      setPhase("armed");
      setNotice(
        `${title ?? `教程 ${index + 1}`} 已待命；首个非 WASD 动作记为 F0。`,
      );
    }
  };

  const armModuleAt = (state: SimState) => {
    if (activeIndexRef.current !== null) return;
    const index = nextSequentialModuleAtPlayer(
      projectRef.current,
      state,
      completedRef.current,
    );
    if (index !== null) setActive(index);
  };

  const installInitial = (restoreProject: boolean) => {
    simulationEpoch.current += 1;
    if (restoreProject) {
      projectRef.current = structuredClone(originalProject.current);
      onChangeRef.current(projectRef.current);
    }
    const initial = initialForScope(projectRef.current, scope);
    initialRef.current = initial;
    snapshotsRef.current = [initial];
    setSnapshots([initial]);
    frameRef.current = 0;
    setFrame(0);
    previousButtons.current = makeEmptyButtons();
    keys.current.clear();
    setReview(null);
    completedRef.current.clear();
    setCompletedCount(0);
    setPlaying(false);
    if (scope.type === "module") setActive(scope.index);
    else {
      activeIndexRef.current = null;
      setActiveIndex(null);
      setPhase("roaming");
      setNotice("录制全部已就绪；从出生点开始游玩并依次经过教程区域。");
      armModuleAt(initial);
    }
  };

  const showReviewNode = (pending: PendingReview, index: number) => {
    const node = pending.nodes[index];
    if (!node) return;
    setReview({ ...pending, currentNode: index });
    setFrame(pending.startGlobalFrame + node.frame + 1);
  };

  const toggleReviewTarget = (kind: RecordingTargetKind) => {
    setReview((pending) => {
      if (!pending) return pending;
      const node = pending.nodes[pending.currentNode];
      const selected = pending.selections[node.id] ?? [];
      const selections = {
        ...pending.selections,
        [node.id]: selected.includes(kind)
          ? selected.filter((candidate) => candidate !== kind)
          : [...selected, kind],
      };
      return { ...pending, selections };
    });
  };

  const finishReview = (pending: PendingReview | null = review) => {
    if (!pending) return;
    const next = applyTutorialRecording(
      projectRef.current,
      pending.moduleIndex,
      pending.initial,
      pending.frames,
      pending.snapshots,
      pending.selections,
    );
    projectRef.current = next;
    onChangeRef.current(next);
    completedRef.current.add(pending.moduleIndex);
    setCompletedCount(completedRef.current.size);
    recordingInitial.current = null;
    recordingFrames.current = [];
    recordingSnapshots.current = [];
    activeIndexRef.current = null;
    setActiveIndex(null);
    setReview(null);
    setFrame(frameRef.current);
    if (
      scope.type === "module" ||
      completedRef.current.size === next.training.modules.length
    ) {
      setPlaying(false);
      setPhase("done");
      setNotice(
        scope.type === "module"
          ? `${pending.title} 已生成教程 JSON。`
          : `全部 ${next.training.modules.length} 个教程均已生成 JSON。`,
      );
    } else {
      setPhase("roaming");
      setNotice(`${pending.title} 已写入；继续前往下一个开始区。`);
      armModuleAt(pending.after);
    }
  };

  const deleteReviewNode = () => {
    if (!review) return;
    const current = review.nodes[review.currentNode];
    if (!current) return;
    const nodes = review.nodes.filter((node) => node.id !== current.id);
    const selections = { ...review.selections };
    delete selections[current.id];
    const pending = {
      ...review,
      nodes,
      selections,
      currentNode: Math.min(review.currentNode, Math.max(0, nodes.length - 1)),
    };
    if (!nodes.length) finishReview(pending);
    else showReviewNode(pending, pending.currentNode);
  };

  useEffect(() => {
    let active = true;
    void client
      .ready()
      .then(() => {
        if (!active) return;
        installInitial(false);
      })
      .catch((error: unknown) => {
        if (!active) return;
        setNotice(error instanceof Error ? error.message : "录制器启动失败");
      });
    return () => {
      active = false;
      client.dispose();
    };
    // The recorder owns a stable snapshot of the project for its whole session.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [client]);

  useEffect(() => {
    const down = (event: KeyboardEvent) => {
      if (
        event.target instanceof HTMLElement &&
        event.target.matches("input, select, button")
      )
        return;
      if (event.code === "Escape") {
        event.preventDefault();
        onExitRef.current();
        return;
      }
      const gameInput = Object.values(bindings).includes(event.code);
      const resetInput = event.code === "KeyR" && !gameInput;
      if (gameInput || resetInput) event.preventDefault();
      if (resetInput && !event.repeat) {
        installInitial(true);
        return;
      }
      if (!gameInput || phase === "done" || phase === "reviewing") return;
      keys.current.add(event.code);
      setPlaying(true);
    };
    const up = (event: KeyboardEvent) => {
      keys.current.delete(event.code);
    };
    const blur = () => keys.current.clear();
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    window.addEventListener("blur", blur);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
      window.removeEventListener("blur", blur);
    };
    // Recorder state lives in refs so key listeners do not drop held inputs.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [bindings, phase]);

  useEffect(() => {
    if (
      !playing ||
      phase === "loading" ||
      phase === "reviewing" ||
      phase === "done"
    )
      return;
    let active = true;
    let animation = 0;
    let last = performance.now();
    let carry = 0;
    const tick = (now: number) => {
      carry += (Math.min(250, now - last) * 60) / 1000;
      last = now;
      if (carry >= 1 && !simulating.current) {
        carry -= 1;
        const currentFrame = frameRef.current;
        const before = snapshotsRef.current[currentFrame];
        if (!before) return;
        const currentButtons = buttonsFromKeyboard(keys.current, bindings);
        const previous = previousButtons.current;
        const moduleIndex = activeIndexRef.current;
        if (
          moduleIndex !== null &&
          recordingInitial.current === null &&
          hasRecordedAction(currentButtons, previous)
        ) {
          recordingInitial.current = structuredClone(before);
          recordingFrames.current = [structuredClone(currentButtons)];
          recordingSnapshots.current = [structuredClone(before)];
          recordingStartGlobalFrame.current = currentFrame;
          setPhase("recording");
          setNotice(
            `${projectRef.current.training.modules[moduleIndex].tutorial.title} 录制中；进入粉色结束区自动完成。`,
          );
        } else if (recordingInitial.current !== null) {
          recordingFrames.current.push(structuredClone(currentButtons));
        }
        previousButtons.current = currentButtons;
        simulating.current = true;
        const epoch = simulationEpoch.current;
        void client
          .simulate(
            before,
            [buttonsToInput(currentButtons, previous)],
            projectRef.current.map,
          )
          .then((trace) => {
            if (!active || epoch !== simulationEpoch.current) return;
            const after = trace.at(-1);
            if (!after) throw new Error("录制模拟未返回状态");
            const nextFrame = currentFrame + 1;
            snapshotsRef.current = [
              ...snapshotsRef.current.slice(0, nextFrame),
              after,
            ];
            setSnapshots(snapshotsRef.current);
            frameRef.current = nextFrame;
            setFrame(nextFrame);

            if (recordingInitial.current)
              recordingSnapshots.current.push(structuredClone(after));

            const recordingIndex = activeIndexRef.current;
            const module =
              recordingIndex === null
                ? undefined
                : projectRef.current.training.modules[recordingIndex];
            if (
              module &&
              recordingInitial.current &&
              triggerContainsPlayer(module.end_trigger, after)
            ) {
              const title = module.tutorial.title;
              const frames = structuredClone(recordingFrames.current);
              const nodes = recordedCriticalNodesFromFrames(frames);
              const pending: PendingReview = {
                moduleIndex: recordingIndex!,
                title,
                initial: structuredClone(recordingInitial.current),
                frames,
                snapshots: structuredClone(recordingSnapshots.current),
                nodes,
                selections: Object.fromEntries(
                  nodes.map((node) => [node.id, []]),
                ),
                currentNode: 0,
                startGlobalFrame: recordingStartGlobalFrame.current,
                after: structuredClone(after),
              };
              setPlaying(false);
              setPhase("reviewing");
              setNotice(
                `${title} 录制完成；请逐个关键节点选择一个或多个目标。`,
              );
              showReviewNode(pending, 0);
            } else {
              if (after.dead) {
                setPlaying(false);
                setNotice("人物已死亡；按 R 重新开始本次录制。");
              }
              if (scope.type === "all") armModuleAt(after);
            }
          })
          .catch((error: unknown) => {
            if (!active || epoch !== simulationEpoch.current) return;
            setPlaying(false);
            setNotice(error instanceof Error ? error.message : "录制模拟失败");
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
  }, [bindings, client, playing, scope.type]);

  const state = snapshots[frame] ?? snapshots.at(-1) ?? initialRef.current;
  const activeModule =
    activeIndex === null
      ? null
      : projectRef.current.training.modules[activeIndex];
  const reviewNode = review?.nodes[review.currentNode];
  const reviewTargets = reviewNode
    ? (review?.selections[reviewNode.id] ?? [])
    : [];
  const reviewSnapshot =
    review && reviewNode
      ? (review.snapshots[reviewNode.frame + 1] ?? review.after)
      : null;
  return (
    <main className="training-recorder">
      <div className="training-recorder-bar">
        <div>
          <small>TUTORIAL AUTO RECORDER</small>
          <strong>
            {scope.type === "all"
              ? `录制全部 · ${completedCount}/${projectRef.current.training.modules.length}`
              : `录制区域 · ${activeModule?.tutorial.title ?? project.training.modules[scope.index]?.tutorial.title}`}
          </strong>
        </div>
        <span>{notice}</span>
        <button onClick={() => installInitial(true)}>R · 重新开始</button>
        <button
          disabled={phase !== "done"}
          onClick={() =>
            downloadJson(
              projectRef.current.trainingFileName,
              projectRef.current.training,
            )
          }
        >
          导出教程 JSON
        </button>
        <button className={phase === "done" ? "primary" : ""} onClick={onExit}>
          {phase === "done" ? "完成并返回" : "退出录制"}
        </button>
      </div>
      <GameView
        map={projectRef.current.map}
        state={state}
        states={snapshots}
        frame={frame}
        stale={false}
        theme={theme}
      >
        {({ width, height }) => {
          const bounds = projectRef.current.map.bounds;
          const scale = Math.min(width / bounds.width, height / bounds.height);
          const offsetX = (width - bounds.width * scale) / 2;
          const offsetY = (height - bounds.height * scale) / 2;
          const anchorX = reviewSnapshot
            ? offsetX + (reviewSnapshot.pos.x - bounds.x) * scale
            : 0;
          const anchorY = reviewSnapshot
            ? offsetY + (reviewSnapshot.pos.y - bounds.y) * scale
            : 0;
          const windowWidth = Math.min(252, width - 16);
          const preferredRight = anchorX + 48;
          const windowLeft =
            preferredRight + windowWidth <= width - 8
              ? preferredRight
              : Math.max(8, anchorX - windowWidth - 48);
          const windowTop = Math.min(
            Math.max(8, anchorY - 108),
            Math.max(8, height - 244),
          );
          return (
            <>
              <svg
                className={`training-recorder-regions ${phase === "reviewing" ? "reviewing" : ""}`}
                viewBox={`${projectRef.current.map.bounds.x} ${projectRef.current.map.bounds.y} ${projectRef.current.map.bounds.width} ${projectRef.current.map.bounds.height}`}
                preserveAspectRatio="xMidYMid meet"
              >
                {projectRef.current.training.modules.map((module, index) => {
                  if (scope.type === "module" && index !== scope.index)
                    return null;
                  return (
                    <g
                      className={index === activeIndex ? "active" : ""}
                      key={module.id}
                    >
                      <rect className="start" {...module.trigger.bounds} />
                      <rect className="end" {...module.end_trigger.bounds} />
                    </g>
                  );
                })}
                {review &&
                  review.nodes.map((node, index) => {
                    const snapshot =
                      review.snapshots[node.frame + 1] ?? review.after;
                    return (
                      <g
                        className={
                          index === review.currentNode
                            ? "recorded-node selected"
                            : "recorded-node"
                        }
                        key={node.id}
                      >
                        <circle
                          cx={snapshot.pos.x}
                          cy={snapshot.pos.y - 6}
                          r="4"
                        />
                        <text x={snapshot.pos.x + 6} y={snapshot.pos.y - 7}>
                          {index + 1}
                        </text>
                      </g>
                    );
                  })}
              </svg>
              {review && reviewNode && reviewSnapshot && (
                <div
                  className="training-record-objective-window"
                  style={{ left: windowLeft, top: windowTop }}
                >
                  <div>
                    <header>
                      <small>
                        关键节点 {review.currentNode + 1}/{review.nodes.length}
                      </small>
                      <strong>{reviewNode.label}</strong>
                    </header>
                    <section className="training-record-target-groups">
                      {RECORDING_TARGET_GROUPS.map((group) => (
                        <article
                          className="training-record-target-group"
                          key={group.id}
                        >
                          <strong>{group.label}</strong>
                          <div>
                            {group.options.map((option) => (
                              <button
                                aria-pressed={reviewTargets.includes(option.id)}
                                className={
                                  reviewTargets.includes(option.id)
                                    ? "selected"
                                    : ""
                                }
                                key={option.id}
                                onClick={() => toggleReviewTarget(option.id)}
                              >
                                <span>{option.label}</span>
                                <b>
                                  {recordingTargetCondition(
                                    option.id,
                                    reviewSnapshot,
                                    review.initial,
                                  )}
                                </b>
                              </button>
                            ))}
                          </div>
                        </article>
                      ))}
                    </section>
                    <footer>
                      <button
                        disabled={review.currentNode === 0}
                        onClick={() =>
                          showReviewNode(review, review.currentNode - 1)
                        }
                      >
                        上一步
                      </button>
                      <button className="danger" onClick={deleteReviewNode}>
                        删除关键点
                      </button>
                      <button
                        className="primary"
                        onClick={() => {
                          if (review.currentNode === review.nodes.length - 1)
                            finishReview();
                          else showReviewNode(review, review.currentNode + 1);
                        }}
                      >
                        {review.currentNode === review.nodes.length - 1
                          ? "生成教程"
                          : "下一步"}
                      </button>
                    </footer>
                  </div>
                </div>
              )}
            </>
          );
        }}
      </GameView>
      <div className={`training-recorder-hud ${phase}`}>
        <i />
        <div>
          <strong>
            {phase === "loading"
              ? "载入中"
              : phase === "armed"
                ? "已暂停待命"
                : phase === "recording"
                  ? `REC · F${recordingFrames.current.length - 1}`
                  : phase === "reviewing"
                    ? `目标节点 ${review ? review.currentNode + 1 : 0}/${review?.nodes.length ?? 0}`
                    : phase === "done"
                      ? "录制完成"
                      : "寻找开始区"}
          </strong>
          <span>
            {phase === "armed"
              ? "按 Jump / Dash / Crouch Dash / Grab 中任一个动作开始；WASD 仅作为后台方向上下文。"
              : phase === "recording"
                ? "粉色框是当前教程结束区；进入后自动生成并保存 JSON 数据。"
                : phase === "reviewing"
                  ? "点击卡片切换目标；可以不选，也可以删除不需要的关键点。"
                  : "方向键可正常游玩；进入蓝色开始区后等待第一个教程动作。"}
          </span>
        </div>
      </div>
    </main>
  );
}
