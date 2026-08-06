import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { GameView, type LiveRenderRefs } from "./components/GameView";
import { GameplayStrawberry } from "./components/GameplaySprite";
import { InputTimeline } from "./components/InputTimeline";
import { KeyBindings } from "./components/KeyBindings";
import { EditorWorkspace } from "./components/EditorWorkspace";
import { StateInspector } from "./components/StateInspector";
import { TrainingGround } from "./components/TrainingGround";
import { trainingCatalog } from "./training/catalog";
import {
  StartSettings,
  type StartConfiguration,
} from "./components/StartSettings";
import {
  ACTIONS,
  ACTION_GLYPHS,
  ACTION_LABELS,
  DEFAULT_BINDINGS,
  PLAYGROUND,
  bindingLabel,
  buttonsToInput,
  createInitialState,
  makeEmptyButtons,
  type Action,
  type FrameButtons,
  type GymMap,
  type KeyBindings as Bindings,
  type SimState,
} from "./model";
import { FrameCache } from "./simulator/frameCache";
import { WasmClient } from "./simulator/wasmClient";
import {
  compareTraces,
  createWebTrace,
  initialStateFromTrace,
  parseTrace,
} from "./recording/trace";
import {
  DEFAULT_GAMEPAD_DIRECTION_SOURCE,
  buttonsEqual,
  buttonsFromGamepad,
  isGamepadDirectionSource,
  latchNewButtons,
  mergeButtons,
  type GamepadDirectionSource,
} from "./input/gamepad";
import {
  DEFAULT_VISUAL_THEME_ID,
  VISUAL_THEME_COLLECTIONS,
  VISUAL_THEMES,
  isVisualThemeId,
  visualThemeById,
  type VisualThemeId,
} from "./visualThemes";
import { ThemePicker } from "./components/ThemePicker";
import { AIViewer } from "./components/AIViewer";

interface RunDocument {
  version: 2;
  map: GymMap;
  initial_state: SimState;
  inputs: FrameButtons[];
  bindings: Bindings;
}

const DEFAULT_ROOM = "playground";
const MAX_ANIMATION_DELTA_MS = 250;
const LIVE_RENDER_HISTORY_FRAMES = 240;
/** How often the live loop flushes state into React (UI panels). The canvas
 * reads the refs every frame, so this only bounds panel re-renders. */
const LIVE_UI_UPDATE_INTERVAL_MS = 100;
const VISUAL_THEME_STORAGE_KEY = "celeste-gym-visual-theme";

type AppMode = "play" | "training" | "editor" | "ai" | "advanced";

const MODE_OPTIONS: readonly {
  id: AppMode;
  label: string;
  subtitle: string;
}[] = [
  { id: "play", label: "自由攀登", subtitle: "PLAY" },
  { id: "training", label: "技巧训练", subtitle: "TRAIN" },
  { id: "editor", label: "地图工坊", subtitle: "BUILD" },
  { id: "ai", label: "AI 观察", subtitle: "AGENT" },
  { id: "advanced", label: "逐帧研究", subtitle: "LAB" },
];

function loadBindings(): Bindings {
  try {
    const saved = JSON.parse(
      localStorage.getItem("celeste-gym-bindings") ?? "",
    ) as Partial<Bindings>;
    return { ...DEFAULT_BINDINGS, ...saved };
  } catch {
    return { ...DEFAULT_BINDINGS };
  }
}

function loadGamepadDirectionSource(): GamepadDirectionSource {
  const saved = localStorage.getItem("celeste-gym-gamepad-direction");
  return isGamepadDirectionSource(saved)
    ? saved
    : DEFAULT_GAMEPAD_DIRECTION_SOURCE;
}

function loadVisualThemeId(): VisualThemeId {
  const saved = localStorage.getItem(VISUAL_THEME_STORAGE_KEY);
  return isVisualThemeId(saved) ? saved : DEFAULT_VISUAL_THEME_ID;
}

function download(name: string, contents: string): void {
  const url = URL.createObjectURL(
    new Blob([contents], { type: "application/json" }),
  );
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = name;
  anchor.click();
  URL.revokeObjectURL(url);
}

function buttonsFromKeys(
  keys: ReadonlySet<string>,
  bindings: Bindings,
  latched?: ReadonlySet<string>,
): FrameButtons {
  const buttons = makeEmptyButtons();
  for (const action of ACTIONS) {
    const code = bindings[action];
    buttons[action] = keys.has(code) || Boolean(latched?.has(code));
  }
  return buttons;
}

export default function App() {
  const client = useMemo(() => new WasmClient(), []);
  const [map, setMap] = useState<GymMap>(() => structuredClone(PLAYGROUND));
  const cache = useMemo(
    () => new FrameCache(client, map, createInitialState(map), 360),
    [client],
  );
  const [mode, setMode] = useState<AppMode>("training");
  const [otherModesOpen, setOtherModesOpen] = useState(false);
  const [editorExperiencing, setEditorExperiencing] = useState(false);
  const [trainingTechniqueId, setTrainingTechniqueId] = useState(
    trainingCatalog[0].id,
  );
  const [trainingVariantId, setTrainingVariantId] = useState(
    trainingCatalog[0].variants[0].id,
  );
  const [liveState, setLiveState] = useState<SimState>(() =>
    createInitialState(map),
  );
  const liveStateRef = useRef(liveState);
  const [liveFrame, setLiveFrame] = useState(0);
  const liveFrameRef = useRef(0);
  const liveHistoryRef = useRef<{
    startFrame: number;
    states: SimState[];
  }>({ startFrame: 0, states: [liveState] });
  const lastLiveUiUpdate = useRef(0);
  const liveStaleRef = useRef(false);
  const livePreviousButtons = useRef<FrameButtons>(makeEmptyButtons());
  const liveRenderRefs = useMemo(
    () => ({
      state: liveStateRef,
      frame: liveFrameRef,
      history: liveHistoryRef,
      stale: liveStaleRef,
    }),
    [],
  );
  const [, redraw] = useState(0);
  const [frame, setFrame] = useState(0);
  const frameRef = useRef(frame);
  const [playing, setPlaying] = useState(false);
  const [recording, setRecording] = useState(false);
  const [speed, setSpeed] = useState(1);
  const [wasmStatus, setWasmStatus] = useState<"loading" | "ready" | "error">(
    "loading",
  );
  const [notice, setNotice] = useState("WASM 核心启动中…");
  const [bindings, setBindings] = useState<Bindings>(loadBindings);
  const [gamepadDirectionSource, setGamepadDirectionSource] =
    useState<GamepadDirectionSource>(loadGamepadDirectionSource);
  const [gamepadName, setGamepadName] = useState<string | null>(null);
  const [visualThemeId, setVisualThemeId] =
    useState<VisualThemeId>(loadVisualThemeId);
  const [liveButtons, setLiveButtons] =
    useState<FrameButtons>(makeEmptyButtons);
  const [bindingsOpen, setBindingsOpen] = useState(false);
  const [startSettingsOpen, setStartSettingsOpen] = useState(false);
  const [startSettingsBusy, setStartSettingsBusy] = useState(false);
  const keys = useRef(new Set<string>());
  const latched = useRef(new Set<string>());
  const gamepadButtons = useRef<FrameButtons>(makeEmptyButtons());
  const gamepadLatched = useRef<FrameButtons>(makeEmptyButtons());
  const advancing = useRef(false);
  const calculationRevision = useRef(0);

  const replaceLiveSession = useCallback((initial: SimState) => {
    liveStateRef.current = initial;
    setLiveState(initial);
    liveFrameRef.current = 0;
    setLiveFrame(0);
    liveHistoryRef.current = { startFrame: 0, states: [initial] };
    livePreviousButtons.current = makeEmptyButtons();
  }, []);

  /** Mirror the frame-cache cursor into the live render refs so the
   * advanced-mode GameView keeps drawing at full cadence while React state
   * updates are throttled to ~10Hz. */
  const syncPlaybackRefs = useCallback(() => {
    const current = frameRef.current;
    const exact = cache.getState(current);
    const visible = exact
      ? { frame: current, state: exact }
      : cache.getNearestState(current);
    liveFrameRef.current = visible.frame;
    liveStateRef.current = visible.state;
    liveHistoryRef.current.startFrame = 0;
    liveHistoryRef.current.states = cache.getStates() as SimState[];
    liveStaleRef.current = !exact || visible.frame !== current;
  }, [cache]);

  useEffect(
    () =>
      cache.subscribe(() => {
        redraw((value) => value + 1);
        syncPlaybackRefs();
      }),
    [cache, syncPlaybackRefs],
  );

  useEffect(() => {
    if (mode === "advanced") syncPlaybackRefs();
  }, [mode, syncPlaybackRefs]);
  useEffect(() => {
    frameRef.current = frame;
  }, [frame]);
  useEffect(() => () => client.dispose(), [client]);

  useEffect(() => {
    let active = true;
    client
      .ready()
      .then(() => {
        if (!active) return;
        setWasmStatus("ready");
        setNotice("WASM 核心已就绪");
      })
      .catch((error: Error) => {
        if (!active) return;
        setWasmStatus("error");
        setNotice(`WASM 加载失败：${error.message}`);
      });
    return () => {
      active = false;
    };
  }, [client]);

  useEffect(() => {
    if (
      (mode !== "play" && !(mode === "editor" && editorExperiencing)) ||
      wasmStatus !== "ready"
    )
      return;
    let active = true;
    let animation = 0;
    let last = performance.now();
    let carry = 0;
    let simulating = false;

    const resetClock = () => {
      last = performance.now();
      carry = 0;
    };

    const tick = (now: number) => {
      const elapsed = now - last;
      last = now;
      if (document.hidden || elapsed > MAX_ANIMATION_DELTA_MS) carry = 0;
      else carry += (elapsed * 60) / 1000;
      const steps = Math.min(6, Math.floor(carry));
      if (steps > 0 && !simulating) {
        carry -= steps;
        let previous = livePreviousButtons.current;
        const inputs = Array.from({ length: steps }, (_, offset) => {
          const keyboard = buttonsFromKeys(
            keys.current,
            bindings,
            offset === 0 ? latched.current : undefined,
          );
          const gamepad =
            offset === 0
              ? mergeButtons(gamepadButtons.current, gamepadLatched.current)
              : gamepadButtons.current;
          const current = mergeButtons(keyboard, gamepad);
          const input = buttonsToInput(current, previous);
          previous = current;
          return input;
        });
        latched.current.clear();
        gamepadLatched.current = makeEmptyButtons();
        livePreviousButtons.current = previous;
        simulating = true;
        void client
          .simulate(liveStateRef.current, inputs, map)
          .then((trace) => {
            if (!active) return;
            const current = trace.at(-1);
            if (!current) throw new Error("WASM 未返回游玩状态");
            const history = liveHistoryRef.current;
            // Mutate the history window in place: the rAF render loop and the
            // throttled React state below both read the same array, so each
            // tick stops allocating a fresh copy (the old spread was the main
            // source of the per-frame GC churn measured in the trace).
            for (let index = 1; index < trace.length; index += 1)
              history.states.push(trace[index]);
            if (history.states.length > LIVE_RENDER_HISTORY_FRAMES) {
              const removed = history.states.length - LIVE_RENDER_HISTORY_FRAMES;
              history.states.splice(0, removed);
              history.startFrame += removed;
            }
            liveStateRef.current = current;
            liveFrameRef.current += steps;
            // Throttle React-bound state to ~10Hz. GameView draws from the
            // refs at full cadence, so the UI panels update at 10Hz while the
            // canvas keeps running at 60/120fps.
            const now = performance.now();
            if (now - lastLiveUiUpdate.current >= LIVE_UI_UPDATE_INTERVAL_MS) {
              lastLiveUiUpdate.current = now;
              setLiveState(current);
              setLiveFrame(liveFrameRef.current);
            }
          })
          .catch((error: Error) => {
            if (active) setNotice(error.message);
          })
          .finally(() => {
            simulating = false;
          });
      }
      if (active) animation = requestAnimationFrame(tick);
    };
    document.addEventListener("visibilitychange", resetClock);
    animation = requestAnimationFrame(tick);
    return () => {
      active = false;
      cancelAnimationFrame(animation);
      document.removeEventListener("visibilitychange", resetClock);
    };
  }, [bindings, client, editorExperiencing, map, mode, wasmStatus]);

  useEffect(() => {
    const down = (event: KeyboardEvent) => {
      if (
        event.target instanceof HTMLElement &&
        event.target.matches("input, textarea, select, button")
      )
        return;
      if (!keys.current.has(event.code)) {
        latched.current.add(event.code);
        keys.current.add(event.code);
        setLiveButtons(
          mergeButtons(
            buttonsFromKeys(keys.current, bindings),
            gamepadButtons.current,
          ),
        );
      }
      keys.current.add(event.code);
      if (Object.values(bindings).includes(event.code)) event.preventDefault();
    };
    const up = (event: KeyboardEvent) => {
      if (keys.current.delete(event.code))
        setLiveButtons(
          mergeButtons(
            buttonsFromKeys(keys.current, bindings),
            gamepadButtons.current,
          ),
        );
    };
    const blur = () => {
      keys.current.clear();
      latched.current.clear();
      gamepadButtons.current = makeEmptyButtons();
      gamepadLatched.current = makeEmptyButtons();
      setLiveButtons(makeEmptyButtons());
    };
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    window.addEventListener("blur", blur);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
      window.removeEventListener("blur", blur);
    };
  }, [bindings]);

  const gamepadSupported =
    typeof navigator !== "undefined" &&
    typeof navigator.getGamepads === "function";

  useEffect(() => {
    if (!gamepadSupported) return;
    let animation = 0;

    const poll = () => {
      const gamepad =
        Array.from(navigator.getGamepads()).find(
          (candidate) => candidate?.connected,
        ) ?? null;
      const next = gamepad
        ? buttonsFromGamepad(gamepad, gamepadDirectionSource)
        : makeEmptyButtons();
      gamepadLatched.current = latchNewButtons(
        gamepadButtons.current,
        next,
        gamepadLatched.current,
      );
      gamepadButtons.current = next;
      setGamepadName((current) =>
        current === gamepad?.id ? current : (gamepad?.id ?? null),
      );
      const combined = mergeButtons(
        buttonsFromKeys(keys.current, bindings),
        next,
      );
      setLiveButtons((current) =>
        buttonsEqual(current, combined) ? current : combined,
      );
      animation = requestAnimationFrame(poll);
    };

    animation = requestAnimationFrame(poll);
    return () => cancelAnimationFrame(animation);
  }, [bindings, gamepadDirectionSource, gamepadSupported]);

  const seek = useCallback(
    (target: number, pause = true) => {
      const requested = Math.max(0, Math.round(target));
      cache.ensureCapacity(requested);
      const next = Math.min(cache.frameCount, requested);
      if (pause) {
        setPlaying(false);
        setRecording(false);
      }
      frameRef.current = next;
      setFrame(next);
      syncPlaybackRefs();
      if (!cache.getState(next)) {
        const requestRevision = ++calculationRevision.current;
        setNotice(`从最近检查点计算到 F${next}…`);
        void cache
          .ensureFrame(next)
          .then((state) => {
            if (
              state &&
              requestRevision === calculationRevision.current &&
              frameRef.current === next
            )
              setNotice(`F${next} 已由 WASM 计算并缓存`);
          })
          .catch((error: Error) => {
            if (requestRevision === calculationRevision.current)
              setNotice(error.message);
          });
      }
    },
    [cache],
  );

  useEffect(() => {
    if ((!playing && !recording) || wasmStatus !== "ready") return;
    let active = true;
    let animation = 0;
    let last = performance.now();
    let carry = 0;

    const resetClock = () => {
      last = performance.now();
      carry = 0;
    };

    const sampleButtons = (includeLatched: boolean): FrameButtons => {
      const keyboard = buttonsFromKeys(
        keys.current,
        bindings,
        includeLatched ? latched.current : undefined,
      );
      const gamepad = includeLatched
        ? mergeButtons(gamepadButtons.current, gamepadLatched.current)
        : gamepadButtons.current;
      return mergeButtons(keyboard, gamepad);
    };

    const tick = (now: number) => {
      const elapsed = now - last;
      last = now;
      if (document.hidden || elapsed > MAX_ANIMATION_DELTA_MS) carry = 0;
      else carry += ((elapsed * 60) / 1000) * speed;
      const steps = Math.min(6, Math.floor(carry));
      if (steps > 0 && !advancing.current) {
        carry -= steps;
        const start = frameRef.current;
        const previousCapacity = cache.frameCount;
        const target = start + steps;
        cache.ensureCapacity(target);
        if (cache.frameCount !== previousCapacity)
          setNotice(`时间线容量自动扩展到 ${cache.frameCount} 帧`);
        if (recording) {
          cache.setButtonsRange(
            start,
            Array.from({ length: steps }, (_, offset) =>
              sampleButtons(offset === 0),
            ),
          );
          latched.current.clear();
          gamepadLatched.current = makeEmptyButtons();
        }
        advancing.current = true;
        void cache
          .ensureFrame(target)
          .then((state) => {
            if (!active || !state) return;
            frameRef.current = target;
            liveFrameRef.current = target;
            liveStateRef.current = state;
            const now = performance.now();
            if (now - lastLiveUiUpdate.current >= LIVE_UI_UPDATE_INTERVAL_MS) {
              lastLiveUiUpdate.current = now;
              setFrame(target);
            }
          })
          .catch((error: Error) => {
            setPlaying(false);
            setRecording(false);
            setNotice(error.message);
          })
          .finally(() => {
            advancing.current = false;
          });
      }
      if (active) animation = requestAnimationFrame(tick);
    };
    document.addEventListener("visibilitychange", resetClock);
    animation = requestAnimationFrame(tick);
    return () => {
      active = false;
      cancelAnimationFrame(animation);
      document.removeEventListener("visibilitychange", resetClock);
    };
  }, [bindings, cache, playing, recording, speed, wasmStatus]);

  const paintInputs = useCallback(
    (action: Action, from: number, to: number, value: boolean) => {
      cache.paint(action, from, to, value);
    },
    [cache],
  );
  const moveInputs = useCallback(
    (
      action: Action,
      targetAction: Action,
      start: number,
      end: number,
      delta: number,
    ): { start: number; end: number } =>
      cache.moveRun(action, targetAction, start, end, delta),
    [cache],
  );
  const completeInputEdit = useCallback(() => {
    const current = frameRef.current;
    if (!cache.getState(current)) seek(current, false);
  }, [cache, seek]);
  const resizeInputs = useCallback(
    (frames: number) => {
      cache.resize(frames);
      if (frameRef.current > cache.frameCount) seek(cache.frameCount);
    },
    [cache, seek],
  );

  const inputs = cache.getInputs();
  const states = cache.getStates();
  const exactState = cache.getState(frame);
  const visible = exactState
    ? { frame, state: exactState }
    : cache.getNearestState(frame);
  const selectableStartMaps = useMemo(
    () => [
      map,
      ...(PLAYGROUND.room === map.room
        ? []
        : [structuredClone(PLAYGROUND)]),
    ],
    [map],
  );

  const changeBinding = useCallback((action: Action, code: string) => {
    setBindings((current) => {
      const next = { ...current };
      const collision = ACTIONS.find(
        (candidate) => candidate !== action && current[candidate] === code,
      );
      if (collision) next[collision] = current[action];
      next[action] = code;
      localStorage.setItem("celeste-gym-bindings", JSON.stringify(next));
      return next;
    });
  }, []);

  const changeGamepadDirectionSource = useCallback(
    (source: GamepadDirectionSource) => {
      setGamepadDirectionSource(source);
      localStorage.setItem("celeste-gym-gamepad-direction", source);
    },
    [],
  );

  const toggleRecording = () => {
    setPlaying(false);
    setRecording((current) => {
      const next = !current;
      if (next) {
        latched.current.clear();
        gamepadLatched.current = makeEmptyButtons();
      }
      return next;
    });
  };

  const reset = () => {
    setPlaying(false);
    setRecording(false);
    cache.reset(createInitialState(map));
    seek(0);
    setNotice("输入和逐帧缓存已清空");
  };

  const applyStartConfiguration = async ({
    room,
    position,
  }: StartConfiguration) => {
    setPlaying(false);
    setRecording(false);
    setStartSettingsBusy(true);
    setNotice(`正在应用房间 ${room} 的新起点…`);
    try {
      const decodedMap = selectableStartMaps.find(
        (candidate) => candidate.room === room,
      );
      if (!decodedMap) throw new Error(`房间 ${room} 不在可选地图中`);
      const configuredMap: GymMap = {
        ...structuredClone(decodedMap),
        room,
        spawn: { ...position },
      };
      calculationRevision.current += 1;
      setMap(configuredMap);
      const initial = createInitialState(configuredMap);
      cache.replace(
        configuredMap,
        initial,
        cache.getInputs().map((input) => ({ ...input })),
      );
      replaceLiveSession(initial);
      frameRef.current = 0;
      setFrame(0);
      syncPlaybackRefs();
      setStartSettingsOpen(false);
      setNotice(
        `已从房间 ${room} 的 (${position.x}, ${position.y}) 开始，时间线输入已保留`,
      );
      void cache.ensureFrame(1);
    } catch (error) {
      setNotice(
        error instanceof Error
          ? `起点设置失败：${error.message}`
          : "起点设置失败",
      );
    } finally {
      setStartSettingsBusy(false);
    }
  };

  const exportRun = () => {
    const document: RunDocument = {
      version: 2,
      map,
      initial_state: cache.getState(0) ?? createInitialState(map),
      inputs: inputs.map((input) => ({ ...input })),
      bindings,
    };
    download("celeste-gym-timeline.json", JSON.stringify(document, null, 2));
    setNotice("时间线已导出");
  };

  const exportTrace = async () => {
    try {
      const endFrame = frameRef.current;
      await cache.ensureFrame(endFrame);
      const trace = createWebTrace(
        map,
        cache.getInputs(),
        cache.getStates(),
        endFrame,
        undefined,
        cache.getSimulationInputs(endFrame),
      );
      download(
        `celeste-gym-web-${endFrame}-frames.trace.json`,
        JSON.stringify(trace, null, 2),
      );
      setNotice(
        `已导出 F0–F${endFrame} 的输入和 ${trace.states.length} 个逐帧状态`,
      );
    } catch (error) {
      setNotice(error instanceof Error ? error.message : "逐帧数据导出失败");
    }
  };

  const compareTrace = async (file: File) => {
    try {
      const expected = parseTrace(JSON.parse(await file.text()));
      const endFrame = expected.inputs.length;
      const comparisonMap = expected.map.data ?? map;
      setPlaying(false);
      setRecording(false);
      const traceMap = { ...comparisonMap, room: expected.map.room };
      setMap(traceMap);
      const initial = initialStateFromTrace(expected.states[0], traceMap);
      cache.replaceSimulationInputs(traceMap, initial, expected.inputs);
      replaceLiveSession(initial);
      await cache.ensureFrame(endFrame);
      frameRef.current = endFrame;
      setFrame(endFrame);
      syncPlaybackRefs();
      const actual = createWebTrace(
        traceMap,
        cache.getInputs(),
        cache.getStates(),
        endFrame,
        undefined,
        cache.getSimulationInputs(endFrame),
      );
      const result = compareTraces(actual, expected);
      setNotice(
        result.matched
          ? `对比通过：${result.compared_frames} 帧，位置 ${result.max_position_error.toFixed(6)}，速度 ${result.max_speed_error.toFixed(6)}`
          : `对比失败：${result.reason}；位置 ${result.max_position_error.toFixed(6)}，速度 ${result.max_speed_error.toFixed(6)}`,
      );
    } catch (error) {
      setNotice(error instanceof Error ? error.message : "逐帧数据对比失败");
    }
  };

  const importRun = async (file: File) => {
    try {
      const document = JSON.parse(await file.text()) as RunDocument;
      if (
        document.version !== 2 ||
        !document.map ||
        !document.initial_state ||
        !Array.isArray(document.inputs)
      )
        throw new Error("不是 Celeste Next Gym v2 时间线");
      setPlaying(false);
      setRecording(false);
      setMap(document.map);
      cache.replace(document.map, document.initial_state, document.inputs);
      replaceLiveSession(document.initial_state);
      if (document.bindings)
        setBindings({ ...DEFAULT_BINDINGS, ...document.bindings });
      seek(0);
      setNotice(`已导入 ${file.name} · 后续 state 将按需重算`);
    } catch (error) {
      setNotice(error instanceof Error ? error.message : "导入失败");
    }
  };

  const resetLiveMap = useCallback(
    (nextMap: GymMap = map) => {
      replaceLiveSession(createInitialState(nextMap));
    },
    [map, replaceLiveSession],
  );

  const updateEditorMap = useCallback(
    (nextMap: GymMap) => {
      setMap(nextMap);
      replaceLiveSession(createInitialState(nextMap));
    },
    [replaceLiveSession],
  );

  const toggleEditorExperience = useCallback(
    (next: boolean, editorMap: GymMap) => {
      setMap(editorMap);
      resetLiveMap(editorMap);
      setEditorExperiencing(next);
      setNotice(
        next
          ? "实时体验已启动 · 输入直接送入 WASM，不记录 state"
          : "已返回地图编辑",
      );
    },
    [resetLiveMap],
  );

  const selectMode = (nextMode: AppMode) => {
    if (nextMode === "play" || nextMode === "editor") {
      setPlaying(false);
      setRecording(false);
      setBindingsOpen(false);
      setStartSettingsOpen(false);
    }
    if (mode === "editor" && nextMode !== "editor") {
      setEditorExperiencing(false);
      const initial = createInitialState(map);
      cache.replace(
        map,
        initial,
        cache.getInputs().map((input) => ({ ...input })),
      );
      replaceLiveSession(initial);
      frameRef.current = 0;
      setFrame(0);
      syncPlaybackRefs();
    } else if (nextMode === "editor") {
      resetLiveMap();
    }
    setOtherModesOpen(false);
    setMode(nextMode);
  };
  const selectTraining = useCallback(
    (techniqueId: string, variantId: string) => {
      setTrainingTechniqueId(techniqueId);
      setTrainingVariantId(variantId);
    },
    [],
  );
  const selectedTrainingTechnique =
    trainingCatalog.find((item) => item.id === trainingTechniqueId) ??
    trainingCatalog[0];
  const selectedTrainingVariant =
    selectedTrainingTechnique.variants.find(
      (item) => item.id === trainingVariantId,
    ) ?? selectedTrainingTechnique.variants[0];
  const visualTheme = visualThemeById(visualThemeId);

  const selectVisualTheme = (id: VisualThemeId) => {
    setVisualThemeId(id);
    localStorage.setItem(VISUAL_THEME_STORAGE_KEY, id);
  };

  return (
    <div
      className={`app-shell ${mode === "play" ? "play-mode" : mode === "training" ? "training-mode" : mode === "editor" ? "editor-mode" : mode === "ai" ? "ai-mode" : "advanced-mode"}`}
      data-visual-theme={visualTheme.id}
    >
      {mode === "advanced" && <div className="mountain-backdrop" />}
      <header className="topbar">
        <div className="brand-mark">
          <GameplayStrawberry scale={3} />
          <div>
            <strong>CELESTE</strong>
            <em>NEXT GYM</em>
          </div>
        </div>
        <nav className="celeste-mode-menu" aria-label="工作区">
          {MODE_OPTIONS.filter((option) => option.id === "training").map(
            (option) => (
              <button
                type="button"
                key={option.id}
                aria-current={mode === option.id ? "page" : undefined}
                onClick={() => selectMode(option.id)}
              >
                <span>{option.subtitle}</span>
                <strong>{option.label}</strong>
              </button>
            ),
          )}
          <div className="celeste-other-modes">
            <button
              type="button"
              className="celeste-other-toggle"
              aria-expanded={otherModesOpen}
              aria-controls="celeste-other-mode-list"
              onClick={() => setOtherModesOpen((open) => !open)}
            >
              <span>MORE</span>
              <strong>其他</strong>
              <i aria-hidden="true">{otherModesOpen ? "▲" : "▼"}</i>
            </button>
            {otherModesOpen && (
              <div
                className="celeste-other-mode-list"
                id="celeste-other-mode-list"
              >
                {MODE_OPTIONS.filter((option) => option.id !== "training").map(
                  (option) => (
                    <button
                      type="button"
                      key={option.id}
                      aria-current={mode === option.id ? "page" : undefined}
                      onClick={() => selectMode(option.id)}
                    >
                      <span>{option.subtitle}</span>
                      <strong>{option.label}</strong>
                    </button>
                  ),
                )}
              </div>
            )}
          </div>
        </nav>
        <label className="mode-tabs">
          <small>工作区</small>
          <select
            aria-label="页面模式"
            value={mode}
            onChange={(event) => selectMode(event.target.value as AppMode)}
          >
            <option value="play">游玩</option>
            <option value="training">训练</option>
            <option value="editor">编辑</option>
            <option value="advanced">高级</option>
          </select>
        </label>
        <label className="visual-theme-picker">
          <small>场景主题</small>
          <ThemePicker
            themes={VISUAL_THEMES}
            collections={VISUAL_THEME_COLLECTIONS}
            value={visualThemeId}
            onChange={(id) => selectVisualTheme(id)}
          />
        </label>
        {mode === "advanced" ? (
          <div className="top-actions">
            <button
              disabled={wasmStatus !== "ready"}
              onClick={() => setStartSettingsOpen(true)}
            >
              起点
            </button>
            <button onClick={() => setBindingsOpen(true)}>控制</button>
            <label className="file-button">
              导入时间线
              <input
                type="file"
                accept="application/json,.json"
                onChange={(event) =>
                  event.target.files?.[0] &&
                  void importRun(event.target.files[0])
                }
              />
            </label>
            <button onClick={exportRun}>导出时间线</button>
            <button onClick={() => void exportTrace()}>导出逐帧</button>
            <label className="file-button">
              对比逐帧
              <input
                type="file"
                accept="application/json,.json"
                onChange={(event) =>
                  event.target.files?.[0] &&
                  void compareTrace(event.target.files[0])
                }
              />
            </label>
          </div>
        ) : mode === "training" ? (
          <div className="play-quick-actions training-context">
            <div className="play-room">
              <small>TRAINING MAP</small>
              <strong>
                {selectedTrainingTechnique.title} ·{" "}
                {selectedTrainingVariant.title}
              </strong>
              <span>
                {selectedTrainingVariant.training.modules.length} 个教程模块 ·{" "}
                {selectedTrainingVariant.summary}
              </span>
            </div>
            <div className="top-actions">
              <button onClick={() => setBindingsOpen(true)}>控制</button>
            </div>
          </div>
        ) : mode === "editor" ? (
          <div className="play-quick-actions editor-context">
            <div className="play-room">
              <small>
                {editorExperiencing ? "LIVE EXPERIENCE" : "MAP EDITOR"}
              </small>
              <strong>{map.name}</strong>
              <span>{map.room ?? DEFAULT_ROOM}</span>
            </div>
            <div className="top-actions">
              <button onClick={() => setBindingsOpen(true)}>控制</button>
            </div>
          </div>
        ) : mode === "ai" ? (
          <div className="play-quick-actions ai-context">
            <div className="play-room">
              <small>RL AGENT</small>
              <strong>策略轨迹与地图感知</strong>
              <span>路径 · 局部 tile 视野 · 全局实体注意力</span>
            </div>
          </div>
        ) : (
          <div className="play-quick-actions">
            <div className="play-room">
              <small>LIVE ROOM</small>
              <strong>{map.name}</strong>
              <span>{map.room ?? DEFAULT_ROOM}</span>
            </div>
            <div className="top-actions">
              <button
                disabled={wasmStatus !== "ready"}
                onClick={() => setStartSettingsOpen(true)}
              >
                起点
              </button>
              <button onClick={() => setBindingsOpen(true)}>控制</button>
            </div>
          </div>
        )}
      </header>

      {mode === "play" ? (
        <main className="play-workspace">
          <GameView
            map={map}
            state={liveState}
            states={liveHistoryRef.current.states}
            stateFrameOffset={liveHistoryRef.current.startFrame}
            frame={liveFrame}
            stale={false}
            theme={visualTheme}
            liveRefs={liveRenderRefs}
          />
        </main>
      ) : mode === "training" ? (
        <TrainingGround
          techniqueId={selectedTrainingTechnique.id}
          variantId={selectedTrainingVariant.id}
          bindings={bindings}
          theme={visualTheme}
          onSelectTraining={selectTraining}
        />
      ) : mode === "editor" ? (
        <EditorWorkspace
          map={map}
          state={liveState}
          frame={liveFrame}
          states={liveHistoryRef.current.states}
          stateFrameOffset={liveHistoryRef.current.startFrame}
          theme={visualTheme}
          bindings={bindings}
          wasmClient={client}
          experiencing={editorExperiencing}
          ready={wasmStatus === "ready"}
          liveRefs={liveRenderRefs}
          onMapChange={updateEditorMap}
          onExperienceChange={toggleEditorExperience}
          onResetExperience={resetLiveMap}
        />
      ) : mode === "ai" ? (
        <AIViewer theme={visualTheme} />
      ) : (
        <>
          <main className="workspace">
            <section className="stage panel-frame">
              <div className="stage-header">
                <div>
                  <small>
                    CELESTE 1.4.0.0-FNA · ROOM {map.room ?? DEFAULT_ROOM} ·
                    START {map.spawn.x}, {map.spawn.y}
                  </small>
                  <h1>{map.name}</h1>
                </div>
                <div className="cache-meter">
                  <span>VALID THROUGH</span>
                  <strong>
                    F{String(cache.computedThrough).padStart(4, "0")}
                  </strong>
                </div>
              </div>
              <GameView
                map={map}
                state={visible.state}
                states={states}
                frame={visible.frame}
                stale={!exactState || visible.frame !== frame}
                theme={visualTheme}
                liveRefs={liveRenderRefs}
              />
              <div className="transport">
                <button aria-label="回到第一帧" onClick={() => seek(0)}>
                  │◀
                </button>
                <button aria-label="上一帧" onClick={() => seek(frame - 1)}>
                  ◀
                </button>
                <button
                  className="play-button"
                  disabled={wasmStatus !== "ready"}
                  aria-label={playing ? "暂停" : "播放"}
                  onClick={() => {
                    setRecording(false);
                    setPlaying((value) => !value);
                  }}
                >
                  {playing ? "Ⅱ" : "▶"}
                </button>
                <button aria-label="下一帧" onClick={() => seek(frame + 1)}>
                  ▶
                </button>
                <button
                  className={
                    recording ? "record-button active" : "record-button"
                  }
                  disabled={wasmStatus !== "ready"}
                  onClick={toggleRecording}
                >
                  <i />
                  {recording ? "录制中" : "录制输入"}
                </button>
                <select
                  aria-label="播放速度"
                  value={speed}
                  onChange={(event) => setSpeed(Number(event.target.value))}
                >
                  <option value={0.25}>0.25×</option>
                  <option value={0.5}>0.5×</option>
                  <option value={1}>1×</option>
                  <option value={2}>2×</option>
                </select>
                <button onClick={reset}>重置</button>
              </div>
            </section>

            <aside className="right-rail">
              <StateInspector
                frame={frame}
                state={visible.state}
                exact={Boolean(exactState)}
              />
              <section className="current-input panel-frame">
                <div className="panel-heading">
                  <div>
                    <small>
                      FRAME{" "}
                      {String(Math.min(frame, inputs.length - 1)).padStart(
                        4,
                        "0",
                      )}
                    </small>
                    <h2>当前输入</h2>
                  </div>
                </div>
                <div className="input-buttons">
                  {ACTIONS.map((action) => {
                    const inputFrame = Math.min(frame, inputs.length - 1);
                    const active = recording
                      ? liveButtons[action]
                      : (inputs[inputFrame]?.[action] ?? false);
                    return (
                      <button
                        key={action}
                        className={`${action} ${active ? "active" : ""}`}
                        onClick={() => {
                          cache.setFrame(inputFrame, action, !active);
                          seek(inputFrame, false);
                        }}
                      >
                        <span>{ACTION_GLYPHS[action]}</span>
                        <strong>{ACTION_LABELS[action]}</strong>
                        <kbd>{bindingLabel(bindings[action])}</kbd>
                      </button>
                    );
                  })}
                </div>
              </section>
              <div className="notice" role="status">
                <i className={wasmStatus} />
                {notice}
              </div>
            </aside>

            <InputTimeline
              frame={frame}
              inputs={inputs}
              states={states}
              onSeek={seek}
              onPaint={paintInputs}
              onMove={moveInputs}
              onEditComplete={completeInputEdit}
              onResize={resizeInputs}
            />
          </main>
          <footer>
            celeste-wasm 0.2.0 rebuilt · Celeste 1.4.0.0-fna Gameplay atlas ·
            CelesteGymPlayground/Playground.bin · 60 FPS
          </footer>
        </>
      )}
      {bindingsOpen && (
        <KeyBindings
          bindings={bindings}
          gamepadDirectionSource={gamepadDirectionSource}
          gamepadName={gamepadName}
          gamepadSupported={gamepadSupported}
          onChange={changeBinding}
          onGamepadDirectionSourceChange={changeGamepadDirectionSource}
          onClose={() => setBindingsOpen(false)}
        />
      )}
      {startSettingsOpen && (
        <StartSettings
          rooms={selectableStartMaps}
          room={map.room ?? DEFAULT_ROOM}
          position={map.spawn}
          busy={startSettingsBusy}
          onApply={(configuration) => {
            void applyStartConfiguration(configuration);
          }}
          onClose={() => setStartSettingsOpen(false)}
        />
      )}
    </div>
  );
}
